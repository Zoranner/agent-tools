use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Result as SqlResult};

use super::error::{task_error, TaskErrorCode};
use crate::tool::ToolError;

pub(crate) type Db = Arc<Mutex<Connection>>;

const SCHEMA: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS tasks (
    id              TEXT PRIMARY KEY,
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    kind            TEXT NOT NULL DEFAULT 'task',
    status          TEXT NOT NULL DEFAULT 'backlog',
    owner           TEXT NOT NULL DEFAULT 'agent',
    priority        TEXT,
    tags            TEXT NOT NULL DEFAULT '[]',
    blocked_reason  TEXT,
    goal            TEXT,
    acceptance      TEXT,
    risk_level      TEXT,
    last_error      TEXT,
    retry_count     INTEGER NOT NULL DEFAULT 0,
    artifact_refs   TEXT NOT NULL DEFAULT '[]',
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS task_steps (
    id          TEXT PRIMARY KEY,
    task_id     TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    seq         INTEGER NOT NULL,
    title       TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'pending',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS task_runs (
    id          TEXT PRIMARY KEY,
    task_id     TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    step_id     TEXT REFERENCES task_steps(id),
    started_at  TEXT NOT NULL,
    ended_at    TEXT,
    status      TEXT NOT NULL DEFAULT 'running',
    error       TEXT,
    summary     TEXT
);

CREATE TABLE IF NOT EXISTS path_locks (
    id          TEXT PRIMARY KEY,
    task_id     TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    run_id      TEXT REFERENCES task_runs(id),
    path        TEXT NOT NULL UNIQUE,
    acquired_at TEXT NOT NULL,
    expires_at  TEXT
);

CREATE TABLE IF NOT EXISTS checkpoints (
    id          TEXT PRIMARY KEY,
    task_id     TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    run_id      TEXT REFERENCES task_runs(id),
    status      TEXT NOT NULL DEFAULT 'open',
    message     TEXT NOT NULL,
    risk_level  TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS artifacts (
    id          TEXT PRIMARY KEY,
    task_id     TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    run_id      TEXT REFERENCES task_runs(id),
    kind        TEXT NOT NULL,
    path        TEXT,
    content     TEXT,
    created_at  TEXT NOT NULL
);
"#;

/// 打开（或创建）数据库，执行 schema 迁移，然后执行启动恢复。
pub(crate) fn open(db_path: &Path) -> Result<Db, ToolError> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            task_error(
                TaskErrorCode::StorageError,
                format!("create db directory: {e}"),
            )
        })?;
    }
    let conn = Connection::open(db_path)
        .map_err(|e| task_error(TaskErrorCode::StorageError, format!("open task db: {e}")))?;
    apply_schema(&conn)?;
    recover_on_startup(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}

fn apply_schema(conn: &Connection) -> Result<(), ToolError> {
    conn.execute_batch(SCHEMA).map_err(|e| {
        task_error(
            TaskErrorCode::StorageError,
            format!("apply task schema: {e}"),
        )
    })
}

/// 启动恢复：将遗留 running 任务转为 blocked，回收过期锁。
fn recover_on_startup(conn: &Connection) -> Result<(), ToolError> {
    let now = chrono::Utc::now().to_rfc3339();

    // running 任务 → blocked
    conn.execute(
        "UPDATE tasks SET status = 'blocked', blocked_reason = 'process_restart', updated_at = ?1 WHERE status = 'running'",
        rusqlite::params![now],
    )
    .map_err(|e| task_error(TaskErrorCode::StorageError, format!("recover running tasks: {e}")))?;

    // running runs → failed
    conn.execute(
        "UPDATE task_runs SET status = 'failed', ended_at = ?1, error = 'process_restart' WHERE status = 'running'",
        rusqlite::params![now],
    )
    .map_err(|e| task_error(TaskErrorCode::StorageError, format!("recover running runs: {e}")))?;

    // 回收过期锁
    conn.execute(
        "DELETE FROM path_locks WHERE expires_at IS NOT NULL AND expires_at < ?1",
        rusqlite::params![now],
    )
    .map_err(|e| {
        task_error(
            TaskErrorCode::StorageError,
            format!("recover expired locks: {e}"),
        )
    })?;

    Ok(())
}

pub(crate) fn with_conn<F, T>(db: &Db, f: F) -> Result<T, ToolError>
where
    F: FnOnce(&Connection) -> SqlResult<T>,
{
    let conn = db
        .lock()
        .map_err(|_| task_error(TaskErrorCode::StorageError, "task db mutex poisoned"))?;
    f(&conn).map_err(|e| task_error(TaskErrorCode::StorageError, e.to_string()))
}
