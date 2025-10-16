use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;

/// Represents a repository entry in the grokmirror manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestRepo {
    pub description: Option<String>,
    pub reference: Option<String>,
    pub modified: Option<i64>,
    pub fingerprint: Option<String>,
    pub alternates: Option<Vec<String>>,
}

/// The full grokmirror manifest structure
/// Keys are repository paths like "/lkml/0", values are repo metadata
pub type Manifest = HashMap<String, ManifestRepo>;

/// Represents a mailing list with all its repository shards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailingListFromManifest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub repos: Vec<RepoShard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoShard {
    pub url: String,
    pub order: i32,
}

/// Fetch and decompress the grokmirror manifest from lore.kernel.org
pub async fn fetch_manifest() -> Result<Manifest, Box<dyn std::error::Error>> {
    log::info!("fetching manifest from https://lore.kernel.org/manifest.js.gz");

    let response = reqwest::get("https://lore.kernel.org/manifest.js.gz")
        .await?
        .error_for_status()?;

    let compressed_bytes = response.bytes().await?;

    log::debug!("decompressing manifest ({} bytes)", compressed_bytes.len());

    // Decompress gzip
    let mut decoder = GzDecoder::new(&compressed_bytes[..]);
    let mut json_string = String::new();
    decoder.read_to_string(&mut json_string)?;

    log::debug!("parsing manifest JSON ({} bytes)", json_string.len());

    // Parse JSON
    let manifest: Manifest = serde_json::from_str(&json_string)?;

    log::info!("manifest loaded: {} repositories", manifest.len());

    Ok(manifest)
}

/// Parse the manifest into a list of mailing lists with their repository shards
/// Groups repositories by mailing list slug (e.g., /lkml/0, /lkml/1 -> lkml list with 2 repos)
pub fn parse_manifest(manifest: &Manifest) -> Vec<MailingListFromManifest> {
    let mut mailing_lists: HashMap<String, MailingListFromManifest> = HashMap::new();

    for (path, repo) in manifest.iter() {
        // Parse path: /slug/epoch or /slug/git/epoch.git
        // Examples: /lkml/0, /bpf/git/0.git
        let path = path.trim_start_matches('/');
        let parts: Vec<&str> = path.split('/').collect();

        if parts.is_empty() {
            continue;
        }

        let slug = parts[0].to_string();

        // Parse epoch/order from path
        // Handle both /slug/epoch and /slug/git/epoch.git formats
        let order = if parts.len() >= 2 {
            let epoch_str = parts[parts.len() - 1].trim_end_matches(".git");
            epoch_str.parse::<i32>().unwrap_or(0)
        } else {
            0
        };

        // Build the full lore.kernel.org URL
        // Format: https://lore.kernel.org/{slug}/git/{epoch}.git
        let url = format!("https://lore.kernel.org/{}/git/{}.git", slug, order);

        // Get or create mailing list entry
        let list = mailing_lists.entry(slug.clone()).or_insert_with(|| {
            // Extract name from description or use slug
            let name = repo
                .description
                .as_ref()
                .and_then(|d| {
                    // Remove epoch suffix like "[epoch 0]" from description
                    let name = d.split('[').next()?.trim();
                    if name.is_empty() {
                        None
                    } else {
                        Some(name.to_string())
                    }
                })
                .unwrap_or_else(|| slug.replace('-', " "));

            MailingListFromManifest {
                name,
                slug: slug.clone(),
                description: repo.description.clone(),
                repos: Vec::new(),
            }
        });

        // Add repository shard
        list.repos.push(RepoShard { url, order });
    }

    // Sort repos within each list by order
    for list in mailing_lists.values_mut() {
        list.repos.sort_by_key(|r| r.order);
    }

    // Convert to sorted vector
    let mut result: Vec<MailingListFromManifest> = mailing_lists.into_values().collect();
    result.sort_by(|a, b| a.slug.cmp(&b.slug));

    log::info!("parsed manifest: {} mailing lists", result.len());

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest_basic() {
        let mut manifest = Manifest::new();

        // Single-epoch list
        manifest.insert(
            "/bpf/0".to_string(),
            ManifestRepo {
                description: Some("BPF development [epoch 0]".to_string()),
                reference: None,
                modified: None,
                fingerprint: None,
                alternates: None,
            },
        );

        // Multi-epoch list
        manifest.insert(
            "/lkml/0".to_string(),
            ManifestRepo {
                description: Some("Linux Kernel Mailing List [epoch 0]".to_string()),
                reference: None,
                modified: None,
                fingerprint: None,
                alternates: None,
            },
        );
        manifest.insert(
            "/lkml/1".to_string(),
            ManifestRepo {
                description: Some("Linux Kernel Mailing List [epoch 1]".to_string()),
                reference: None,
                modified: None,
                fingerprint: None,
                alternates: None,
            },
        );

        let lists = parse_manifest(&manifest);

        assert_eq!(lists.len(), 2);

        let bpf = lists.iter().find(|l| l.slug == "bpf").unwrap();
        assert_eq!(bpf.repos.len(), 1);
        assert_eq!(bpf.repos[0].order, 0);
        assert!(bpf.repos[0].url.contains("bpf"));

        let lkml = lists.iter().find(|l| l.slug == "lkml").unwrap();
        assert_eq!(lkml.repos.len(), 2);
        assert_eq!(lkml.repos[0].order, 0);
        assert_eq!(lkml.repos[1].order, 1);
    }

    #[test]
    fn test_parse_manifest_git_format() {
        let mut manifest = Manifest::new();

        // Git format path: /slug/git/epoch.git
        manifest.insert(
            "/netdev/git/0.git".to_string(),
            ManifestRepo {
                description: Some("Network device development".to_string()),
                reference: None,
                modified: None,
                fingerprint: None,
                alternates: None,
            },
        );

        let lists = parse_manifest(&manifest);

        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].slug, "netdev");
        assert_eq!(lists[0].repos[0].order, 0);
        assert!(lists[0].repos[0].url.contains("/git/0.git"));
    }
}
