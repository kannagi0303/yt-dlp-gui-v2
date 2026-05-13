use std::sync::mpsc::Sender;
use std::thread;

use crate::infrastructure::{
    DependencyTool, ToolInstallCancelHandle, ToolInstallProgress, ToolInstallStage,
    install_dependency_tool_with_progress_using_proxy,
};

pub enum ToolInstallEvent {
    Progress(ToolInstallProgress),
    Finished {
        tool: DependencyTool,
        result: Result<String, String>,
    },
}

pub fn run_tool_install_worker(
    tool: DependencyTool,
    proxy_url: Option<String>,
    tx: Sender<ToolInstallEvent>,
) -> ToolInstallCancelHandle {
    let cancel_handle = ToolInstallCancelHandle::new();
    let cancel_token = cancel_handle.token();
    thread::spawn(move || {
        let progress_tx = tx.clone();
        let result = install_dependency_tool_with_progress_using_proxy(
            tool,
            proxy_url,
            Some(cancel_token),
            |progress| {
                let _ = progress_tx.send(ToolInstallEvent::Progress(progress));
            },
        )
        .map(|installed| installed.path.display().to_string())
        .map_err(|error| {
            let _ = tx.send(ToolInstallEvent::Progress(ToolInstallProgress {
                tool,
                stage: ToolInstallStage::Failed,
                percent: None,
                message: error.clone(),
            }));
            error
        });
        let _ = tx.send(ToolInstallEvent::Finished { tool, result });
    });
    cancel_handle
}
