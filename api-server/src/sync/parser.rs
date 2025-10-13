use chrono::{DateTime, Utc};
use mailparse::{parse_mail, MailHeaderMap};

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

/// Clean and normalize message IDs
fn clean_message_id(msg_id: Option<String>) -> Option<String> {
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

/// Normalize email subject for threading comparison
/// Removes common prefixes like Re:, Fwd:, [PATCH], version numbers, etc.
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

/// Parse an email from raw bytes
pub fn parse_email(blob_data: &[u8]) -> Result<ParsedEmail, Box<dyn std::error::Error>> {
    let parsed = parse_mail(blob_data).map_err(|e| {
        log::debug!("failed to parse MIME: {}", e);
        e
    })?;

    // Extract Message-ID (required)
    let message_id = clean_message_id(
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
    let in_reply_to = clean_message_id(
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
    fn test_clean_message_id() {
        assert_eq!(
            clean_message_id(Some("<test@example.com>".to_string())),
            Some("test@example.com".to_string())
        );
        assert_eq!(clean_message_id(Some("".to_string())), None);
        assert_eq!(clean_message_id(None), None);
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
