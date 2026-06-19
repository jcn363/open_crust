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
