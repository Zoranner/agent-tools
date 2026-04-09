use serde::{Deserialize, Serialize};

use super::error::{task_error, TaskErrorCode};
use crate::tool::ToolError;

// ── TaskKind ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TaskKind {
    #[default]
    Task,
    Milestone,
    Checkpoint,
}

impl TaskKind {
    pub(crate) fn parse(s: &str) -> Result<Self, ToolError> {
        match s {
            "task" => Ok(Self::Task),
            "milestone" => Ok(Self::Milestone),
            "checkpoint" => Ok(Self::Checkpoint),
            other => Err(task_error(
                TaskErrorCode::InvalidKind,
                format!("kind must be \"task\", \"milestone\", or \"checkpoint\", got {other:?}"),
            )),
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Milestone => "milestone",
            Self::Checkpoint => "checkpoint",
        }
    }
}

// ── TaskStatus ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TaskStatus {
    #[default]
    Backlog,
    Ready,
    Running,
    WaitingCheckpoint,
    Blocked,
    Done,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub(crate) fn parse(s: &str) -> Result<Self, ToolError> {
        match s {
            "backlog" => Ok(Self::Backlog),
            "ready" => Ok(Self::Ready),
            "running" => Ok(Self::Running),
            "waiting_checkpoint" => Ok(Self::WaitingCheckpoint),
            "blocked" => Ok(Self::Blocked),
            "done" => Ok(Self::Done),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(task_error(
                TaskErrorCode::InvalidStatus,
                format!("invalid task status {other:?}"),
            )),
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Backlog => "backlog",
            Self::Ready => "ready",
            Self::Running => "running",
            Self::WaitingCheckpoint => "waiting_checkpoint",
            Self::Blocked => "blocked",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

// ── TaskOwner ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TaskOwner {
    #[default]
    Agent,
    Human,
}

impl TaskOwner {
    pub(crate) fn parse(s: &str) -> Result<Self, ToolError> {
        match s {
            "agent" => Ok(Self::Agent),
            "human" => Ok(Self::Human),
            other => Err(task_error(
                TaskErrorCode::InvalidOwner,
                format!("owner must be \"agent\" or \"human\", got {other:?}"),
            )),
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Human => "human",
        }
    }
}

// ── TaskPriority ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TaskPriority {
    Low,
    Medium,
    High,
}

impl TaskPriority {
    pub(crate) fn parse(s: &str) -> Result<Self, ToolError> {
        match s {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            other => Err(task_error(
                TaskErrorCode::InvalidPriority,
                format!("priority must be \"low\", \"medium\", or \"high\", got {other:?}"),
            )),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

// ── RiskLevel ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub(crate) fn parse(s: &str) -> Result<Self, ToolError> {
        match s {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            other => Err(task_error(
                TaskErrorCode::InvalidInput,
                format!("risk_level must be \"low\", \"medium\", \"high\", or \"critical\", got {other:?}"),
            )),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

// ── StepStatus ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StepStatus {
    #[default]
    Pending,
    Running,
    Done,
    Failed,
    Cancelled,
}

impl StepStatus {
    pub(crate) fn parse(s: &str) -> Result<Self, ToolError> {
        match s {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "done" => Ok(Self::Done),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(task_error(
                TaskErrorCode::InvalidStatus,
                format!("invalid step status {other:?}"),
            )),
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

// ── RunStatus ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RunStatus {
    #[default]
    Running,
    Done,
    Failed,
    Cancelled,
}

impl RunStatus {
    pub(crate) fn parse(s: &str) -> Result<Self, ToolError> {
        match s {
            "running" => Ok(Self::Running),
            "done" => Ok(Self::Done),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(task_error(
                TaskErrorCode::InvalidStatus,
                format!("invalid run status {other:?}"),
            )),
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

// ── CheckpointStatus ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CheckpointStatus {
    #[default]
    Open,
    Acknowledged,
    ActionRequired,
    Resolved,
    Closed,
}

impl CheckpointStatus {
    pub(crate) fn parse(s: &str) -> Result<Self, ToolError> {
        match s {
            "open" => Ok(Self::Open),
            "acknowledged" => Ok(Self::Acknowledged),
            "action_required" => Ok(Self::ActionRequired),
            "resolved" => Ok(Self::Resolved),
            "closed" => Ok(Self::Closed),
            other => Err(task_error(
                TaskErrorCode::InvalidStatus,
                format!("invalid checkpoint status {other:?}"),
            )),
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Acknowledged => "acknowledged",
            Self::ActionRequired => "action_required",
            Self::Resolved => "resolved",
            Self::Closed => "closed",
        }
    }
}
