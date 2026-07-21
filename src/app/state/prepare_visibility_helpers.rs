use super::*;

pub(super) fn prepare_dependency_update_status_is_visible(
    status: ComponentUpdateStatus,
    snapshot_running: bool,
    tool_update_running: bool,
    tool_installed: bool,
) -> bool {
    if tool_update_running {
        return true;
    }

    match status {
        ComponentUpdateStatus::Installed | ComponentUpdateStatus::UpToDate => true,
        ComponentUpdateStatus::Failed => !tool_installed,
        ComponentUpdateStatus::Missing | ComponentUpdateStatus::UpdateAvailable => {
            snapshot_running && !tool_installed
        }
        _ => false,
    }
}

pub(super) fn component_update_status_needs_attention_signal(
    status: ComponentUpdateStatus,
) -> bool {
    matches!(
        status,
        ComponentUpdateStatus::UpdateAvailable | ComponentUpdateStatus::PendingRestart
    )
}
