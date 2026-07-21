use super::*;

impl AppState {
    pub fn runtime_log_entries(&self) -> &VecDeque<String> {
        &self.logs.runtime_log
    }
    pub fn tool_log_actions(&self) -> &VecDeque<ToolLogAction> {
        &self.logs.tool_logs
    }
    pub fn log_viewer_selected_step(&self) -> Option<u64> {
        self.logs.viewer_selected_step
    }
    pub fn log_viewer_expanded_action(&self) -> Option<u64> {
        self.logs.viewer_expanded_action
    }
    pub fn set_log_viewer_selected_step(&mut self, step_id: Option<u64>) {
        self.logs.viewer_selected_step = step_id;
    }
    pub fn set_log_viewer_expanded_action(&mut self, action_id: Option<u64>) {
        self.logs.viewer_expanded_action = action_id;
    }
    pub fn push_runtime_log(&mut self, message: impl Into<String>) {
        let message = message.into();
        if message.trim().is_empty() {
            return;
        }
        self.logs.runtime_log.push_back(message);
        while self.logs.runtime_log.len() > 20 {
            self.logs.runtime_log.pop_front();
        }
    }
    pub fn push_tool_log_action(
        &mut self,
        mode: impl Into<String>,
        action: impl Into<String>,
    ) -> u64 {
        let id = self.logs.next_action_id;
        self.logs.next_action_id = self.logs.next_action_id.saturating_add(1);
        self.logs.tool_logs.push_back(ToolLogAction {
            id,
            timestamp: current_log_timestamp(),
            status: ToolLogStatus::Running,
            mode: mode.into(),
            action: action.into(),
            steps: Vec::new(),
        });
        self.trim_tool_logs();
        id
    }
    pub(super) fn mark_last_failed_tool_log_step_as_recoverable(&mut self, action_id: u64) {
        if let Some(parent) = self
            .logs
            .tool_logs
            .iter_mut()
            .find(|entry| entry.id == action_id)
        {
            if let Some(step) = parent
                .steps
                .iter_mut()
                .rev()
                .find(|step| step.status == ToolLogStatus::Failed)
            {
                step.status = ToolLogStatus::Recovered;
            }
            parent.status = aggregate_tool_log_status(&parent.steps);
        }
    }
    pub fn push_tool_log_step(
        &mut self,
        action_id: u64,
        status: ToolLogStatus,
        tool: impl Into<String>,
        action: impl Into<String>,
        command: impl Into<String>,
    ) -> u64 {
        self.push_tool_log_step_internal(action_id, status, tool, action, command, None, true)
    }
    pub(super) fn push_tool_log_step_with_detail_without_failure_reveal(
        &mut self,
        action_id: u64,
        status: ToolLogStatus,
        tool: impl Into<String>,
        action: impl Into<String>,
        command: impl Into<String>,
        detail: Option<String>,
    ) -> u64 {
        self.push_tool_log_step_internal(action_id, status, tool, action, command, detail, false)
    }
    pub(super) fn push_tool_log_step_internal(
        &mut self,
        action_id: u64,
        status: ToolLogStatus,
        tool: impl Into<String>,
        action: impl Into<String>,
        command: impl Into<String>,
        detail: Option<String>,
        reveal_on_failure: bool,
    ) -> u64 {
        let id = self.logs.next_step_id;
        self.logs.next_step_id = self.logs.next_step_id.saturating_add(1);
        if let Some(parent) = self
            .logs
            .tool_logs
            .iter_mut()
            .find(|entry| entry.id == action_id)
        {
            parent.steps.push(ToolLogStep {
                id,
                status,
                tool: tool.into(),
                action: action.into(),
                command: command.into(),
                detail,
            });
            parent.status = aggregate_tool_log_status(&parent.steps);
            if status == ToolLogStatus::Failed {
                self.logs.viewer_expanded_action = Some(action_id);
                self.logs.viewer_selected_step = Some(id);
                if reveal_on_failure {
                    self.reveal_log_tab_for_tool_failure();
                }
            }
        }
        id
    }
    pub fn workflow_tool_log_action(
        &mut self,
        workflow_id: WorkflowRunId,
        mode: impl Into<String>,
        action: impl Into<String>,
    ) -> u64 {
        if let Some(action_id) = self.logs.action_by_workflow.get(&workflow_id).copied() {
            if self
                .logs
                .tool_logs
                .iter()
                .any(|entry| entry.id == action_id)
            {
                return action_id;
            }
        }
        let action_id = self.push_tool_log_action(mode, action);
        self.logs.action_by_workflow.insert(workflow_id, action_id);
        action_id
    }
    pub fn finish_workflow_tool_log(&mut self, workflow_id: WorkflowRunId) {
        self.logs.action_by_workflow.remove(&workflow_id);
    }
    pub fn enter_log_tab(&mut self) {
        self.config.show_log_tab = true;
        self.active_tab = AppTab::Log;
        self.collapse_tool_log_viewer();
    }
    pub(super) fn reveal_log_tab_for_tool_failure(&mut self) {
        self.config.show_log_tab = true;
        self.active_tab = AppTab::Log;
    }
    fn collapse_tool_log_viewer(&mut self) {
        self.logs.viewer_expanded_action = None;
        self.logs.viewer_selected_step = None;
    }
    fn trim_tool_logs(&mut self) {
        while self.logs.tool_logs.len() > 20 {
            let removed = self.logs.tool_logs.pop_front();
            if let Some(removed) = removed {
                if self.logs.viewer_expanded_action == Some(removed.id) {
                    self.logs.viewer_expanded_action = None;
                }
                if let Some(selected_step) = self.logs.viewer_selected_step {
                    if removed.steps.iter().any(|step| step.id == selected_step) {
                        self.logs.viewer_selected_step = None;
                    }
                }
                self.logs
                    .action_by_workflow
                    .retain(|_, action_id| *action_id != removed.id);
            }
        }
    }
}
