use chrono::Utc;
use rusqlite::params;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::core::json::{json_str, ok_data};
use crate::tool::ToolResult;

use super::db::{with_conn, Db};
use super::error::{task_error, TaskErrorCode};
use super::schema::{
    CheckpointStatus, RiskLevel, RunStatus, StepStatus, TaskKind, TaskOwner, TaskPriority,
    TaskStatus,
};

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

fn tags_from_value(v: &Value) -> Vec<String> {
    v.get("tags")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn tags_to_json(tags: &[String]) -> String {
    serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string())
}

fn tags_from_json(s: &str) -> Vec<String> {
    serde_json::from_str(s).unwrap_or_default()
}

// ── task_create ───────────────────────────────────────────────────────────────

pub(crate) fn op_task_create(db: &Db, params: &Value) -> ToolResult {
    let title = json_str(params, "title")?.trim().to_string();
    if title.is_empty() {
        return Err(task_error(
            TaskErrorCode::InvalidInput,
            "`title` must be non-empty",
        ));
    }
    let description = params
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let kind = match params.get("kind").and_then(|v| v.as_str()) {
        Some(s) => TaskKind::parse(s)?,
        None => TaskKind::default(),
    };
    let owner = match params.get("owner").and_then(|v| v.as_str()) {
        Some(s) => TaskOwner::parse(s)?,
        None => TaskOwner::default(),
    };
    let priority = match params.get("priority").and_then(|v| v.as_str()) {
        Some(s) => Some(TaskPriority::parse(s)?),
        None => None,
    };
    let tags = tags_from_value(params);
    let goal = params
        .get("goal")
        .and_then(|v| v.as_str())
        .map(String::from);
    let acceptance = params
        .get("acceptance")
        .and_then(|v| v.as_str())
        .map(String::from);
    let risk_level = match params.get("risk_level").and_then(|v| v.as_str()) {
        Some(s) => Some(RiskLevel::parse(s)?),
        None => None,
    };

    let id = new_id();
    let now = now();
    with_conn(db, |conn| {
        conn.execute(
            "INSERT INTO tasks (id, title, description, kind, status, owner, priority, tags, goal, acceptance, risk_level, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'backlog', ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
            params![
                id,
                title,
                description,
                kind.as_str(),
                owner.as_str(),
                priority.map(|p| p.as_str()),
                tags_to_json(&tags),
                goal,
                acceptance,
                risk_level.map(|r| r.as_str()),
                now,
            ],
        )?;
        Ok(())
    })?;
    Ok(ok_data(json!({ "id": id })))
}

// ── task_list ─────────────────────────────────────────────────────────────────

pub(crate) fn op_task_list(db: &Db, params: &Value) -> ToolResult {
    let status_filter = match params.get("status").and_then(|v| v.as_str()) {
        Some(s) => Some(TaskStatus::parse(s)?),
        None => None,
    };
    let kind_filter = match params.get("kind").and_then(|v| v.as_str()) {
        Some(s) => Some(TaskKind::parse(s)?),
        None => None,
    };
    let owner_filter = match params.get("owner").and_then(|v| v.as_str()) {
        Some(s) => Some(TaskOwner::parse(s)?),
        None => None,
    };
    let tag_filter = params.get("tag").and_then(|v| v.as_str()).map(String::from);
    let limit = params
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(50)
        .clamp(1, 200) as usize;

    let items = with_conn(db, |conn| {
        let mut stmt = conn.prepare(
            "SELECT id, title, description, kind, status, owner, priority, tags, blocked_reason,
                    goal, acceptance, risk_level, last_error, retry_count, artifact_refs, created_at, updated_at
             FROM tasks ORDER BY updated_at DESC",
        )?;
        let rows: Vec<Value> = stmt
            .query_map([], |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "title": row.get::<_, String>(1)?,
                    "description": row.get::<_, String>(2)?,
                    "kind": row.get::<_, String>(3)?,
                    "status": row.get::<_, String>(4)?,
                    "owner": row.get::<_, String>(5)?,
                    "priority": row.get::<_, Option<String>>(6)?,
                    "tags": tags_from_json(&row.get::<_, String>(7)?),
                    "blocked_reason": row.get::<_, Option<String>>(8)?,
                    "goal": row.get::<_, Option<String>>(9)?,
                    "acceptance": row.get::<_, Option<String>>(10)?,
                    "risk_level": row.get::<_, Option<String>>(11)?,
                    "last_error": row.get::<_, Option<String>>(12)?,
                    "retry_count": row.get::<_, i64>(13)?,
                    "artifact_refs": tags_from_json(&row.get::<_, String>(14)?),
                    "created_at": row.get::<_, String>(15)?,
                    "updated_at": row.get::<_, String>(16)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .filter(|item| {
                if let Some(ref sf) = status_filter {
                    if item["status"].as_str() != Some(sf.as_str()) {
                        return false;
                    }
                }
                if let Some(ref kf) = kind_filter {
                    if item["kind"].as_str() != Some(kf.as_str()) {
                        return false;
                    }
                }
                if let Some(ref of_) = owner_filter {
                    if item["owner"].as_str() != Some(of_.as_str()) {
                        return false;
                    }
                }
                if let Some(ref tag) = tag_filter {
                    let has_tag = item["tags"]
                        .as_array()
                        .map(|a| a.iter().any(|t| t.as_str() == Some(tag.as_str())))
                        .unwrap_or(false);
                    if !has_tag {
                        return false;
                    }
                }
                true
            })
            .take(limit)
            .collect();
        Ok(rows)
    })?;
    Ok(ok_data(json!({ "items": items })))
}

// ── task_get ──────────────────────────────────────────────────────────────────

pub(crate) fn op_task_get(db: &Db, params: &Value) -> ToolResult {
    let id = json_str(params, "id")?;
    let task = with_conn(db, |conn| {
        conn.query_row(
            "SELECT id, title, description, kind, status, owner, priority, tags, blocked_reason,
                    goal, acceptance, risk_level, last_error, retry_count, artifact_refs, created_at, updated_at
             FROM tasks WHERE id = ?1",
            params![id],
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "title": row.get::<_, String>(1)?,
                    "description": row.get::<_, String>(2)?,
                    "kind": row.get::<_, String>(3)?,
                    "status": row.get::<_, String>(4)?,
                    "owner": row.get::<_, String>(5)?,
                    "priority": row.get::<_, Option<String>>(6)?,
                    "tags": tags_from_json(&row.get::<_, String>(7)?),
                    "blocked_reason": row.get::<_, Option<String>>(8)?,
                    "goal": row.get::<_, Option<String>>(9)?,
                    "acceptance": row.get::<_, Option<String>>(10)?,
                    "risk_level": row.get::<_, Option<String>>(11)?,
                    "last_error": row.get::<_, Option<String>>(12)?,
                    "retry_count": row.get::<_, i64>(13)?,
                    "artifact_refs": tags_from_json(&row.get::<_, String>(14)?),
                    "created_at": row.get::<_, String>(15)?,
                    "updated_at": row.get::<_, String>(16)?,
                }))
            },
        )
    })
    .map_err(|e| {
        if e.code == "TASK_STORAGE_ERROR" && e.message.contains("no rows") {
            task_error(TaskErrorCode::NotFound, format!("no task with id `{id}`"))
        } else {
            e
        }
    })?;
    Ok(ok_data(task))
}

// ── task_update ───────────────────────────────────────────────────────────────

pub(crate) fn op_task_update(db: &Db, params: &Value) -> ToolResult {
    let id = json_str(params, "id")?;
    let now = now();

    with_conn(db, |conn| {
        conn.query_row(
            "SELECT id FROM tasks WHERE id = ?1",
            params![id],
            |_| Ok(()),
        )
    })
    .map_err(|_| task_error(TaskErrorCode::NotFound, format!("no task with id `{id}`")))?;

    if let Some(v) = params.get("title") {
        if let Some(s) = v.as_str() {
            let t = s.trim();
            if t.is_empty() {
                return Err(task_error(
                    TaskErrorCode::InvalidInput,
                    "`title` cannot be empty",
                ));
            }
            with_conn(db, |conn| {
                conn.execute(
                    "UPDATE tasks SET title = ?1, updated_at = ?2 WHERE id = ?3",
                    params![t, now, id],
                )?;
                Ok(())
            })?;
        }
    }
    if let Some(v) = params.get("description") {
        if let Some(s) = v.as_str() {
            with_conn(db, |conn| {
                conn.execute(
                    "UPDATE tasks SET description = ?1, updated_at = ?2 WHERE id = ?3",
                    params![s, now, id],
                )?;
                Ok(())
            })?;
        }
    }
    if let Some(v) = params.get("status") {
        if let Some(s) = v.as_str() {
            let st = TaskStatus::parse(s)?;
            with_conn(db, |conn| {
                conn.execute(
                    "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
                    params![st.as_str(), now, id],
                )?;
                Ok(())
            })?;
        }
    }
    if let Some(v) = params.get("priority") {
        if v.is_null() {
            with_conn(db, |conn| {
                conn.execute(
                    "UPDATE tasks SET priority = NULL, updated_at = ?1 WHERE id = ?2",
                    params![now, id],
                )?;
                Ok(())
            })?;
        } else if let Some(s) = v.as_str() {
            let p = TaskPriority::parse(s)?;
            with_conn(db, |conn| {
                conn.execute(
                    "UPDATE tasks SET priority = ?1, updated_at = ?2 WHERE id = ?3",
                    params![p.as_str(), now, id],
                )?;
                Ok(())
            })?;
        }
    }
    if let Some(v) = params.get("blocked_reason") {
        let br: Option<&str> = if v.is_null() { None } else { v.as_str() };
        with_conn(db, |conn| {
            conn.execute(
                "UPDATE tasks SET blocked_reason = ?1, updated_at = ?2 WHERE id = ?3",
                params![br, now, id],
            )?;
            Ok(())
        })?;
    }
    if let Some(v) = params.get("last_error") {
        let le: Option<&str> = if v.is_null() { None } else { v.as_str() };
        with_conn(db, |conn| {
            conn.execute(
                "UPDATE tasks SET last_error = ?1, updated_at = ?2 WHERE id = ?3",
                params![le, now, id],
            )?;
            Ok(())
        })?;
    }
    if let Some(v) = params.get("tags") {
        if let Some(arr) = v.as_array() {
            let tags: Vec<String> = arr
                .iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect();
            let tags_json = tags_to_json(&tags);
            with_conn(db, |conn| {
                conn.execute(
                    "UPDATE tasks SET tags = ?1, updated_at = ?2 WHERE id = ?3",
                    params![tags_json, now, id],
                )?;
                Ok(())
            })?;
        }
    }
    if let Some(v) = params.get("risk_level") {
        if v.is_null() {
            with_conn(db, |conn| {
                conn.execute(
                    "UPDATE tasks SET risk_level = NULL, updated_at = ?1 WHERE id = ?2",
                    params![now, id],
                )?;
                Ok(())
            })?;
        } else if let Some(s) = v.as_str() {
            let rl = RiskLevel::parse(s)?;
            with_conn(db, |conn| {
                conn.execute(
                    "UPDATE tasks SET risk_level = ?1, updated_at = ?2 WHERE id = ?3",
                    params![rl.as_str(), now, id],
                )?;
                Ok(())
            })?;
        }
    }

    Ok(ok_data(json!({ "id": id })))
}

// ── task_delete ───────────────────────────────────────────────────────────────

pub(crate) fn op_task_delete(db: &Db, params: &Value) -> ToolResult {
    let id = json_str(params, "id")?;
    let affected = with_conn(db, |conn| {
        conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])
    })?;
    if affected == 0 {
        return Err(task_error(
            TaskErrorCode::NotFound,
            format!("no task with id `{id}`"),
        ));
    }
    Ok(ok_data(json!({ "id": id, "deleted": true })))
}

// ── task_start_run ────────────────────────────────────────────────────────────

pub(crate) fn op_task_start_run(db: &Db, params: &Value) -> ToolResult {
    let task_id = json_str(params, "task_id")?;
    let step_id = params
        .get("step_id")
        .and_then(|v| v.as_str())
        .map(String::from);
    let now = now();
    let run_id = new_id();

    with_conn(db, |conn| {
        conn.query_row(
            "SELECT id FROM tasks WHERE id = ?1",
            params![task_id],
            |_| Ok(()),
        )
    })
    .map_err(|_| {
        task_error(
            TaskErrorCode::NotFound,
            format!("no task with id `{task_id}`"),
        )
    })?;

    with_conn(db, |conn| {
        conn.execute(
            "INSERT INTO task_runs (id, task_id, step_id, started_at, status) VALUES (?1, ?2, ?3, ?4, 'running')",
            params![run_id, task_id, step_id, now],
        )?;
        conn.execute(
            "UPDATE tasks SET status = 'running', updated_at = ?1 WHERE id = ?2",
            params![now, task_id],
        )?;
        Ok(())
    })?;
    Ok(ok_data(json!({ "run_id": run_id })))
}

// ── task_end_run ──────────────────────────────────────────────────────────────

pub(crate) fn op_task_end_run(db: &Db, params: &Value) -> ToolResult {
    let run_id = json_str(params, "run_id")?;
    let status_str = params
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("done");
    let status = RunStatus::parse(status_str)?;
    let error = params
        .get("error")
        .and_then(|v| v.as_str())
        .map(String::from);
    let summary = params
        .get("summary")
        .and_then(|v| v.as_str())
        .map(String::from);
    let now = now();

    let task_id: String = with_conn(db, |conn| {
        conn.query_row(
            "SELECT task_id FROM task_runs WHERE id = ?1",
            params![run_id],
            |row| row.get(0),
        )
    })
    .map_err(|_| {
        task_error(
            TaskErrorCode::NotFound,
            format!("no run with id `{run_id}`"),
        )
    })?;

    with_conn(db, |conn| {
        conn.execute(
            "UPDATE task_runs SET status = ?1, ended_at = ?2, error = ?3, summary = ?4 WHERE id = ?5",
            params![status.as_str(), now, error, summary, run_id],
        )?;
        let task_status = match status {
            RunStatus::Done => "done",
            RunStatus::Failed => "failed",
            RunStatus::Cancelled => "cancelled",
            RunStatus::Running => "running",
        };
        conn.execute(
            "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![task_status, now, task_id],
        )?;
        Ok(())
    })?;
    Ok(ok_data(json!({ "run_id": run_id })))
}

// ── task_append_step ──────────────────────────────────────────────────────────

pub(crate) fn op_task_append_step(db: &Db, params: &Value) -> ToolResult {
    let task_id = json_str(params, "task_id")?;
    let title = json_str(params, "title")?.trim().to_string();
    if title.is_empty() {
        return Err(task_error(
            TaskErrorCode::InvalidInput,
            "`title` must be non-empty",
        ));
    }
    let now = now();
    let step_id = new_id();

    with_conn(db, |conn| {
        conn.query_row(
            "SELECT id FROM tasks WHERE id = ?1",
            params![task_id],
            |_| Ok(()),
        )
    })
    .map_err(|_| {
        task_error(
            TaskErrorCode::NotFound,
            format!("no task with id `{task_id}`"),
        )
    })?;

    with_conn(db, |conn| {
        let seq: i64 = conn.query_row(
            "SELECT COALESCE(MAX(seq), 0) + 1 FROM task_steps WHERE task_id = ?1",
            params![task_id],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT INTO task_steps (id, task_id, seq, title, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, 'pending', ?5, ?5)",
            params![step_id, task_id, seq, title, now],
        )?;
        Ok(())
    })?;
    Ok(ok_data(json!({ "step_id": step_id })))
}

// ── task_update_step ──────────────────────────────────────────────────────────

pub(crate) fn op_task_update_step(db: &Db, params: &Value) -> ToolResult {
    let step_id = json_str(params, "step_id")?;
    let status_str = json_str(params, "status")?;
    let status = StepStatus::parse(status_str)?;
    let now = now();

    let affected = with_conn(db, |conn| {
        conn.execute(
            "UPDATE task_steps SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), now, step_id],
        )
    })?;
    if affected == 0 {
        return Err(task_error(
            TaskErrorCode::NotFound,
            format!("no step with id `{step_id}`"),
        ));
    }
    Ok(ok_data(json!({ "step_id": step_id })))
}

// ── task_open_checkpoint ──────────────────────────────────────────────────────

pub(crate) fn op_task_open_checkpoint(db: &Db, params: &Value) -> ToolResult {
    let task_id = json_str(params, "task_id")?;
    let message = json_str(params, "message")?;
    let run_id = params
        .get("run_id")
        .and_then(|v| v.as_str())
        .map(String::from);
    let risk_level = match params.get("risk_level").and_then(|v| v.as_str()) {
        Some(s) => Some(RiskLevel::parse(s)?),
        None => None,
    };
    let now = now();
    let cp_id = new_id();

    with_conn(db, |conn| {
        conn.query_row(
            "SELECT id FROM tasks WHERE id = ?1",
            params![task_id],
            |_| Ok(()),
        )
    })
    .map_err(|_| {
        task_error(
            TaskErrorCode::NotFound,
            format!("no task with id `{task_id}`"),
        )
    })?;

    with_conn(db, |conn| {
        conn.execute(
            "INSERT INTO checkpoints (id, task_id, run_id, status, message, risk_level, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'open', ?4, ?5, ?6, ?6)",
            params![cp_id, task_id, run_id, message, risk_level.map(|r| r.as_str()), now],
        )?;
        conn.execute(
            "UPDATE tasks SET status = 'waiting_checkpoint', updated_at = ?1 WHERE id = ?2",
            params![now, task_id],
        )?;
        Ok(())
    })?;
    Ok(ok_data(json!({ "checkpoint_id": cp_id })))
}

// ── task_close_checkpoint ─────────────────────────────────────────────────────

pub(crate) fn op_task_close_checkpoint(db: &Db, params: &Value) -> ToolResult {
    let cp_id = json_str(params, "checkpoint_id")?;
    let status_str = params
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("closed");
    let status = CheckpointStatus::parse(status_str)?;
    let next_task_status = params
        .get("task_status")
        .and_then(|v| v.as_str())
        .unwrap_or("ready");
    let _ = TaskStatus::parse(next_task_status)?;
    let now = now();

    let task_id: String = with_conn(db, |conn| {
        conn.query_row(
            "SELECT task_id FROM checkpoints WHERE id = ?1",
            params![cp_id],
            |row| row.get(0),
        )
    })
    .map_err(|_| {
        task_error(
            TaskErrorCode::NotFound,
            format!("no checkpoint with id `{cp_id}`"),
        )
    })?;

    with_conn(db, |conn| {
        conn.execute(
            "UPDATE checkpoints SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), now, cp_id],
        )?;
        conn.execute(
            "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![next_task_status, now, task_id],
        )?;
        Ok(())
    })?;
    Ok(ok_data(json!({ "checkpoint_id": cp_id })))
}

// ── task_acquire_lock ─────────────────────────────────────────────────────────

pub(crate) fn op_task_acquire_lock(db: &Db, params: &Value) -> ToolResult {
    let task_id = json_str(params, "task_id")?;
    let path = json_str(params, "path")?;
    let run_id = params
        .get("run_id")
        .and_then(|v| v.as_str())
        .map(String::from);
    let expires_at = params
        .get("expires_at")
        .and_then(|v| v.as_str())
        .map(String::from);
    let now = now();
    let lock_id = new_id();

    with_conn(db, |conn| {
        conn.query_row(
            "SELECT id FROM tasks WHERE id = ?1",
            params![task_id],
            |_| Ok(()),
        )
    })
    .map_err(|_| {
        task_error(
            TaskErrorCode::NotFound,
            format!("no task with id `{task_id}`"),
        )
    })?;

    with_conn(db, |conn| {
        conn.execute(
            "INSERT INTO path_locks (id, task_id, run_id, path, acquired_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![lock_id, task_id, run_id, path, now, expires_at],
        )
    })
    .map_err(|e| {
        if e.message.contains("UNIQUE") {
            task_error(TaskErrorCode::LockConflict, format!("path `{path}` is already locked"))
        } else {
            e
        }
    })?;
    Ok(ok_data(json!({ "lock_id": lock_id })))
}

// ── task_release_lock ─────────────────────────────────────────────────────────

pub(crate) fn op_task_release_lock(db: &Db, params: &Value) -> ToolResult {
    let lock_id = json_str(params, "lock_id")?;
    let affected = with_conn(db, |conn| {
        conn.execute("DELETE FROM path_locks WHERE id = ?1", params![lock_id])
    })?;
    if affected == 0 {
        return Err(task_error(
            TaskErrorCode::NotFound,
            format!("no lock with id `{lock_id}`"),
        ));
    }
    Ok(ok_data(json!({ "lock_id": lock_id, "released": true })))
}

// ── task_add_artifact ─────────────────────────────────────────────────────────

pub(crate) fn op_task_add_artifact(db: &Db, params: &Value) -> ToolResult {
    let task_id = json_str(params, "task_id")?;
    let kind = json_str(params, "kind")?;
    let run_id = params
        .get("run_id")
        .and_then(|v| v.as_str())
        .map(String::from);
    let path = params
        .get("path")
        .and_then(|v| v.as_str())
        .map(String::from);
    let content = params
        .get("content")
        .and_then(|v| v.as_str())
        .map(String::from);
    let now = now();
    let artifact_id = new_id();

    with_conn(db, |conn| {
        conn.query_row(
            "SELECT id FROM tasks WHERE id = ?1",
            params![task_id],
            |_| Ok(()),
        )
    })
    .map_err(|_| {
        task_error(
            TaskErrorCode::NotFound,
            format!("no task with id `{task_id}`"),
        )
    })?;

    with_conn(db, |conn| {
        conn.execute(
            "INSERT INTO artifacts (id, task_id, run_id, kind, path, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![artifact_id, task_id, run_id, kind, path, content, now],
        )?;
        let refs_json: String = conn.query_row(
            "SELECT artifact_refs FROM tasks WHERE id = ?1",
            params![task_id],
            |row| row.get(0),
        )?;
        let mut refs: Vec<String> = serde_json::from_str(&refs_json).unwrap_or_default();
        refs.push(artifact_id.to_string());
        let new_refs = serde_json::to_string(&refs).unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "UPDATE tasks SET artifact_refs = ?1, updated_at = ?2 WHERE id = ?3",
            params![new_refs, now, task_id],
        )?;
        Ok(())
    })?;
    Ok(ok_data(json!({ "artifact_id": artifact_id })))
}
