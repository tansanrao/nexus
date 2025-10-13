use gix::ObjectId;
use std::path::PathBuf;
use std::env;

/// Configuration for a single repository
#[derive(Debug, Clone)]
pub struct RepoConfig {
    #[allow(dead_code)]
    pub url: String,
    pub order: i32,
}

/// Configuration for syncing a mailing list with multiple repositories
#[derive(Debug, Clone)]
pub struct MailingListSyncConfig {
    #[allow(dead_code)]
    pub list_id: i32,
    pub slug: String,
    pub repos: Vec<RepoConfig>,
    pub mirror_base_path: PathBuf,
}

impl MailingListSyncConfig {
    /// Create config from environment variables and database data
    pub fn new(list_id: i32, slug: String, repos: Vec<RepoConfig>) -> Self {
        let mirror_base = env::var("MIRROR_BASE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let project_root = env::var("PROJECT_ROOT")
                    .unwrap_or_else(|_| ".".to_string());
                PathBuf::from(project_root).join("mirrors")
            });

        Self {
            list_id,
            slug,
            repos,
            mirror_base_path: mirror_base,
        }
    }

    /// Get the mirror path for a specific repository using grokmirror's structure
    /// Path structure: {mirror_base}/{slug}/git/{epoch}.git
    /// Example: /app/mirrors/bpf/git/0.git
    pub fn get_repo_mirror_path(&self, repo_order: i32) -> PathBuf {
        self.mirror_base_path
            .join(&self.slug)
            .join("git")
            .join(format!("{}.git", repo_order))
    }
}

/// Manages git operations for mailing list repositories
pub struct GitManager {
    pub config: MailingListSyncConfig,
}

#[derive(Debug)]
pub enum GitError {
    Gix(gix::open::Error),
    GixClone(gix::clone::Error),
    GixFetch(gix::clone::fetch::Error),
    Io(std::io::Error),
    Other(String),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::Gix(e) => write!(f, "Git error: {}", e),
            GitError::GixClone(e) => write!(f, "Clone error: {}", e),
            GitError::GixFetch(e) => write!(f, "Fetch error: {}", e),
            GitError::Io(e) => write!(f, "IO error: {}", e),
            GitError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for GitError {}

impl From<gix::open::Error> for GitError {
    fn from(e: gix::open::Error) -> Self {
        GitError::Gix(e)
    }
}

impl From<gix::clone::Error> for GitError {
    fn from(e: gix::clone::Error) -> Self {
        GitError::GixClone(e)
    }
}

impl From<gix::clone::fetch::Error> for GitError {
    fn from(e: gix::clone::fetch::Error) -> Self {
        GitError::GixFetch(e)
    }
}

impl From<std::io::Error> for GitError {
    fn from(e: std::io::Error) -> Self {
        GitError::Io(e)
    }
}

impl GitManager {
    pub fn new(config: MailingListSyncConfig) -> Self {
        Self { config }
    }

    /// Check if a specific repository mirror exists and is valid
    /// Returns an error with helpful message if mirror doesn't exist
    fn validate_mirror(&self, mirror_path: &PathBuf) -> Result<(), GitError> {
        if !mirror_path.exists() {
            return Err(GitError::Other(format!(
                "Mirror not found at {:?}. Please ensure grokmirror is running and has completed at least one sync. \
                See grokmirror/README.md for setup instructions.",
                mirror_path
            )));
        }

        if gix::open(mirror_path).is_err() {
            return Err(GitError::Other(format!(
                "Invalid git repository at {:?}. The mirror may be corrupted. \
                Try running 'grok-fsck -c grokmirror/grokmirror.conf' to check repository health.",
                mirror_path
            )));
        }

        Ok(())
    }

    /// Validate that all repository mirrors exist for this mailing list
    pub fn validate_all_mirrors(&self) -> Result<(), GitError> {
        for repo_config in &self.config.repos {
            let mirror_path = self.config.get_repo_mirror_path(repo_config.order);
            self.validate_mirror(&mirror_path)?;
        }
        Ok(())
    }

    /// Get commits for a specific epoch (repo_order)
    /// Used by the worker to discover commits sequentially per epoch
    pub fn get_commits_for_epoch(
        &self,
        repo_order: i32,
        since: Option<&str>
    ) -> Result<Vec<(String, String, i32)>, GitError> {
        let mirror_path = self.config.get_repo_mirror_path(repo_order);
        self.get_email_commits_from_repo_since(&mirror_path, repo_order, since)
    }

    /// Get email commits from a specific repository, optionally filtering by last indexed commit
    /// Returns (commit_hash, path, repo_order) in chronological order (oldest to newest)
    /// If since_commit is Some, only returns commits newer than the specified commit
    fn get_email_commits_from_repo_since(
        &self,
        mirror_path: &PathBuf,
        repo_order: i32,
        since_commit: Option<&str>,
    ) -> Result<Vec<(String, String, i32)>, GitError> {
        let repo = gix::open(mirror_path)?;
        let mut commits = Vec::new();

        // Iterate through all references
        let references = repo.references()
            .map_err(|e| GitError::Other(format!("Failed to get references: {}", e)))?;

        for reference in references.all()
            .map_err(|e| GitError::Other(format!("Failed to iterate references: {}", e)))?
        {
            let reference = reference
                .map_err(|e| GitError::Other(format!("Failed to get reference: {}", e)))?;

            // Only process branch references
            if !reference.name().category().map(|c| c == gix::refs::Category::LocalBranch).unwrap_or(false) {
                continue;
            }

            if let Some(target) = reference.target().try_id() {
                // Collect commits for this branch in a temporary vector
                // We'll reverse it later to get chronological order
                let mut branch_commits = Vec::new();

                // Walk the commit history from HEAD backwards
                let commit = repo
                    .find_object(target)
                    .map_err(|e| GitError::Other(format!("Failed to find object: {}", e)))?
                    .try_into_commit()
                    .map_err(|e| GitError::Other(format!("Failed to convert to commit: {}", e)))?;

                let commit_hash = commit.id.to_hex().to_string();

                // For incremental sync: if HEAD is the since_commit, this branch has no new commits
                if let Some(since) = since_commit {
                    if commit_hash == since {
                        continue; // Skip this branch entirely
                    }
                }

                // Get the tree for this commit
                let tree = commit
                    .tree()
                    .map_err(|e| GitError::Other(format!("Failed to get tree: {}", e)))?;

                // Check if 'm' file exists in the tree (public-inbox v2 format)
                let has_email = tree.iter().any(|entry| {
                    entry.map(|e| e.filename() == "m" && e.mode().is_blob()).unwrap_or(false)
                });

                if has_email {
                    branch_commits.push((commit_hash.clone(), "m".to_string(), repo_order));
                }

                // Walk commit ancestors
                let ancestors = commit
                    .ancestors()
                    .all()
                    .map_err(|e| GitError::Other(format!("Failed to create ancestor iterator: {}", e)))?;

                for info in ancestors {
                    let info = info
                        .map_err(|e| GitError::Other(format!("Failed to get ancestor info: {}", e)))?;

                    let ancestor_hash = info.id.to_hex().to_string();

                    // Check if we've reached the since_commit
                    if let Some(since) = since_commit {
                        if ancestor_hash == since {
                            // We've reached the last indexed commit, stop walking this branch
                            break;
                        }
                    }

                    let ancestor_commit = repo
                        .find_object(info.id)
                        .map_err(|e| GitError::Other(format!("Failed to find ancestor object: {}", e)))?
                        .try_into_commit()
                        .map_err(|e| GitError::Other(format!("Failed to convert ancestor to commit: {}", e)))?;

                    let ancestor_tree = ancestor_commit
                        .tree()
                        .map_err(|e| GitError::Other(format!("Failed to get ancestor tree: {}", e)))?;

                    // In public-inbox v2 format, emails are stored in 'm' files
                    let has_email = ancestor_tree.iter().any(|entry| {
                        entry.map(|e| e.filename() == "m" && e.mode().is_blob()).unwrap_or(false)
                    });

                    if has_email {
                        branch_commits.push((ancestor_hash.clone(), "m".to_string(), repo_order));
                    }
                }

                // Reverse to get chronological order (oldest to newest)
                branch_commits.reverse();
                commits.extend(branch_commits);
            }
        }

        Ok(commits)
    }

    /// Get the blob data for a specific commit and path from a specific repository
    pub fn get_blob_data(&self, commit_hash: &str, path: &str, repo_order: i32) -> Result<Vec<u8>, GitError> {
        let mirror_path = self.config.get_repo_mirror_path(repo_order);
        let repo = gix::open(&mirror_path)?;

        // Parse the commit hash
        let oid = ObjectId::from_hex(commit_hash.as_bytes())
            .map_err(|e| GitError::Other(format!("Invalid commit hash: {}", e)))?;

        // Find the commit
        let commit = repo
            .find_object(oid)
            .map_err(|e| GitError::Other(format!("Failed to find commit: {}", e)))?
            .try_into_commit()
            .map_err(|e| GitError::Other(format!("Object is not a commit: {}", e)))?;

        // Get the tree
        let tree = commit
            .tree()
            .map_err(|e| GitError::Other(format!("Failed to get tree: {}", e)))?;

        // Find the entry in the tree by iterating
        let mut found_entry = None;
        for entry in tree.iter() {
            let entry = entry
                .map_err(|e| GitError::Other(format!("Failed to iterate tree: {}", e)))?;
            if entry.filename() == path {
                found_entry = Some(entry);
                break;
            }
        }

        let entry = found_entry
            .ok_or_else(|| GitError::Other(format!("Path '{}' not found in tree", path)))?;

        // Get the blob
        let blob = entry
            .object()
            .map_err(|e| GitError::Other(format!("Failed to get object: {}", e)))?
            .try_into_blob()
            .map_err(|e| GitError::Other(format!("Object is not a blob: {}", e)))?;

        Ok(blob.data.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mailing_list_sync_config() {
        let repos = vec![
            RepoConfig {
                url: "https://lore.kernel.org/bpf/0".to_string(),
                order: 0,
            },
            RepoConfig {
                url: "https://lore.kernel.org/bpf/1".to_string(),
                order: 1,
            },
        ];

        let config = MailingListSyncConfig::new(1, "bpf".to_string(), repos);

        assert_eq!(config.list_id, 1);
        assert_eq!(config.slug, "bpf");
        assert_eq!(config.repos.len(), 2);

        let path0 = config.get_repo_mirror_path(0);
        let path1 = config.get_repo_mirror_path(1);

        // Check grokmirror path structure: {mirror_base}/{slug}/git/{epoch}.git
        assert!(path0.to_string_lossy().contains("bpf"));
        assert!(path0.to_string_lossy().contains("/git/"));
        assert!(path0.to_string_lossy().ends_with("0.git"));

        assert!(path1.to_string_lossy().contains("bpf"));
        assert!(path1.to_string_lossy().contains("/git/"));
        assert!(path1.to_string_lossy().ends_with("1.git"));
    }
}
