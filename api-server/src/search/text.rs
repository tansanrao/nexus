use crate::models::{PatchMetadata, PatchSection, PatchType};
use crate::sync::parser::ParsedEmail;

/// Build the canonical text used for email embeddings.
///
/// Combines the normalized subject with the conversational body and removes
/// patch content (diffs, diffstats, trailers) so semantic search focuses on the
/// discussion rather than the code changes.
pub fn build_email_embedding_text(email: &ParsedEmail) -> String {
    let mut body = email.body.clone();

    if matches!(email.patch_type, PatchType::Attachment) && email.is_patch_only {
        body.clear();
    } else if let Some(metadata) = &email.patch_metadata {
        body = strip_patch_sections(&body, metadata);
    }

    let body = normalize_whitespace(body.trim());
    let subject = email.subject.trim();

    if body.is_empty() {
        subject.to_string()
    } else if subject.is_empty() {
        body
    } else {
        format!("{subject}\n\n{body}")
    }
}

fn strip_patch_sections(body: &str, metadata: &PatchMetadata) -> String {
    if body.is_empty() {
        return String::new();
    }

    let lines: Vec<&str> = body.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let mut drop_mask = vec![false; lines.len()];

    for section in &metadata.diff_sections {
        mark_section(&mut drop_mask, section);
    }

    if let Some(section) = &metadata.diffstat_section {
        mark_section(&mut drop_mask, section);
    }

    for section in &metadata.trailer_sections {
        mark_section(&mut drop_mask, section);
    }

    let filtered: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter_map(|(idx, line)| (!drop_mask[idx]).then_some(*line))
        .collect();

    filtered.join("\n")
}

fn mark_section(mask: &mut [bool], section: &PatchSection) {
    let start = section.start_line.min(mask.len());
    let end = section.end_line.min(mask.len().saturating_sub(1));
    for idx in start..=end {
        mask[idx] = true;
    }
}

fn normalize_whitespace(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    let mut previous_was_blank = false;

    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            if !previous_was_blank {
                normalized.push('\n');
            }
            previous_was_blank = true;
            continue;
        }

        if previous_was_blank && !normalized.is_empty() && !normalized.ends_with('\n') {
            normalized.push('\n');
        }

        normalized.push_str(trimmed);
        normalized.push('\n');
        previous_was_blank = false;
    }

    // Remove trailing newline inserted by the loop if present.
    if normalized.ends_with('\n') {
        normalized.pop();
    }

    normalized
}
