//! Email parsing and normalization.
//!
//! This module handles parsing raw email content from Git blobs into structured
//! data suitable for database storage and threading. It uses the `mailparse` crate
//! for MIME parsing and implements custom normalization logic for threading.
//!
//! # Key Responsibilities
//!
//! - **MIME Parsing**: Extract headers, body, and metadata from raw email bytes
//! - **Header Extraction**: Parse Message-ID, Subject, From, To, Cc, Date, etc.
//! - **Text Sanitization**: Remove invalid characters (NUL bytes) that PostgreSQL can't store
//! - **Subject Normalization**: Canonicalize subjects for threading fallback
//! - **Reference Parsing**: Extract In-Reply-To and References for threading
//! - **Address Parsing**: Extract and normalize email addresses
//!
//! # Threading Support
//!
//! The parser prepares data for the JWZ (Jamie Zawinski) threading algorithm:
//!
//! - **message_id**: Unique identifier (required for threading)
//! - **in_reply_to**: Direct parent reference
//! - **references**: Full reference chain from oldest to newest
//! - **normalized_subject**: Fallback for subject-based grouping
//!
//! # Subject Normalization
//!
//! Subject normalization removes common prefixes to identify related emails:
//!
//! - Removes: `Re:`, `Fwd:`, `Fw:`, `Aw:` (case-insensitive)
//! - Removes: `[PATCH]`, `[PATCH v2]`, `[PATCH 1/3]`, `[RFC]`, etc.
//! - Collapses: Multiple spaces → single space
//! - Converts: To lowercase for comparison
//!
//! Examples:
//! - `"Re: [PATCH v2] Fix memory leak"` → `"fix memory leak"`
//! - `"[RFC PATCH 1/3] Add new feature"` → `"add new feature"`
//!
//! # Error Handling
//!
//! Parsing can fail for various reasons:
//! - Missing Message-ID (required)
//! - Missing author email (required)
//! - Invalid MIME structure
//! - Malformed headers
//!
//! Failed parses are logged but don't stop the sync process. The parallel
//! parsing phase filters out failures and continues with successful parses.
//!
//! # Performance
//!
//! - CPU-intensive (MIME parsing, regex, UTF-8 decoding)
//! - Parallelized using Rayon in the orchestrator
//! - Memory-efficient (processes one email at a time)
//! - No database I/O during parsing

use crate::models::{PatchMetadata, PatchSection, PatchType};
use chrono::{DateTime, Duration, Utc};
use mailparse::{MailHeaderMap, ParsedMail, parse_mail};
use regex::Regex;
use std::sync::OnceLock;
use thiserror::Error;

/// Structured representation of a parsed email.
///
/// Contains all fields needed for database storage and threading.
/// Extracted from raw email bytes using `parse_email()`.
#[derive(Debug, Clone)]
pub struct ParsedEmail {
    pub message_id: String,
    pub subject: String,
    pub normalized_subject: String, // For threading fallback
    pub date: DateTime<Utc>,
    pub author_name: String,
    pub author_email: String,
    pub body: String,
    pub to_addrs: Vec<(String, String)>, // (name, email)
    pub cc_addrs: Vec<(String, String)>, // (name, email)
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub patch_type: PatchType,
    pub is_patch_only: bool,
    pub patch_metadata: Option<PatchMetadata>,
}

/// Maximum tolerated clock skew for future-dated emails.
const MAX_FUTURE_SKEW: Duration = Duration::hours(24);

/// Errors that can be returned while parsing and validating an email.
#[derive(Debug, Error)]
pub enum ParseEmailError {
    #[error("failed to parse MIME structure: {0}")]
    MimeParse(#[from] mailparse::MailParseError),
    #[error("missing Message-ID header")]
    MissingMessageId,
    #[error("missing author email for message {message_id}")]
    MissingAuthorEmail { message_id: String },
    #[error("missing Date header for message {message_id}")]
    MissingDate { message_id: String },
    #[error("invalid Date header `{raw}` for message {message_id}: {error}")]
    InvalidDate {
        message_id: String,
        raw: String,
        error: String,
    },
    #[error("future Date header `{raw}` for message {message_id}")]
    FutureDate { message_id: String, raw: String },
}

/// Regex used by the b4 patch tooling to detect inline diffs.
static B4_DIFF_RE: OnceLock<Regex> = OnceLock::new();

/// Regex used by the b4 patch tooling to detect diffstat summaries.
static B4_DIFFSTAT_RE: OnceLock<Regex> = OnceLock::new();

fn b4_diff_regex() -> &'static Regex {
    B4_DIFF_RE.get_or_init(|| {
        Regex::new(r"(?mi)^(---.*\n\+\+\+|GIT binary patch|diff --git \w/\S+ \w/\S+)")
            .expect("valid diff detection regex")
    })
}

fn b4_diffstat_regex() -> &'static Regex {
    B4_DIFFSTAT_RE.get_or_init(|| {
        Regex::new(r"(?mi)^\s*\d+ file.*\d+ (insertion|deletion)")
            .expect("valid diffstat detection regex")
    })
}

/// Sanitize text by removing NUL bytes that PostgreSQL cannot store
fn sanitize_text(text: &str) -> String {
    text.replace('\0', "").trim().to_string()
}

/// Clean and normalize message IDs by removing angle brackets and whitespace
fn normalize_message_id(msg_id: Option<String>) -> Option<String> {
    msg_id.and_then(|id| {
        let cleaned = id.trim().trim_matches(&['<', '>'][..]).trim();
        if cleaned.is_empty() {
            None
        } else {
            Some(sanitize_text(cleaned))
        }
    })
}

/// Parse email addresses from a header value
fn parse_email_addresses(header_value: &str) -> Vec<(String, String)> {
    let mut addresses = Vec::new();

    // Split by comma and parse each address
    for addr_str in header_value.split(',') {
        if let Ok(addr) = mailparse::addrparse(addr_str.trim()) {
            for single in addr.iter() {
                if let mailparse::MailAddr::Single(info) = single {
                    let name = info.display_name.clone().unwrap_or_default();
                    let email = info.addr.clone();
                    addresses.push((sanitize_text(&name), email.to_lowercase()));
                }
            }
        }
    }

    addresses
}

/// Normalize email subject for threading comparison.
///
/// Implements aggressive subject normalization to identify related emails even when
/// subject prefixes vary. This is used as a fallback threading mechanism when
/// reference headers are missing or incomplete.
///
/// # Algorithm
///
/// The normalization process runs in a loop, repeatedly removing prefixes until
/// no more matches are found:
///
/// 1. **Convert to lowercase**: Ensure case-insensitive comparison
/// 2. **Loop until stable**:
///    - Remove reply prefixes: `Re:`, `Fwd:`, `Fw:`, `Aw:` (German "Re:")
///    - Remove bracketed tags: `[PATCH]`, `[PATCH v2]`, `[RFC]`, `[PATCH 1/3]`, etc.
///    - Trim whitespace after each removal
///    - Repeat until no changes occur
/// 3. **Collapse whitespace**: Multiple spaces → single space
///
/// # Why the Loop?
///
/// Emails can have multiple layers of prefixes:
/// - `"Re: Re: [PATCH v2] Subject"` requires 3 removals
/// - `"[RFC PATCH] Subject"` requires 1 removal (whole bracket content)
/// - Loop ensures all prefixes are removed regardless of nesting
///
/// # Mailing List Patterns
///
/// Common kernel mailing list patterns handled:
/// - `[PATCH]` - Simple patch
/// - `[PATCH v2]` - Patch revision
/// - `[PATCH 1/3]` - Patch series
/// - `[RFC PATCH]` - Request for comments
/// - `[PATCH net-next]` - Subsystem tag
/// - `Re: [PATCH]` - Reply to patch
///
/// # Examples
///
/// ```rust,ignore
/// assert_eq!(normalize_subject("Re: [PATCH] Fix bug"), "fix bug");
/// assert_eq!(normalize_subject("[PATCH v2 1/3] Add feature"), "add feature");
/// assert_eq!(normalize_subject("Re: Fwd: [RFC] Question"), "question");
/// ```
///
/// # Performance
///
/// - Runs in O(n×m) where n = subject length, m = number of prefixes
/// - Typically 1-3 iterations for most emails
/// - Fast enough for parallel processing phase
///
/// # Threading Impact
///
/// JWZ algorithm uses normalized subjects to:
/// - Group emails with same normalized subject but different Message-IDs
/// - Handle broken email clients that don't set References headers
/// - Merge threads that split due to missing references
pub fn normalize_subject(subject: &str) -> String {
    let mut normalized = subject.trim().to_lowercase();

    // Keep removing prefixes until none match
    loop {
        let before = normalized.clone();

        // Remove Re:, Fwd:, Fw: prefixes (case insensitive, already lowercase)
        for prefix in &["re:", "fwd:", "fw:", "aw:"] {
            if normalized.starts_with(prefix) {
                normalized = normalized[prefix.len()..].trim_start().to_string();
            }
        }

        // Remove [PATCH], [PATCH v2], [PATCH 1/3], [RFC], etc.
        if normalized.starts_with('[') {
            if let Some(end_bracket) = normalized.find(']') {
                normalized = normalized[end_bracket + 1..].trim_start().to_string();
            }
        }

        // If nothing changed, we're done
        if before == normalized {
            break;
        }
    }

    // Collapse multiple spaces into one
    let words: Vec<&str> = normalized.split_whitespace().collect();
    words.join(" ")
}

/// Extract message IDs from References header
/// Uses whitespace-based splitting for better compatibility
fn extract_references(header_value: &str) -> Vec<String> {
    header_value
        .split_whitespace()
        .map(|id| {
            // Remove angle brackets and sanitize
            let cleaned = id.trim().trim_matches(&['<', '>'][..]);
            sanitize_text(cleaned)
        })
        .filter(|id| !id.is_empty())
        .collect()
}

/// Parse an email from raw bytes into structured data.
///
/// Main entry point for email parsing. Extracts all headers, body content,
/// and performs necessary sanitization for database storage.
///
/// # Arguments
///
/// - `blob_data`: Raw email bytes from Git blob (RFC 5322 format)
///
/// # Returns
///
/// - `Ok(ParsedEmail)`: Successfully parsed email with all fields
/// - `Err(...)`: Parse failure (missing required fields or MIME errors)
///
/// # Required Fields
///
/// These fields must be present or parsing fails:
/// - **Message-ID**: Unique identifier (after normalization)
/// - **Author Email**: Sender address (From header)
///
/// # Optional Fields (with defaults)
///
/// - **Subject**: Defaults to `"(No Subject)"`
/// - **Date**: Defaults to current UTC time if unparseable
/// - **Body**: Empty string if extraction fails
/// - **To/Cc**: Empty vectors if headers missing
/// - **In-Reply-To**: None if header missing
/// - **References**: Empty vector if header missing
///
/// # Processing Steps
///
/// 1. **MIME Parsing**: Parse raw bytes into MIME structure
/// 2. **Message-ID**: Extract and normalize (remove angle brackets)
/// 3. **Subject**: Extract and normalize for threading
/// 4. **Date**: Parse using dateparser (handles various formats)
/// 5. **Author**: Parse From header (name + email)
/// 6. **Body**: Extract from text/plain part (or fallback to root)
/// 7. **Recipients**: Parse To and Cc headers
/// 8. **Threading**: Extract In-Reply-To and References
/// 9. **Sanitization**: Remove NUL bytes from all text fields
///
/// # Body Extraction
///
/// For multipart emails:
/// - Searches for first `text/plain` part
/// - Falls back to root body if no text/plain found
/// - Ignores HTML-only content
///
/// For single-part emails:
/// - Uses root body directly
///
/// # Sanitization
///
/// All text fields are sanitized to remove NUL bytes (\0) because:
/// - PostgreSQL cannot store NUL bytes in text columns
/// - Some malformed emails contain NUL bytes
/// - Trimming whitespace for cleaner storage
///
/// # Error Cases
///
/// Returns error if:
/// - MIME parsing fails (corrupted email)
/// - Message-ID missing or empty after normalization
/// - Author email missing or empty
///
/// Warnings logged (but not errors):
/// - Missing Subject (uses placeholder)
/// - Missing body (empty string)
/// - Missing recipients (empty vectors)
///
/// # Performance
///
/// - CPU-intensive (MIME parsing, UTF-8 decoding)
/// - Called in parallel from Rayon thread pool
/// - Typical parse time: 1-5ms per email
/// - Memory usage: ~2x email size during parsing
pub fn parse_email(blob_data: &[u8]) -> Result<ParsedEmail, ParseEmailError> {
    let parsed = parse_mail(blob_data).map_err(|e| {
        log::debug!("failed to parse MIME: {}", e);
        ParseEmailError::MimeParse(e)
    })?;

    // Extract Message-ID (required)
    let message_id = normalize_message_id(parsed.headers.get_first_value("Message-ID"))
        .ok_or_else(|| {
            log::debug!("missing Message-ID header");
            ParseEmailError::MissingMessageId
        })?;

    // Extract subject
    let subject = parsed
        .headers
        .get_first_value("Subject")
        .map(|s| sanitize_text(&s))
        .unwrap_or_else(|| "(No Subject)".to_string());

    let date = parse_email_date(
        parsed.headers.get_first_value("Date"),
        &message_id,
        &subject,
    )?;

    // Parse author
    let from_str = parsed.headers.get_first_value("From").unwrap_or_default();

    let (author_name, author_email) = if let Ok(addrs) = mailparse::addrparse(&from_str) {
        if let Some(mailparse::MailAddr::Single(info)) = addrs.iter().next() {
            let name = info.display_name.clone().unwrap_or_default();
            let email = info.addr.clone();
            (sanitize_text(&name), email.to_lowercase())
        } else {
            (String::new(), String::new())
        }
    } else {
        (String::new(), String::new())
    };

    if author_email.is_empty() {
        log::warn!(
            "email {} ({}) missing author email, skipping",
            message_id,
            subject
        );
        return Err(ParseEmailError::MissingAuthorEmail {
            message_id: message_id.clone(),
        });
    }

    // Extract body using b4's fallback preference for diff-carrying parts.
    let body = sanitize_text(&extract_preferred_body(&parsed));

    let (patch_type, is_patch_only, patch_metadata) = analyze_patch(&parsed, &body);

    // Parse recipients
    let to_addrs = parsed
        .headers
        .get_first_value("To")
        .map(|v| parse_email_addresses(&v))
        .unwrap_or_default();

    let cc_addrs = parsed
        .headers
        .get_first_value("Cc")
        .map(|v| parse_email_addresses(&v))
        .unwrap_or_default();

    // Parse In-Reply-To
    let in_reply_to = normalize_message_id(parsed.headers.get_first_value("In-Reply-To"));

    // Parse References
    let references = parsed
        .headers
        .get_first_value("References")
        .map(|v| extract_references(&v))
        .unwrap_or_default();

    let normalized_subject = normalize_subject(&subject);

    log::trace!("parsed: {} - {}", message_id, subject);

    Ok(ParsedEmail {
        message_id,
        subject,
        normalized_subject,
        date,
        author_name,
        author_email,
        body,
        to_addrs,
        cc_addrs,
        in_reply_to,
        references,
        patch_type,
        is_patch_only,
        patch_metadata,
    })
}

fn extract_preferred_body(parsed: &ParsedMail) -> String {
    let mut preferred: Option<String> = None;
    let mut stack: Vec<&ParsedMail> = Vec::new();
    stack.push(parsed);

    while let Some(part) = stack.pop() {
        for sub in part.subparts.iter().rev() {
            stack.push(sub);
        }

        let mime = part.ctype.mimetype.to_ascii_lowercase();
        if !mime.contains("/plain") && !mime.contains("/x-patch") {
            continue;
        }

        let body = match part.get_body() {
            Ok(body) => body,
            Err(err) => {
                log::debug!("failed to decode body part ({}): {}", mime, err);
                continue;
            }
        };

        if body.is_empty() {
            continue;
        }

        if preferred.is_none() {
            preferred = Some(body.clone());
            continue;
        }

        if b4_diff_regex().is_match(&body) {
            preferred = Some(body);
        }
    }

    preferred
        .or_else(|| parsed.get_body().ok())
        .unwrap_or_default()
}

fn parse_email_date(
    raw_date: Option<String>,
    message_id: &str,
    subject: &str,
) -> Result<DateTime<Utc>, ParseEmailError> {
    let raw = raw_date.unwrap_or_default();
    if raw.trim().is_empty() {
        log::warn!(
            "email {} ({}) missing Date header, skipping",
            message_id,
            subject
        );
        return Err(ParseEmailError::MissingDate {
            message_id: message_id.to_string(),
        });
    }

    match dateparser::parse(&raw) {
        Ok(dt) => {
            let utc = dt.with_timezone(&Utc);
            let now = Utc::now();
            if utc > now + MAX_FUTURE_SKEW {
                log::warn!(
                    "email {} ({}) has future date `{}` (> {} hours ahead), skipping",
                    message_id,
                    subject,
                    raw,
                    MAX_FUTURE_SKEW.num_hours()
                );
                Err(ParseEmailError::FutureDate {
                    message_id: message_id.to_string(),
                    raw,
                })
            } else {
                Ok(utc)
            }
        }
        Err(source) => {
            log::warn!(
                "email {} ({}) has invalid date `{}`, skipping: {}",
                message_id,
                subject,
                raw,
                source
            );
            Err(ParseEmailError::InvalidDate {
                message_id: message_id.to_string(),
                raw,
                error: source.to_string(),
            })
        }
    }
}

const PATCH_TRAILER_PREFIXES: &[&str] = &[
    "signed-off-by:",
    "co-developed-by:",
    "acknowledged-by:",
    "acked-by:",
    "reviewed-by:",
    "tested-by:",
    "reported-by:",
    "suggested-by:",
    "fixes:",
    "link:",
    "cc:",
    "changelog:",
    "changes in v",
    "changes since v",
    "base-commit:",
    "supersedes:",
    "requires:",
    "dependencies:",
    "depends-on:",
    "note:",
    "notes:",
];

struct InlinePatchAnalysis {
    metadata: PatchMetadata,
    is_patch_only: bool,
}

fn analyze_patch(parsed: &ParsedMail, body: &str) -> (PatchType, bool, Option<PatchMetadata>) {
    if body.is_empty() {
        if has_patch_attachment(parsed) {
            let has_meaningful_text = false;
            return (PatchType::Attachment, has_meaningful_text, None);
        }
        return (PatchType::None, false, None);
    }

    let has_inline_diff = b4_diff_regex().is_match(body);
    if has_inline_diff {
        if let Some(analysis) = analyze_inline_patch(body) {
            return (
                PatchType::Inline,
                analysis.is_patch_only,
                Some(analysis.metadata),
            );
        }
        return (PatchType::Inline, false, None);
    }

    if has_patch_attachment(parsed) {
        let has_meaningful_text = body.lines().any(|line| !line.trim().is_empty());
        let has_quote = body.lines().any(|line| line.trim_start().starts_with('>'));
        let is_patch_only = !has_meaningful_text && !has_quote;
        return (PatchType::Attachment, is_patch_only, None);
    }

    (PatchType::None, false, None)
}

fn analyze_inline_patch(body: &str) -> Option<InlinePatchAnalysis> {
    if body.is_empty() {
        return None;
    }

    let lines: Vec<&str> = body.lines().collect();
    if lines.is_empty() {
        return None;
    }

    let has_diffstat_hint = b4_diffstat_regex().is_match(body);

    let mut diff_starts: Vec<usize> = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        if line.starts_with("diff --git ") || line.starts_with("Index: ") {
            diff_starts.push(idx);
        }
    }

    if diff_starts.is_empty() {
        for (idx, line) in lines.iter().enumerate() {
            if line.starts_with("--- ") {
                if let Some(next) = lines.get(idx + 1) {
                    if next.starts_with("+++ ") {
                        diff_starts.push(idx);
                    }
                }
            }
        }
    }

    if diff_starts.is_empty() {
        return None;
    }

    diff_starts.sort_unstable();
    diff_starts.dedup();

    let mut diff_sections: Vec<PatchSection> = Vec::new();
    for (pos, start) in diff_starts.iter().enumerate() {
        let end = diff_starts.get(pos + 1).copied().unwrap_or(lines.len());
        if *start >= end {
            continue;
        }
        diff_sections.push(PatchSection {
            start_line: *start,
            end_line: end.saturating_sub(1),
        });
    }

    for section in &mut diff_sections {
        let mut last_diff_line = section.start_line;
        for idx in section.start_line..=section.end_line {
            if let Some(line) = lines.get(idx) {
                if is_diff_content_line(line) {
                    last_diff_line = idx;
                }
            }
        }
        section.end_line = last_diff_line;
    }

    diff_sections.retain(|section| {
        section.start_line <= section.end_line
            && lines
                .get(section.start_line)
                .map(|line| is_diff_content_line(line) || line.starts_with("diff --git "))
                .unwrap_or(false)
    });

    if diff_sections.is_empty() {
        return None;
    }

    let separator_line = lines
        .iter()
        .position(|line| line.trim() == "---")
        .filter(|idx| *idx < diff_sections[0].start_line);

    let diffstat_section = if has_diffstat_hint {
        separator_line.and_then(|sep| {
            if sep + 1 >= diff_sections[0].start_line {
                return None;
            }

            let mut start = sep + 1;
            while start < diff_sections[0].start_line && lines[start].trim().is_empty() {
                start += 1;
            }

            if start >= diff_sections[0].start_line {
                return None;
            }

            let mut end = diff_sections[0].start_line.saturating_sub(1);
            while end > start && lines[end].trim().is_empty() {
                end = end.saturating_sub(1);
            }

            if start > end {
                None
            } else {
                Some(PatchSection {
                    start_line: start,
                    end_line: end,
                })
            }
        })
    } else {
        None
    };

    let trailer_boundary = separator_line.unwrap_or(diff_sections[0].start_line);
    let (commit_trailer_section, trailer_count) =
        find_commit_trailer_section(&lines, trailer_boundary);
    let footer_section = find_footer_section(&lines);

    let mut trailer_sections = Vec::new();
    if let Some(section) = commit_trailer_section.clone() {
        trailer_sections.push(section);
    }
    if let Some(section) = footer_section.clone() {
        trailer_sections.push(section);
    }

    if let Some(footer) = &footer_section {
        let start = footer.start_line;
        for section in &mut diff_sections {
            if section.start_line >= start {
                section.end_line = start.saturating_sub(1);
            } else if section.end_line >= start {
                section.end_line = section.end_line.min(start.saturating_sub(1));
            }
        }
        diff_sections.retain(|section| section.start_line <= section.end_line);
        if diff_sections.is_empty() {
            return None;
        }
    }

    let has_quote = lines.iter().any(|line| line.trim_start().starts_with('>'));
    let has_reply_header = lines.iter().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("On ") && trimmed.to_lowercase().contains(" wrote:")
    });

    let last_diff_end = diff_sections
        .last()
        .map(|section| section.end_line)
        .unwrap_or(0);

    let footer_start = footer_section.as_ref().map(|section| section.start_line);

    let has_tail_text = lines
        .iter()
        .enumerate()
        .skip(last_diff_end.saturating_add(1))
        .any(|(idx, line)| {
            if line.trim().is_empty() {
                return false;
            }
            if let Some(start) = footer_start {
                if idx >= start {
                    return false;
                }
            }
            true
        });

    let metadata = PatchMetadata {
        diff_sections,
        diffstat_section,
        trailer_sections,
        separator_line,
        trailer_count,
    };

    let is_patch_only = !has_quote && !has_reply_header && !has_tail_text;

    Some(InlinePatchAnalysis {
        metadata,
        is_patch_only,
    })
}

fn find_commit_trailer_section(lines: &[&str], boundary: usize) -> (Option<PatchSection>, usize) {
    if lines.is_empty() || boundary == 0 {
        return (None, 0);
    }

    let mut idx = boundary;
    let mut start: Option<usize> = None;
    let mut trailer_count = 0;

    while idx > 0 {
        let line = lines[idx - 1];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            idx -= 1;
            continue;
        }

        let lower = trimmed.to_ascii_lowercase();
        if PATCH_TRAILER_PREFIXES
            .iter()
            .any(|prefix| lower.starts_with(prefix))
        {
            start = Some(idx - 1);
            trailer_count += 1;
            idx -= 1;
            continue;
        }

        break;
    }

    if let Some(start_idx) = start {
        return (
            Some(PatchSection {
                start_line: start_idx,
                end_line: boundary.saturating_sub(1),
            }),
            trailer_count,
        );
    }

    (None, trailer_count)
}

fn find_footer_section(lines: &[&str]) -> Option<PatchSection> {
    if lines.is_empty() {
        return None;
    }

    lines
        .iter()
        .enumerate()
        .find(|(_, line)| {
            let trimmed = line.trim();
            trimmed == "--" || trimmed == "-- "
        })
        .map(|(idx, _)| PatchSection {
            start_line: idx,
            end_line: lines.len().saturating_sub(1),
        })
}

fn is_diff_content_line(line: &str) -> bool {
    let trimmed = line.trim_start_matches('\t');
    trimmed.starts_with("diff --git ")
        || trimmed.starts_with("index ")
        || trimmed.starts_with("--- ")
        || trimmed.starts_with("+++ ")
        || trimmed.starts_with("@@")
        || trimmed.starts_with('+')
        || (trimmed.starts_with('-') && !trimmed.starts_with("--"))
        || trimmed.starts_with(' ')
        || trimmed.starts_with("\\ No newline at end of file")
        || trimmed.starts_with("new file mode")
        || trimmed.starts_with("deleted file mode")
        || trimmed.starts_with("old mode")
        || trimmed.starts_with("new mode")
        || trimmed.starts_with("rename from")
        || trimmed.starts_with("rename to")
        || trimmed.starts_with("similarity index")
        || trimmed.starts_with("dissimilarity index")
}

fn has_patch_attachment(parsed: &ParsedMail) -> bool {
    fn part_contains_patch(part: &ParsedMail) -> bool {
        if is_patch_attachment(part) {
            return true;
        }
        for sub in &part.subparts {
            if part_contains_patch(sub) {
                return true;
            }
        }
        false
    }

    for part in &parsed.subparts {
        if part_contains_patch(part) {
            return true;
        }
    }

    false
}

fn is_patch_attachment(part: &ParsedMail) -> bool {
    let mime = part.ctype.mimetype.to_ascii_lowercase();
    let is_patch_mime = matches!(
        mime.as_str(),
        "text/x-diff" | "text/x-patch" | "application/x-patch" | "application/x-diff"
    );

    let mut name_is_patch = false;

    for (key, value) in &part.ctype.params {
        if key.eq_ignore_ascii_case("name") {
            let lower = value.to_ascii_lowercase();
            if lower.ends_with(".patch") || lower.ends_with(".diff") {
                name_is_patch = true;
                break;
            }
        }
    }

    let disposition = part
        .get_headers()
        .get_first_value("Content-Disposition")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let has_attachment_disposition = disposition.contains("attachment");
    let filename_patch = disposition.contains(".patch") || disposition.contains(".diff");

    if is_patch_mime || name_is_patch || filename_patch {
        return true;
    }

    // Fallback: treat explicit attachment disposition with patch-like filename.
    has_attachment_disposition && filename_patch
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_email_detects_inline_patch_metadata() {
        let raw = "From: Dev <dev@example.com>\n\
To: list@example.com\n\
Subject: [PATCH] test\n\
Message-ID: <inline-patch@example.com>\n\
Date: Wed, 30 Nov 2022 08:22:42 +0000\n\
Content-Type: text/plain; charset=\"utf-8\"\n\
\n\
Commit message body\n\
Signed-off-by: Dev <dev@example.com>\n\
---\n\
 foo.c | 2 +-\n\
 1 file changed, 1 insertion(+), 1 deletion(-)\n\
diff --git a/foo.c b/foo.c\n\
index 1111111..2222222 100644\n\
--- a/foo.c\n\
+++ b/foo.c\n\
@@ -1,2 +1,2 @@\n\
-old\n\
+new\n\
-- \n\
2.38.1\n";

        let parsed = parse_email(raw.as_bytes()).expect("inline patch parses");
        assert_eq!(parsed.patch_type, PatchType::Inline);
        assert!(parsed.is_patch_only);

        let metadata = parsed.patch_metadata.expect("metadata present");
        assert_eq!(metadata.diff_sections.len(), 1);
        let section = &metadata.diff_sections[0];
        let lines: Vec<&str> = parsed.body.lines().collect();
        assert!(
            lines
                .get(section.start_line)
                .map(|line| line.starts_with("diff --git"))
                .unwrap_or(false)
        );
        assert!(
            lines
                .get(section.end_line)
                .map(|line| line.starts_with('+'))
                .unwrap_or(false)
        );

        assert_eq!(metadata.trailer_count, 1);
        assert_eq!(metadata.trailer_sections.len(), 2);
        let trailer_lines: Vec<&str> = metadata
            .trailer_sections
            .iter()
            .filter_map(|section| lines.get(section.start_line))
            .map(|line| *line)
            .collect();
        assert!(
            trailer_lines
                .iter()
                .any(|line| line.starts_with("Signed-off-by:"))
        );
        assert!(
            trailer_lines
                .iter()
                .any(|line| line.trim_start().starts_with("--"))
        );
    }

    #[test]
    fn test_parse_email_marks_tail_text_as_not_patch_only() {
        let raw = "From: Dev <dev@example.com>\n\
To: list@example.com\n\
Subject: [PATCH] test\n\
Message-ID: <tail-text@example.com>\n\
Date: Wed, 30 Nov 2022 08:22:42 +0000\n\
Content-Type: text/plain; charset=\"utf-8\"\n\
\n\
Commit message body\n\
Signed-off-by: Dev <dev@example.com>\n\
---\n\
 foo.c | 2 +-\n\
 1 file changed, 1 insertion(+), 1 deletion(-)\n\
diff --git a/foo.c b/foo.c\n\
index 1111111..2222222 100644\n\
--- a/foo.c\n\
+++ b/foo.c\n\
@@ -1,2 +1,2 @@\n\
-old\n\
+new\n\
\n\
Thanks,\n\
Dev\n\
-- \n\
2.38.1\n";

        let parsed = parse_email(raw.as_bytes()).expect("inline patch parses");
        assert_eq!(parsed.patch_type, PatchType::Inline);
        assert!(
            !parsed.is_patch_only,
            "metadata = {:?}",
            parsed.patch_metadata
        );
    }

    #[test]
    fn test_parse_email_cover_letter_not_patch() {
        let raw = "From: Dev <dev@example.com>\n\
To: list@example.com\n\
Subject: [PATCH 0/1] cover letter\n\
Message-ID: <cover-letter@example.com>\n\
Date: Wed, 30 Nov 2022 08:22:42 +0000\n\
Content-Type: text/plain; charset=\"utf-8\"\n\
\n\
Hi all,\n\
\n\
This is a cover letter without an inline diff.\n\
There should be no patch detected here.\n\
-- \n\
Cover Letter\n";

        let parsed = parse_email(raw.as_bytes()).expect("cover letter parses");
        assert_eq!(parsed.patch_type, PatchType::None);
        assert!(!parsed.is_patch_only);
        assert!(parsed.patch_metadata.is_none());
    }

    #[test]
    fn test_sanitize_text() {
        assert_eq!(sanitize_text("hello\0world"), "helloworld");
        assert_eq!(sanitize_text("  test  "), "test");
    }

    #[test]
    fn test_normalize_message_id() {
        assert_eq!(
            normalize_message_id(Some("<test@example.com>".to_string())),
            Some("test@example.com".to_string())
        );
        assert_eq!(normalize_message_id(Some("".to_string())), None);
        assert_eq!(normalize_message_id(None), None);
    }

    #[test]
    fn test_extract_references() {
        let refs = extract_references("<msg1@example.com> <msg2@example.com>");
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0], "msg1@example.com");
        assert_eq!(refs[1], "msg2@example.com");
    }

    #[test]
    fn test_normalize_subject() {
        assert_eq!(
            normalize_subject("Re: [PATCH] Fix memory leak"),
            "fix memory leak"
        );
        assert_eq!(
            normalize_subject("[PATCH v2 1/3] Add new feature"),
            "add new feature"
        );
        assert_eq!(normalize_subject("Re: Fwd: [RFC PATCH] Test"), "test");
        assert_eq!(
            normalize_subject("Re: Re: [PATCH v3] Important fix"),
            "important fix"
        );
    }

    #[test]
    fn test_parse_email_rejects_missing_date() {
        let raw = concat!(
            "Message-ID: <missing-date@test>\r\n",
            "Subject: Missing Date\r\n",
            "From: Tester <tester@example.com>\r\n",
            "\r\n",
            "Body\r\n"
        );

        let err = parse_email(raw.as_bytes()).unwrap_err();
        assert!(matches!(err, ParseEmailError::MissingDate { .. }));
    }

    #[test]
    fn test_parse_email_rejects_invalid_date() {
        let raw = concat!(
            "Message-ID: <invalid-date@test>\r\n",
            "Subject: Invalid Date\r\n",
            "From: Tester <tester@example.com>\r\n",
            "Date: not-a-real-date\r\n",
            "\r\n",
            "Body\r\n"
        );

        let err = parse_email(raw.as_bytes()).unwrap_err();
        assert!(matches!(err, ParseEmailError::InvalidDate { .. }));
    }

    #[test]
    fn test_parse_email_rejects_future_date() {
        let future = Utc::now() + Duration::days(10);
        let raw = format!(
            "Message-ID: <future-date@test>\r\nSubject: Future Date\r\nFrom: Tester <tester@example.com>\r\nDate: {}\r\n\r\nBody\r\n",
            future.to_rfc2822()
        );

        let err = parse_email(raw.as_bytes()).unwrap_err();
        assert!(matches!(err, ParseEmailError::FutureDate { .. }));
    }
}
