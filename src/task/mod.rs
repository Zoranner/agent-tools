//! Task store for autonomous agents: create, track, and govern tasks with steps,
//! execution runs, file locks, checkpoints, and artifacts.
//!
//! Data is stored in a SQLite database at **`.agent/tasks.db`** under the workspace root
//! (configurable via [`TaskContext::with_db_relative`]).
//!
//! On startup, any tasks left in `running` state are moved to `blocked` and stale
//! path locks are released automatically.

mod db;
mod error;
mod ops;
mod schema;
mod tools;

use std::path::{Path, PathBuf};

pub use tools::{
    all_tools, TaskAcquireLockTool, TaskAddArtifactTool, TaskAppendStepTool,
    TaskCloseCheckpointTool, TaskCreateTool, TaskDeleteTool, TaskEndRunTool, TaskGetTool,
    TaskListTool, TaskOpenCheckpointTool, TaskReleaseLockTool, TaskStartRunTool,
    TaskUpdateStepTool, TaskUpdateTool,
};

use db::Db;

/// Workspace root, sandbox settings, and path to the SQLite task database.
#[derive(Clone)]
pub struct TaskContext {
    pub root_canonical: PathBuf,
    pub allow_outside_root: bool,
    /// Relative to `root_canonical`, e.g. `.agent/tasks.db`.
    pub db_relative: PathBuf,
    pub(crate) db: Db,
}

impl TaskContext {
    /// `root: None` uses [`std::env::current_dir`]. DB defaults to `.agent/tasks.db`.
    pub fn new(root: Option<PathBuf>, allow_outside_root: bool) -> std::io::Result<Self> {
        Self::with_db_relative(root, allow_outside_root, Path::new(".agent/tasks.db"))
    }

    pub fn with_db_relative(
        root: Option<PathBuf>,
        allow_outside_root: bool,
        db_relative: &Path,
    ) -> std::io::Result<Self> {
        let r = match root {
            Some(p) => p,
            None => std::env::current_dir()?,
        };
        let root_canonical = r.canonicalize()?;
        let db_path = root_canonical.join(db_relative);
        let db = db::open(&db_path)
            .map_err(|e| std::io::Error::other(e.message))?;
        Ok(Self {
            root_canonical,
            allow_outside_root,
            db_relative: db_relative.to_path_buf(),
            db,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;

    use serde_json::json;

    use super::*;

    fn tmp_root() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "agentool_task_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&dir).expect("create tmp");
        dir
    }

    #[tokio::test]
    async fn full_lifecycle() {
        let root = tmp_root();
        let ctx = Arc::new(
            TaskContext::with_db_relative(Some(root.clone()), false, Path::new("tasks.db"))
                .unwrap(),
        );
        let tools = all_tools(ctx);

        let create = tools.iter().find(|t| t.name() == "task_create").unwrap();
        let list = tools.iter().find(|t| t.name() == "task_list").unwrap();
        let get = tools.iter().find(|t| t.name() == "task_get").unwrap();
        let update = tools.iter().find(|t| t.name() == "task_update").unwrap();
        let delete = tools.iter().find(|t| t.name() == "task_delete").unwrap();
        let start_run = tools.iter().find(|t| t.name() == "task_start_run").unwrap();
        let end_run = tools.iter().find(|t| t.name() == "task_end_run").unwrap();
        let append_step = tools
            .iter()
            .find(|t| t.name() == "task_append_step")
            .unwrap();
        let update_step = tools
            .iter()
            .find(|t| t.name() == "task_update_step")
            .unwrap();
        let open_cp = tools
            .iter()
            .find(|t| t.name() == "task_open_checkpoint")
            .unwrap();
        let close_cp = tools
            .iter()
            .find(|t| t.name() == "task_close_checkpoint")
            .unwrap();
        let acquire_lock = tools
            .iter()
            .find(|t| t.name() == "task_acquire_lock")
            .unwrap();
        let release_lock = tools
            .iter()
            .find(|t| t.name() == "task_release_lock")
            .unwrap();
        let add_artifact = tools
            .iter()
            .find(|t| t.name() == "task_add_artifact")
            .unwrap();

        // create
        let out = create
            .execute(json!({
                "title": "Write overview.md",
                "description": "Draft the product overview document",
                "priority": "high",
                "tags": ["docs"],
                "goal": "Produce a clear overview for business stakeholders",
                "acceptance": "Reviewed and approved by PM"
            }))
            .await
            .unwrap();
        let task_id = out["data"]["id"].as_str().unwrap().to_string();

        // list
        let all = list.execute(json!({})).await.unwrap();
        assert_eq!(all["data"]["items"].as_array().unwrap().len(), 1);
        assert_eq!(all["data"]["items"][0]["status"], "backlog");

        // get
        let detail = get.execute(json!({ "id": task_id })).await.unwrap();
        assert_eq!(detail["data"]["title"], "Write overview.md");

        // update status
        update
            .execute(json!({ "id": task_id, "status": "ready" }))
            .await
            .unwrap();

        // append steps
        let s1 = append_step
            .execute(json!({ "task_id": task_id, "title": "Research existing docs" }))
            .await
            .unwrap();
        let step1_id = s1["data"]["step_id"].as_str().unwrap().to_string();

        let s2 = append_step
            .execute(json!({ "task_id": task_id, "title": "Write first draft" }))
            .await
            .unwrap();
        let step2_id = s2["data"]["step_id"].as_str().unwrap().to_string();

        // start run
        let run_out = start_run
            .execute(json!({ "task_id": task_id, "step_id": step1_id }))
            .await
            .unwrap();
        let run_id = run_out["data"]["run_id"].as_str().unwrap().to_string();

        // acquire lock
        let lock_out = acquire_lock
            .execute(
                json!({ "task_id": task_id, "run_id": run_id, "path": "/workspace/overview.md" }),
            )
            .await
            .unwrap();
        let lock_id = lock_out["data"]["lock_id"].as_str().unwrap().to_string();

        // duplicate lock should fail
        let lock_err = acquire_lock
            .execute(json!({ "task_id": task_id, "path": "/workspace/overview.md" }))
            .await
            .unwrap_err();
        assert_eq!(lock_err.code, "TASK_LOCK_CONFLICT");

        // update step
        update_step
            .execute(json!({ "step_id": step1_id, "status": "done" }))
            .await
            .unwrap();
        update_step
            .execute(json!({ "step_id": step2_id, "status": "running" }))
            .await
            .unwrap();

        // open checkpoint
        let cp_out = open_cp
            .execute(json!({
                "task_id": task_id,
                "run_id": run_id,
                "message": "Draft complete, please review direction",
                "risk_level": "low"
            }))
            .await
            .unwrap();
        let cp_id = cp_out["data"]["checkpoint_id"]
            .as_str()
            .unwrap()
            .to_string();

        // task should be waiting_checkpoint
        let after_cp = get.execute(json!({ "id": task_id })).await.unwrap();
        assert_eq!(after_cp["data"]["status"], "waiting_checkpoint");

        // close checkpoint
        close_cp
            .execute(json!({ "checkpoint_id": cp_id, "task_status": "running" }))
            .await
            .unwrap();

        // add artifact
        let art_out = add_artifact
            .execute(json!({
                "task_id": task_id,
                "run_id": run_id,
                "kind": "file",
                "path": "/workspace/overview.md"
            }))
            .await
            .unwrap();
        assert!(art_out["data"]["artifact_id"].as_str().is_some());

        // end run
        end_run
            .execute(
                json!({ "run_id": run_id, "status": "done", "summary": "overview.md written" }),
            )
            .await
            .unwrap();

        // release lock
        release_lock
            .execute(json!({ "lock_id": lock_id }))
            .await
            .unwrap();

        // task should be done
        let final_state = get.execute(json!({ "id": task_id })).await.unwrap();
        assert_eq!(final_state["data"]["status"], "done");

        // delete
        delete.execute(json!({ "id": task_id })).await.unwrap();
        let empty = list.execute(json!({})).await.unwrap();
        assert_eq!(empty["data"]["items"].as_array().unwrap().len(), 0);

        // not found
        let miss = get.execute(json!({ "id": "nope" })).await.unwrap_err();
        assert_eq!(miss.code, "TASK_NOT_FOUND");

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn filter_by_status_and_tag() {
        let root = tmp_root();
        let ctx = Arc::new(
            TaskContext::with_db_relative(Some(root.clone()), false, Path::new("tasks.db"))
                .unwrap(),
        );
        let tools = all_tools(ctx);
        let create = tools.iter().find(|t| t.name() == "task_create").unwrap();
        let list = tools.iter().find(|t| t.name() == "task_list").unwrap();
        let update = tools.iter().find(|t| t.name() == "task_update").unwrap();

        let t1 = create
            .execute(json!({ "title": "A", "tags": ["alpha"] }))
            .await
            .unwrap();
        let t1_id = t1["data"]["id"].as_str().unwrap().to_string();
        create
            .execute(json!({ "title": "B", "tags": ["beta"] }))
            .await
            .unwrap();

        update
            .execute(json!({ "id": t1_id, "status": "done" }))
            .await
            .unwrap();

        let done = list.execute(json!({ "status": "done" })).await.unwrap();
        assert_eq!(done["data"]["items"].as_array().unwrap().len(), 1);

        let alpha = list.execute(json!({ "tag": "alpha" })).await.unwrap();
        assert_eq!(alpha["data"]["items"].as_array().unwrap().len(), 1);

        let beta_backlog = list
            .execute(json!({ "tag": "beta", "status": "backlog" }))
            .await
            .unwrap();
        assert_eq!(beta_backlog["data"]["items"].as_array().unwrap().len(), 1);

        let _ = fs::remove_dir_all(&root);
    }
}
