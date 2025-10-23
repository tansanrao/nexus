//! Patch series detection and linking
//!
//! Linux kernel mailing list emails often come in "patch series" with subjects like:
//! - [PATCH 0/5] Cover letter describing the series
//! - [PATCH 1/5] First actual patch
//! - [PATCH 2/5] Second patch
//! - etc.
//!
//! This module handles detection and proper threading of such series.

use regex::Regex;
use std::sync::OnceLock;

/// Lazy-initialized regex for matching patch series patterns
static PATCH_REGEX: OnceLock<Regex> = OnceLock::new();

/// Get the compiled patch series regex
///
/// Pattern matches:
/// - [PATCH 2/5] - basic patch series
/// - [PATCH v2 3/10] - versioned series
/// - [RFC PATCH 1/3] - RFC patches
/// - [PATCH v3 0/5] - versioned cover letter
///
/// The regex captures:
/// 1. Version number (optional) - e.g., "2" from "v2"
/// 2. Patch number - e.g., "3" from "3/10"
/// 3. Total patches - e.g., "10" from "3/10"
fn get_patch_regex() -> &'static Regex {
    PATCH_REGEX.get_or_init(|| {
        Regex::new(r"\[.*?PATCH\s*(?:v(\d+))?\s*(\d+)/(\d+)\s*\]")
            .expect("Invalid patch series regex")
    })
}

/// Extract patch series information from an email subject
///
/// ## Returns
///
/// `Some((version, patch_number, total_patches))` if the subject contains a patch series marker.
///
/// ## Examples
///
/// ```rust
/// use api_server::threading::patch_series::extract_patch_series_info;
///
/// assert_eq!(
///     extract_patch_series_info("[PATCH 2/5] Fix memory leak"),
///     Some(("".to_string(), 2, 5))
/// );
/// assert_eq!(
///     extract_patch_series_info("[PATCH v2 3/10] Add new feature"),
///     Some(("v2".to_string(), 3, 10))
/// );
/// assert_eq!(
///     extract_patch_series_info("[RFC PATCH v3 0/5] Cover letter"),
///     Some(("v3".to_string(), 0, 5))
/// );
/// assert_eq!(
///     extract_patch_series_info("Regular email subject"),
///     None
/// );
/// ```
pub fn extract_patch_series_info(subject: &str) -> Option<(String, i32, i32)> {
    let re = get_patch_regex();

    re.captures(subject).and_then(|caps| {
        // Extract version (e.g., "v2" becomes "v2", or empty string if no version)
        let version = caps
            .get(1)
            .map(|m| format!("v{}", m.as_str()))
            .unwrap_or_default();

        // Extract patch number
        let patch_num = caps.get(2)?.as_str().parse::<i32>().ok()?;

        // Extract total patches
        let total = caps.get(3)?.as_str().parse::<i32>().ok()?;

        Some((version, patch_num, total))
    })
}

// NOTE: This module now only extracts patch series metadata.
// We no longer use patch series information for threading - that is handled
// purely by email headers (References, In-Reply-To) in the JWZ algorithm.
//
// The link_patch_series function has been removed as it was creating false
// threading relationships that don't match public-inbox behavior.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_basic_patch() {
        let result = extract_patch_series_info("[PATCH 2/5] Fix memory leak");
        assert_eq!(result, Some(("".to_string(), 2, 5)));
    }

    #[test]
    fn test_extract_versioned_patch() {
        let result = extract_patch_series_info("[PATCH v2 3/10] Add new feature");
        assert_eq!(result, Some(("v2".to_string(), 3, 10)));
    }

    #[test]
    fn test_extract_cover_letter() {
        let result = extract_patch_series_info("[PATCH v3 0/5] Cover letter");
        assert_eq!(result, Some(("v3".to_string(), 0, 5)));
    }

    #[test]
    fn test_extract_rfc_patch() {
        let result = extract_patch_series_info("[RFC PATCH 1/3] Experimental feature");
        assert_eq!(result, Some(("".to_string(), 1, 3)));
    }

    #[test]
    fn test_extract_no_patch() {
        let result = extract_patch_series_info("Regular email subject");
        assert_eq!(result, None);
    }
}
