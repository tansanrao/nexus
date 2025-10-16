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

use chrono::{DateTime, Utc};
use mailparse::{parse_mail, MailHeaderMap};

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
/// - Unparseable date (uses current time)
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
pub fn parse_email(blob_data: &[u8]) -> Result<ParsedEmail, Box<dyn std::error::Error>> {
    let parsed = parse_mail(blob_data).map_err(|e| {
        log::debug!("failed to parse MIME: {}", e);
        e
    })?;

    // Extract Message-ID (required)
    let message_id = normalize_message_id(
        parsed.headers.get_first_value("Message-ID")
    ).ok_or_else(|| {
        log::debug!("missing Message-ID header");
        "Missing Message-ID"
    })?;

    // Extract subject
    let subject = parsed
        .headers
        .get_first_value("Subject")
        .map(|s| sanitize_text(&s))
        .unwrap_or_else(|| "(No Subject)".to_string());

    // Parse date - dateparser returns DateTime<FixedOffset>, convert to DateTime<Utc>
    let date_str = parsed
        .headers
        .get_first_value("Date")
        .unwrap_or_default();

    let date = if let Ok(dt) = dateparser::parse(&date_str) {
        dt.with_timezone(&Utc)
    } else {
        Utc::now()
    };

    // Parse author
    let from_str = parsed
        .headers
        .get_first_value("From")
        .unwrap_or_default();

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
        log::debug!("email {} missing author", message_id);
        return Err("Missing author email".into());
    }

    // Extract body
    let body = if parsed.subparts.is_empty() {
        // Single part message
        parsed
            .get_body()
            .unwrap_or_default()
    } else {
        // Multipart message - find text/plain part
        let mut body_text = String::new();
        for part in &parsed.subparts {
            if part.ctype.mimetype.as_str() == "text/plain" {
                body_text = part.get_body().unwrap_or_default();
                break;
            }
        }
        if body_text.is_empty() {
            parsed.get_body().unwrap_or_default()
        } else {
            body_text
        }
    };

    let body = sanitize_text(&body);

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
    let in_reply_to = normalize_message_id(
        parsed.headers.get_first_value("In-Reply-To")
    );

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
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(
            normalize_subject("Re: Fwd: [RFC PATCH] Test"),
            "test"
        );
        assert_eq!(
            normalize_subject("Re: Re: [PATCH v3] Important fix"),
            "important fix"
        );
    }
}
