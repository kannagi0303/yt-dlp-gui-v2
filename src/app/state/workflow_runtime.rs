use std::process::Child;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use crate::domain::{QueueItemId, ToolKind, WorkflowKind, WorkflowRunId};

pub(super) struct ActiveWorkflow {
    pub(super) item_id: QueueItemId,
    pub(super) workflow_id: WorkflowRunId,
    pub(super) kind: WorkflowKind,
    pub(super) tool: ToolKind,
    pub(super) download_child: Option<Arc<Mutex<Option<Child>>>>,
    pub(super) cancel_requested: Option<Arc<AtomicBool>>,
}

pub(super) struct AnalyzeResult {
    pub(super) source: String,
    pub(super) target_item_id: Option<QueueItemId>,
    pub(super) workflow_id: Option<WorkflowRunId>,
    pub(super) used_cookies: bool,
    pub(super) tool_log_action_id: Option<u64>,
    pub(super) command_line: Option<String>,
    pub(super) result: Result<serde_json::Value, String>,
}
