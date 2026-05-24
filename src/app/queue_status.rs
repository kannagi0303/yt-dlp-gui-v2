use crate::domain::{
    CompletedSelection, DownloadSelection, MetadataState, QueueItem, WorkflowKind, WorkflowState,
};

pub(super) fn selection_matches_completed(
    selection: &DownloadSelection,
    completed: &CompletedSelection,
) -> bool {
    selection.video_selector == completed.video_selector
        && selection.audio_selector == completed.audio_selector
        && selection.subtitle_selector == completed.subtitle_selector
        && selection.file_name == completed.file_name
        && selection.download_sections == completed.download_sections
        && selection.write_thumbnail == completed.write_thumbnail
        && selection.embed_thumbnail == completed.embed_thumbnail
        && selection.write_subtitles == completed.write_subtitles
        && selection.embed_subtitles == completed.embed_subtitles
        && selection.write_chapters == completed.write_chapters
        && selection.embed_chapters == completed.embed_chapters
}

pub(super) fn item_latest_download_state(item: &QueueItem) -> Option<WorkflowState> {
    item.workflows
        .iter()
        .rev()
        .find(|run| run.kind == WorkflowKind::DownloadMedia)
        .map(|run| run.state)
}

pub(super) fn item_can_enter_download_queue(item: &QueueItem) -> bool {
    if item.source_url.trim().is_empty() {
        return false;
    }

    match item_latest_download_state(item) {
        Some(state) => is_restartable_download_state(state),
        None => true,
    }
}

pub(super) fn is_pending_download_state(state: WorkflowState) -> bool {
    matches!(state, WorkflowState::Queued)
}

pub(super) fn is_restartable_download_state(state: WorkflowState) -> bool {
    matches!(
        state,
        WorkflowState::Queued
            | WorkflowState::Failed
            | WorkflowState::Finished
            | WorkflowState::Cancelled
    )
}

pub(super) enum QueueSummaryBucket {
    Queued,
    Completed,
    Failed,
}

pub(super) fn item_summary_bucket(item: &QueueItem) -> QueueSummaryBucket {
    if item.workflows.iter().any(|run| {
        matches!(
            run.kind,
            WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia | WorkflowKind::PostProcess
        ) && matches!(run.state, WorkflowState::Queued | WorkflowState::Running)
    }) {
        return QueueSummaryBucket::Queued;
    }

    match item_latest_download_state(item) {
        Some(WorkflowState::Failed) => return QueueSummaryBucket::Failed,
        Some(WorkflowState::Finished) if item.last_error.is_none() => {
            return QueueSummaryBucket::Completed;
        }
        _ => {}
    }

    if item.last_error.is_some() || matches!(item.metadata_state, MetadataState::Failed(_)) {
        return QueueSummaryBucket::Failed;
    }

    QueueSummaryBucket::Queued
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemTitleVisualState {
    Default,
    Pending,
    Ready,
    Completed,
    Failed,
}

#[derive(Default)]
pub struct QueueSummary {
    pub total: usize,
    pub queued: usize,
    pub completed: usize,
    pub failed: usize,
}
