//! Git tools: [`git_status`](GitStatusTool), [`git_diff`](GitDiffTool), [`git_commit`](GitCommitTool), [`git_log`](GitLogTool).
//!
//! [`GitContext::default_repo_root`] is the directory used when `path` is omitted; relative `path`
//! joins against it before [`git2::Repository::discover`].

mod error;
mod ops;
mod tools;

use std::path::PathBuf;

pub use tools::{all_tools, GitCommitTool, GitDiffTool, GitLogTool, GitStatusTool};

/// Default directory for optional `path` in git tools (canonical).
#[derive(Debug, Clone)]
pub struct GitContext {
    pub default_repo_root: PathBuf,
}

impl GitContext {
    /// `root: None` uses [`std::env::current_dir`].
    pub fn new(root: Option<PathBuf>) -> std::io::Result<Self> {
        let r = match root {
            Some(p) => p,
            None => std::env::current_dir()?,
        };
        Ok(Self {
            default_repo_root: r.canonicalize()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;

    use git2::{Repository, Signature};
    use serde_json::json;

    use super::*;

    fn tmp_repo_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "agentool_git_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&dir).expect("create tmp");
        dir
    }

    fn init_repo_with_initial_commit(path: &std::path::Path) {
        let repo = Repository::init(path).expect("init");
        let mut cfg = repo.config().expect("config");
        let _ = cfg.set_str("user.name", "AgentTest");
        let _ = cfg.set_str("user.email", "agent@test.local");
        let mut index = repo.index().expect("index");
        let tree_id = index.write_tree().expect("empty tree");
        let tree = repo.find_tree(tree_id).expect("tree");
        let sig = Signature::now("AgentTest", "agent@test.local").expect("sig");
        repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .expect("commit");
    }

    #[tokio::test]
    async fn git_status_log_and_commit_roundtrip() {
        let dir = tmp_repo_dir();
        init_repo_with_initial_commit(&dir);
        fs::write(dir.join("tracked.txt"), "v1").expect("write");

        let ctx = Arc::new(GitContext::new(Some(dir.clone())).expect("ctx"));
        let tools = all_tools(ctx);

        let status = tools.iter().find(|t| t.name() == "git_status").unwrap();
        let st = status.execute(json!({})).await.expect("status");
        assert_eq!(st["success"], true);
        let changes = st["data"]["changes"].as_array().unwrap();
        assert!(
            changes.iter().any(|c| c["file"] == "tracked.txt"),
            "expected untracked tracked.txt"
        );

        let log_tool = tools.iter().find(|t| t.name() == "git_log").unwrap();
        let log = log_tool.execute(json!({ "limit": 5 })).await.expect("log");
        let commits = log["data"]["commits"].as_array().unwrap();
        assert!(!commits.is_empty());
        assert_eq!(commits[0]["message"], "init");

        let commit_tool = tools.iter().find(|t| t.name() == "git_commit").unwrap();
        let cm = commit_tool
            .execute(json!({ "message": "add tracked" }))
            .await
            .expect("commit");
        assert_eq!(cm["success"], true);
        assert!(cm["data"]["hash"].as_str().unwrap().len() >= 7);

        let st2 = status.execute(json!({})).await.expect("status2");
        let changes2 = st2["data"]["changes"].as_array().unwrap();
        assert!(
            !changes2.iter().any(|c| c["file"] == "tracked.txt"),
            "tracked.txt should be clean after commit"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
