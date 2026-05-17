//! Multi-Repository Support — register, manage, and operate across repos
//!
//! Allows users to register multiple Git repositories with OpenCrust,
//! enabling cross-repo search, bulk operations, and context-aware agent
//! execution across workspaces.
//!
//! ## Architecture
//!
//! ```text
//! MultiRepoManager
//!   ├── repos: HashMap<String, RegisteredRepo>
//!   ├── persistence: JSON file at ~/.config/opencrust/repos.json
//!   └── cross-repo search via parallel git operations
//! ```
//!
//! Each registered repo has a name alias, local path, optional remote URL,
//! tags for categorization, and last-indexed timestamp.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// A registered repository with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredRepo {
    /// User-assigned short name (alias)
    pub name: String,
    /// Absolute local path to the repo root
    pub path: PathBuf,
    /// Optional remote URL (e.g., git@github.com:user/repo.git)
    pub remote: Option<String>,
    /// Current branch (auto-detected)
    pub branch: Option<String>,
    /// User-defined tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    /// When the repo was registered
    pub registered_at: DateTime<Utc>,
    /// When the repo was last indexed for cross-repo search
    pub last_indexed: Option<DateTime<Utc>>,
}

impl RegisteredRepo {
    /// Create a new registration from a path, auto-detecting remote and branch.
    pub fn new(name: String, path: PathBuf, tags: Vec<String>) -> Result<Self, String> {
        let canonical = path
            .canonicalize()
            .map_err(|e| format!("invalid path '{}': {}", path.display(), e))?;

        if !canonical.join(".git").exists() && !canonical.join(".git").is_dir() {
            return Err(format!("'{}' is not a git repository", canonical.display()));
        }

        let remote = detect_remote(&canonical);
        let branch = detect_branch(&canonical);

        Ok(Self {
            name,
            path: canonical,
            remote,
            branch,
            tags,
            registered_at: Utc::now(),
            last_indexed: None,
        })
    }

    /// Refresh branch and remote information from the filesystem.
    pub fn refresh(&mut self) {
        self.branch = detect_branch(&self.path);
        self.remote = detect_remote(&self.path);
    }

    /// Quick status summary string.
    pub fn summary(&self) -> String {
        let branch_str = self.branch.as_deref().unwrap_or("(detached)");
        format!(
            "{} @ {} [{}]{}",
            self.name,
            branch_str,
            self.path.display(),
            self.remote
                .as_ref()
                .map(|r| format!(" ← {}", r))
                .unwrap_or_default()
        )
    }
}

/// Thread-safe manager for multi-repo operations.
pub struct MultiRepoManager {
    repos: Arc<RwLock<HashMap<String, RegisteredRepo>>>,
    storage_path: PathBuf,
}

impl MultiRepoManager {
    /// Create a new manager, loading existing registrations from disk.
    pub fn new() -> Self {
        Self::with_storage_path(
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("opencrust/repos.json"),
        )
    }

    /// Create a manager with a specific storage path.
    pub fn with_storage_path(storage_path: PathBuf) -> Self {
        let repos = if storage_path.exists() {
            fs::read_to_string(&storage_path)
                .ok()
                .and_then(|content| {
                    serde_json::from_str::<HashMap<String, RegisteredRepo>>(&content).ok()
                })
                .unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self {
            repos: Arc::new(RwLock::new(repos)),
            storage_path,
        }
    }

    /// Persist current state to disk.
    async fn save(&self) {
        let guard = self.repos.read().await;
        if let Ok(content) = serde_json::to_string_pretty(&*guard) {
            if let Some(parent) = self.storage_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&self.storage_path, content);
        }
    }

    /// Register a new repository by path.
    pub async fn add(
        &self,
        name: String,
        path: PathBuf,
        tags: Vec<String>,
    ) -> Result<RegisteredRepo, String> {
        let mut guard = self.repos.write().await;
        if guard.contains_key(&name) {
            return Err(format!("repo '{}' is already registered", name));
        }
        let repo = RegisteredRepo::new(name.clone(), path, tags)?;
        guard.insert(name, repo.clone());
        drop(guard);
        self.save().await;
        Ok(repo)
    }

    /// Remove a registered repository.
    pub async fn remove(&self, name: &str) -> bool {
        let mut guard = self.repos.write().await;
        let removed = guard.remove(name).is_some();
        drop(guard);
        if removed {
            self.save().await;
        }
        removed
    }

    /// List all registered repositories.
    pub async fn list(&self) -> Vec<RegisteredRepo> {
        let guard = self.repos.read().await;
        let mut repos: Vec<RegisteredRepo> = guard.values().cloned().collect();
        repos.sort_by(|a, b| a.name.cmp(&b.name));
        repos
    }

    /// Get a specific repo by name.
    pub async fn get(&self, name: &str) -> Option<RegisteredRepo> {
        self.repos.read().await.get(name).cloned()
    }

    /// Refresh branch/remote info for all repos.
    pub async fn refresh_all(&self) {
        let mut guard = self.repos.write().await;
        for repo in guard.values_mut() {
            repo.refresh();
        }
    }

    /// Search across all registered repos for a pattern in file names.
    pub async fn search_files(&self, pattern: &str) -> Vec<(RegisteredRepo, Vec<String>)> {
        let repos = self.list().await;
        let mut results = Vec::new();

        for repo in &repos {
            let mut matches = Vec::new();
            if let Ok(dir) = fs::read_dir(&repo.path) {
                for entry in dir.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.contains(pattern) {
                            matches.push(path.display().to_string());
                        }
                    }
                }
            }
            if !matches.is_empty() {
                results.push((repo.clone(), matches));
            }
        }

        results
    }

    /// Run a git command across all repos in parallel.
    pub async fn git_command_all(
        &self,
        args: &[&str],
    ) -> Vec<(RegisteredRepo, Result<String, String>)> {
        let repos = self.list().await;
        let mut handles = Vec::new();

        for repo in repos {
            let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            let path = repo.path.clone();
            let name = repo.name.clone();

            let handle = tokio::spawn(async move {
                let output = std::process::Command::new("git")
                    .args(&args)
                    .current_dir(&path)
                    .output();
                match output {
                    Ok(out) => {
                        if out.status.success() {
                            (name, Ok(String::from_utf8_lossy(&out.stdout).to_string()))
                        } else {
                            (name, Err(String::from_utf8_lossy(&out.stderr).to_string()))
                        }
                    }
                    Err(e) => (name, Err(format!("io error: {}", e))),
                }
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok((name, result)) = handle.await {
                if let Some(repo) = self.get(&name).await {
                    results.push((repo, result));
                }
            }
        }
        results
    }

    /// Return aggregate statistics.
    pub async fn stats(&self) -> RepoStats {
        let repos = self.list().await;
        let total = repos.len();
        let with_remote = repos.iter().filter(|r| r.remote.is_some()).count();
        let indexed = repos.iter().filter(|r| r.last_indexed.is_some()).count();
        RepoStats {
            total,
            with_remote,
            indexed,
        }
    }
}

impl Default for MultiRepoManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregate multi-repo statistics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepoStats {
    pub total: usize,
    pub with_remote: usize,
    pub indexed: usize,
}

impl std::fmt::Display for RepoStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Repos: {} total, {} with remote, {} indexed",
            self.total, self.with_remote, self.indexed
        )
    }
}

// --- Helpers ---

/// Detect the current git branch.
fn detect_branch(path: &Path) -> Option<String> {
    let head_path = path.join(".git").join("HEAD");
    let content = fs::read_to_string(head_path).ok()?;
    if let Some(ref_line) = content.lines().next() {
        if let Some(branch) = ref_line.strip_prefix("ref: refs/heads/") {
            return Some(branch.trim().to_string());
        }
    }
    None
}

/// Detect the remote origin URL.
fn detect_remote(path: &Path) -> Option<String> {
    let config_path = path.join(".git").join("config");
    let content = fs::read_to_string(config_path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(url) = trimmed.strip_prefix("url = ") {
            return Some(url.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_temp_repo() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path().to_path_buf();
        // Initialize a git repo
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .output()
            .unwrap();
        // Set user config to prevent git warnings
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&repo_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&repo_path)
            .output()
            .unwrap();
        (dir, repo_path)
    }

    fn create_test_manager() -> (tempfile::TempDir, MultiRepoManager) {
        let storage_dir = tempfile::tempdir().unwrap();
        let storage_path = storage_dir.path().join("repos.json");
        let mgr = MultiRepoManager::with_storage_path(storage_path);
        (storage_dir, mgr)
    }

    #[test]
    fn test_register_repo() {
        let (_dir, repo_path) = create_temp_repo();
        let repo = RegisteredRepo::new("test-repo".into(), repo_path.clone(), vec!["rust".into()]);
        assert!(repo.is_ok());
        let repo = repo.unwrap();
        assert_eq!(repo.name, "test-repo");
        assert!(repo.tags.contains(&"rust".into()));
    }

    #[test]
    fn test_register_non_git_dir_fails() {
        let dir = tempfile::tempdir().unwrap();
        let result = RegisteredRepo::new("bad".into(), dir.path().to_path_buf(), vec![]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a git repository"));
    }

    #[test]
    fn test_register_nonexistent_path_fails() {
        let result = RegisteredRepo::new("bad".into(), PathBuf::from("/nonexistent/path"), vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_manager_new_is_empty() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (_storage_dir, mgr) = create_test_manager();
        let repos = rt.block_on(async { mgr.list().await });
        assert!(repos.is_empty());
    }

    #[test]
    fn test_manager_add_and_list() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (_dir, repo_path) = create_temp_repo();
        let (_storage_dir, mgr) = create_test_manager();

        rt.block_on(async {
            mgr.add("my-repo".into(), repo_path.clone(), vec![])
                .await
                .unwrap();
            let repos = mgr.list().await;
            assert_eq!(repos.len(), 1);
            assert_eq!(repos[0].name, "my-repo");
        });
    }

    #[test]
    fn test_manager_remove() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (_dir, repo_path) = create_temp_repo();
        let (_storage_dir, mgr) = create_test_manager();

        rt.block_on(async {
            mgr.add("to-remove".into(), repo_path.clone(), vec![])
                .await
                .unwrap();
            assert!(mgr.remove("to-remove").await);
            assert_eq!(mgr.list().await.len(), 0);
        });
    }

    #[test]
    fn test_manager_get() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (_dir, repo_path) = create_temp_repo();
        let (_storage_dir, mgr) = create_test_manager();

        rt.block_on(async {
            mgr.add("get-test".into(), repo_path, vec![]).await.unwrap();
            assert!(mgr.get("get-test").await.is_some());
            assert!(mgr.get("nope").await.is_none());
        });
    }

    #[test]
    fn test_manager_duplicate_add_fails() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (_dir, repo_path) = create_temp_repo();
        let (_storage_dir, mgr) = create_test_manager();

        rt.block_on(async {
            mgr.add("dup".into(), repo_path.clone(), vec![])
                .await
                .unwrap();
            let result = mgr.add("dup".into(), repo_path, vec![]).await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_stats() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (_dir, repo_path) = create_temp_repo();
        let (_storage_dir, mgr) = create_test_manager();
        rt.block_on(async {
            mgr.add("s1".into(), repo_path.clone(), vec![])
                .await
                .unwrap();
            let stats = mgr.stats().await;
            assert_eq!(stats.total, 1);
        });
    }
}
