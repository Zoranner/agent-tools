use std::path::{Path, PathBuf};

use chrono::{TimeZone, Utc};
use git2::{
    BranchType, DiffFormat, DiffOptions, Repository, Signature, Status, StatusOptions,
    WorktreeAddOptions,
};
use serde_json::{json, Value};

use crate::core::json::{json_str, ok_data};
use crate::core::path::combine_and_normalize;
use crate::tool::{ToolError, ToolResult};

use super::error::{map_git_err, tool_error, GitErrorCode};
use super::GitContext;

fn repo_logical_path(ctx: &GitContext, path: Option<&str>) -> PathBuf {
    match path.map(str::trim).filter(|s| !s.is_empty()) {
        None => ctx.default_repo_root.clone(),
        Some(s) => {
            let p = Path::new(s);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                combine_and_normalize(&ctx.default_repo_root, p)
            }
        }
    }
}

fn open_repo(ctx: &GitContext, path: Option<&str>) -> Result<Repository, ToolError> {
    let logical = repo_logical_path(ctx, path);
    Repository::discover(logical).map_err(map_git_err)
}

fn status_label(s: Status) -> &'static str {
    if s.is_wt_new() {
        return "untracked";
    }
    if s.is_index_new() {
        return "added";
    }
    if s.is_wt_deleted() || s.is_index_deleted() {
        return "deleted";
    }
    if s.is_wt_modified() || s.is_index_modified() {
        return "modified";
    }
    "modified"
}

pub(crate) fn op_git_status(ctx: &GitContext, params: &Value) -> ToolResult {
    let path = params.get("path").and_then(|v| v.as_str());
    let repo = open_repo(ctx, path)?;
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    opts.include_ignored(false);
    let statuses = repo.statuses(Some(&mut opts)).map_err(map_git_err)?;
    let mut changes = Vec::new();
    for entry in statuses.iter() {
        let Some(p) = entry.path() else {
            continue;
        };
        let st = entry.status();
        if st == Status::CURRENT {
            continue;
        }
        changes.push(json!({
            "file": p,
            "status": status_label(st),
        }));
    }
    Ok(ok_data(json!({ "changes": changes })))
}

pub(crate) fn op_git_diff(ctx: &GitContext, params: &Value) -> ToolResult {
    let path = params.get("path").and_then(|v| v.as_str());
    let staged = params
        .get("staged")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let repo = open_repo(ctx, path)?;
    let mut diff_opts = DiffOptions::new();
    let diff = if staged {
        let head = repo.head().map_err(map_git_err)?;
        let tree = head.peel_to_tree().map_err(map_git_err)?;
        let index = repo.index().map_err(map_git_err)?;
        repo.diff_tree_to_index(Some(&tree), Some(&index), Some(&mut diff_opts))
    } else {
        let index = repo.index().map_err(map_git_err)?;
        repo.diff_index_to_workdir(Some(&index), Some(&mut diff_opts))
    }
    .map_err(map_git_err)?;
    let mut buf = String::new();
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        if let Ok(content) = std::str::from_utf8(line.content()) {
            buf.push_str(content);
        }
        true
    })
    .map_err(map_git_err)?;
    Ok(ok_data(json!({ "diff": buf })))
}

fn signature(repo: &Repository) -> Result<Signature<'_>, ToolError> {
    let cfg = repo.config().map_err(map_git_err)?;
    let name = cfg
        .get_string("user.name")
        .unwrap_or_else(|_| "agentool".to_string());
    let email = cfg
        .get_string("user.email")
        .unwrap_or_else(|_| "agentool@localhost".to_string());
    Signature::now(&name, &email).map_err(map_git_err)
}

pub(crate) fn op_git_commit(ctx: &GitContext, params: &Value) -> ToolResult {
    let message = json_str(params, "message")?;
    let repo = open_repo(ctx, params.get("path").and_then(|v| v.as_str()))?;
    let mut index = repo.index().map_err(map_git_err)?;

    match params.get("files") {
        None => {
            index
                .add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)
                .map_err(map_git_err)?;
        }
        Some(v) => {
            let arr = v.as_array().ok_or_else(|| {
                tool_error(
                    GitErrorCode::GitError,
                    "`files` must be a JSON array of strings",
                )
            })?;
            if arr.is_empty() {
                return Err(tool_error(
                    GitErrorCode::GitError,
                    "`files` must list at least one path when provided",
                ));
            }
            for f in arr {
                let p = f.as_str().ok_or_else(|| {
                    tool_error(GitErrorCode::GitError, "`files` entries must be strings")
                })?;
                index.add_path(Path::new(p)).map_err(map_git_err)?;
            }
        }
    }

    index.write().map_err(map_git_err)?;
    let tree_id = index.write_tree().map_err(map_git_err)?;
    let tree = repo.find_tree(tree_id).map_err(map_git_err)?;
    let sig = signature(&repo)?;
    let parents: Vec<git2::Commit> = match repo.head() {
        Ok(head) => {
            let c = head.peel_to_commit().map_err(map_git_err)?;
            vec![c]
        }
        Err(_) => vec![],
    };
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
    let oid = repo
        .commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
        .map_err(map_git_err)?;
    Ok(ok_data(json!({
        "hash": oid.to_string(),
        "message": message,
    })))
}

pub(crate) fn op_git_log(ctx: &GitContext, params: &Value) -> ToolResult {
    let path = params.get("path").and_then(|v| v.as_str());
    let limit = params
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10)
        .min(100) as usize;
    let repo = open_repo(ctx, path)?;
    let mut revwalk = repo.revwalk().map_err(map_git_err)?;
    revwalk.push_head().map_err(map_git_err)?;
    let mut commits = Vec::new();
    for oid in revwalk.take(limit) {
        let oid = oid.map_err(map_git_err)?;
        let c = repo.find_commit(oid).map_err(map_git_err)?;
        let msg = c.summary().unwrap_or("").to_string();
        let author = c.author().name().unwrap_or("").to_string();
        let t = c.time();
        let date = Utc
            .timestamp_opt(t.seconds(), 0)
            .single()
            .map(|d| d.to_rfc3339())
            .unwrap_or_default();
        commits.push(json!({
            "hash": oid.to_string(),
            "message": msg,
            "author": author,
            "date": date,
        }));
    }
    Ok(ok_data(json!({ "commits": commits })))
}

pub(crate) fn op_worktree_add(ctx: &GitContext, params: &Value) -> ToolResult {
    let name = json_str(params, "name")?;
    let wt_path_str = json_str(params, "path")?;
    let branch_name = params.get("branch").and_then(|v| v.as_str());
    let repo = open_repo(ctx, params.get("repo").and_then(|v| v.as_str()))?;

    let wt_path = {
        let p = Path::new(wt_path_str);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            combine_and_normalize(&ctx.default_repo_root, p)
        }
    };

    let mut opts = WorktreeAddOptions::new();
    // If a branch name is given, resolve or create it and attach to the worktree.
    let branch_ref;
    if let Some(bname) = branch_name {
        let reference = match repo.find_branch(bname, BranchType::Local) {
            Ok(b) => b.into_reference(),
            Err(e) if e.code() == git2::ErrorCode::NotFound => {
                // Branch does not exist — create from HEAD
                let head_commit = repo
                    .head()
                    .map_err(map_git_err)?
                    .peel_to_commit()
                    .map_err(map_git_err)?;
                repo.branch(bname, &head_commit, false)
                    .map_err(map_git_err)?
                    .into_reference()
            }
            Err(e) => return Err(map_git_err(e)),
        };
        branch_ref = reference;
        opts.reference(Some(&branch_ref));
    }

    let wt = repo
        .worktree(name, &wt_path, Some(&opts))
        .map_err(map_git_err)?;

    Ok(ok_data(json!({
        "name": wt.name().unwrap_or(name),
        "path": wt.path().to_string_lossy(),
        "branch": branch_name.unwrap_or(""),
    })))
}

pub(crate) fn op_worktree_list(ctx: &GitContext, params: &Value) -> ToolResult {
    let repo = open_repo(ctx, params.get("repo").and_then(|v| v.as_str()))?;
    let names = repo.worktrees().map_err(map_git_err)?;
    let mut list = Vec::new();
    for name in names.iter().flatten() {
        let wt = repo.find_worktree(name).map_err(map_git_err)?;
        let locked = !matches!(
            wt.is_locked().map_err(map_git_err)?,
            git2::WorktreeLockStatus::Unlocked
        );
        list.push(json!({
            "name": name,
            "path": wt.path().to_string_lossy(),
            "locked": locked,
        }));
    }
    Ok(ok_data(json!({ "worktrees": list })))
}

pub(crate) fn op_worktree_remove(ctx: &GitContext, params: &Value) -> ToolResult {
    let name = json_str(params, "name")?;
    let force = params
        .get("force")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let repo = open_repo(ctx, params.get("repo").and_then(|v| v.as_str()))?;
    let wt = repo.find_worktree(name).map_err(map_git_err)?;
    let mut prune_opts = git2::WorktreePruneOptions::new();
    prune_opts.working_tree(true);
    prune_opts.valid(true);
    if force {
        prune_opts.locked(true);
    }
    wt.prune(Some(&mut prune_opts)).map_err(map_git_err)?;
    Ok(ok_data(json!({ "name": name })))
}

pub(crate) fn op_worktree_lock(ctx: &GitContext, params: &Value) -> ToolResult {
    let name = json_str(params, "name")?;
    let reason = params.get("reason").and_then(|v| v.as_str());
    let repo = open_repo(ctx, params.get("repo").and_then(|v| v.as_str()))?;
    let wt = repo.find_worktree(name).map_err(map_git_err)?;
    wt.lock(reason).map_err(map_git_err)?;
    Ok(ok_data(json!({
        "name": name,
        "reason": reason.unwrap_or(""),
    })))
}

pub(crate) fn op_worktree_unlock(ctx: &GitContext, params: &Value) -> ToolResult {
    let name = json_str(params, "name")?;
    let repo = open_repo(ctx, params.get("repo").and_then(|v| v.as_str()))?;
    let wt = repo.find_worktree(name).map_err(map_git_err)?;
    wt.unlock().map_err(map_git_err)?;
    Ok(ok_data(json!({ "name": name })))
}
