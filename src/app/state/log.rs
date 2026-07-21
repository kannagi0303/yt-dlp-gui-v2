use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::WorkflowRunId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolLogStatus {
    Running,
    Success,
    Recovered,
    Failed,
    Skipped,
}

#[derive(Clone, Debug)]
pub struct ToolLogStep {
    pub id: u64,
    pub status: ToolLogStatus,
    pub tool: String,
    pub action: String,
    pub command: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ToolLogAction {
    pub id: u64,
    pub timestamp: String,
    pub status: ToolLogStatus,
    pub mode: String,
    pub action: String,
    pub steps: Vec<ToolLogStep>,
}

#[derive(Debug)]
pub(super) struct LogState {
    pub(super) runtime_log: VecDeque<String>,
    pub(super) tool_logs: VecDeque<ToolLogAction>,
    pub(super) viewer_selected_step: Option<u64>,
    pub(super) viewer_expanded_action: Option<u64>,
    pub(super) action_by_workflow: HashMap<WorkflowRunId, u64>,
    pub(super) next_action_id: u64,
    pub(super) next_step_id: u64,
}

impl Default for LogState {
    fn default() -> Self {
        Self {
            runtime_log: VecDeque::new(),
            tool_logs: VecDeque::new(),
            viewer_selected_step: None,
            viewer_expanded_action: None,
            action_by_workflow: HashMap::new(),
            next_action_id: 1,
            next_step_id: 1,
        }
    }
}

pub(super) fn aggregate_tool_log_status(steps: &[ToolLogStep]) -> ToolLogStatus {
    if steps.is_empty()
        || steps
            .iter()
            .any(|step| step.status == ToolLogStatus::Running)
    {
        ToolLogStatus::Running
    } else if steps
        .iter()
        .any(|step| step.status == ToolLogStatus::Failed)
    {
        ToolLogStatus::Failed
    } else if steps
        .iter()
        .any(|step| step.status == ToolLogStatus::Success)
    {
        ToolLogStatus::Success
    } else if steps
        .iter()
        .any(|step| step.status == ToolLogStatus::Recovered)
    {
        ToolLogStatus::Recovered
    } else {
        ToolLogStatus::Skipped
    }
}

pub(super) fn current_log_timestamp() -> String {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let seconds_in_day = elapsed % 86_400;
    let hour = seconds_in_day / 3_600;
    let minute = (seconds_in_day % 3_600) / 60;
    let second = seconds_in_day % 60;
    format!("{hour:02}:{minute:02}:{second:02}")
}
