use std::borrow::Cow;

use crate::models::PatchMetadata;

/// Strip git patch payloads (diffs, diffstat, trailers) from an email body while
/// preserving conversational content.
///
/// The parser records patch metadata that marks exact line ranges for inline
/// diffs. When metadata is present we drop those ranges precisely; otherwise we
/// fall back to diff/trailer heuristics so legacy rows do not contribute large
/// noise blocks to the search index.
pub fn strip_patch_payload<'a>(
    body: &'a str,
    metadata: Option<&PatchMetadata>,
    is_patch_only: bool,
) -> Cow<'a, str> {
    if body.is_empty() || is_patch_only {
        return Cow::Borrowed("");
    }

    let lines: Vec<&str> = body.lines().collect();
    if lines.is_empty() {
        return Cow::Borrowed("");
    }

    let mut drop_mask = vec![false; lines.len()];

    if let Some(meta) = metadata {
        mark_sections(&mut drop_mask, &meta.diff_sections, lines.len());
        if let Some(diffstat) = &meta.diffstat_section {
            mark_section(&mut drop_mask, diffstat, lines.len());
        }
        for trailer in &meta.trailer_sections {
            mark_section(&mut drop_mask, trailer, lines.len());
        }
        if let Some(separator) = meta.separator_line {
            if separator < drop_mask.len() {
                drop_mask[separator] = true;
            }
        }
    } else {
        apply_heuristics(&lines, &mut drop_mask);
    }

    let mut sanitized: Vec<&str> = Vec::with_capacity(lines.len());
    let mut last_was_blank = true;

    for (idx, line) in lines.iter().enumerate() {
        if drop_mask[idx] {
            continue;
        }

        let trimmed = line.trim();
        let is_blank = trimmed.is_empty();

        if is_blank {
            if sanitized.is_empty() || last_was_blank {
                continue;
            }
        }

        sanitized.push(*line);
        last_was_blank = is_blank;
    }

    // Remove trailing blank line left after diff removal.
    while sanitized
        .last()
        .map(|line| line.trim().is_empty())
        .unwrap_or(false)
    {
        sanitized.pop();
    }

    if sanitized.is_empty() {
        Cow::Owned(String::new())
    } else if sanitized.len() == lines.len()
        && sanitized
            .iter()
            .zip(lines.iter())
            .all(|(a, b)| std::ptr::eq(*a, *b))
    {
        Cow::Borrowed(body)
    } else {
        Cow::Owned(sanitized.join("\n"))
    }
}

fn mark_sections(drop_mask: &mut [bool], sections: &[crate::models::PatchSection], total: usize) {
    for section in sections {
        mark_section(drop_mask, section, total);
    }
}

fn mark_section(drop_mask: &mut [bool], section: &crate::models::PatchSection, total: usize) {
    if section.start_line >= total {
        return;
    }
    let end = section.end_line.min(total.saturating_sub(1));
    for idx in section.start_line..=end {
        drop_mask[idx] = true;
    }
}

fn apply_heuristics(lines: &[&str], drop_mask: &mut [bool]) {
    const DIFF_START_PREFIXES: &[&str] =
        &["diff --git ", "Index: ", "+++ ", "--- ", "*** ", "===="];

    const TRAILER_PREFIXES: &[&str] = &[
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

    let mut in_diff = false;

    for (idx, line) in lines.iter().enumerate() {
        let original = *line;
        let trimmed = original.trim_start();
        let lower = trimmed.to_ascii_lowercase();

        if DIFF_START_PREFIXES
            .iter()
            .any(|prefix| trimmed.starts_with(prefix))
        {
            in_diff = true;
            drop_mask[idx] = true;
            continue;
        }

        if in_diff {
            if trimmed.starts_with("@@")
                || trimmed.starts_with("+++")
                || trimmed.starts_with("---")
                || trimmed.starts_with("index ")
            {
                drop_mask[idx] = true;
                continue;
            }

            if original.starts_with('+') || original.starts_with('-') || original.starts_with(' ') {
                drop_mask[idx] = true;
                continue;
            }

            if trimmed.is_empty() {
                drop_mask[idx] = true;
                in_diff = false;
                continue;
            }

            // Exit diff block when we hit a non diff line.
            in_diff = false;
        }

        if lower.is_empty() {
            // Preserve blank lines here; the caller collapses duplicates later.
            continue;
        }

        if TRAILER_PREFIXES
            .iter()
            .any(|prefix| lower.starts_with(prefix))
        {
            drop_mask[idx] = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PatchMetadata, PatchSection};

    fn sample_metadata() -> PatchMetadata {
        PatchMetadata {
            diff_sections: vec![PatchSection {
                start_line: 4,
                end_line: 9,
            }],
            diffstat_section: None,
            trailer_sections: vec![PatchSection {
                start_line: 10,
                end_line: 10,
            }],
            separator_line: Some(4),
            trailer_count: 1,
        }
    }

    #[test]
    fn strip_patch_payload_with_metadata() {
        let body = "Greeting\n\nHere is the patch\n\n---\ndiff --git a/file b/file\n@@ -1,2 +1,2 @@\n-line1\n+line2\n\nSigned-off-by: Dev\n";

        let result = strip_patch_payload(body, Some(&sample_metadata()), false);
        assert_eq!(result, "Greeting\n\nHere is the patch");
    }

    #[test]
    fn strip_patch_payload_patch_only_drops_all() {
        let body = "diff --git a/file b/file\n@@ -1 +1 @@\n-line\n+line";
        let result = strip_patch_payload(body, Some(&sample_metadata()), true);
        assert!(result.is_empty());
    }

    #[test]
    fn strip_patch_payload_heuristics_remove_diff() {
        let body = "Intro text\n\nOn patch\n\ndiff --git a/foo b/foo\nindex 111..222 100644\n--- a/foo\n+++ b/foo\n@@ -1,3 +1,4 @@\n-line1\n+line1 changed\n line2\n\nSigned-off-by: Dev\n\nReply continues";

        let result = strip_patch_payload(body, None, false);
        assert_eq!(result, "Intro text\n\nOn patch\n\nReply continues");
    }
}
