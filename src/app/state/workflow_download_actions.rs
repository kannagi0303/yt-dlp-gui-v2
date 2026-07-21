use super::*;

impl AppState {
    pub(super) fn enqueue_download_ready_items(&mut self) {
        let ready_item_ids = self
            .queue_items
            .iter()
            .filter(|item| !item.source_url.trim().is_empty())
            .filter(|item| match item_latest_download_state(item) {
                None => true,
                Some(
                    WorkflowState::Failed | WorkflowState::Finished | WorkflowState::Cancelled,
                ) => true,
                Some(_) => false,
            })
            .map(|item| item.id)
            .collect::<Vec<_>>();

        for item_id in ready_item_ids {
            let workflow_id = self.alloc_workflow_run_id();
            let Some(item) = self.queue_item_mut_by_id(item_id) else {
                continue;
            };
            reset_item_for_new_work(item, DownloadTargetKind::Normal);
            item.completed_selection = None;
            let mut run = WorkflowRun::new(
                workflow_id,
                WorkflowKind::DownloadMedia,
                ToolKind::YtDlp,
                WorkflowState::Queued,
            );
            run.detail = item.source_url.clone();
            item.workflows.push(run);
        }
    }

    pub(super) fn ensure_queue_item_for_url(&mut self, url: &str) -> QueueItemId {
        let source_key = canonical_queue_source_key(url);
        if let Some(item) = self
            .queue_items
            .iter()
            .find(|item| canonical_queue_source_key(&item.source_url) == source_key)
        {
            return item.id;
        }

        let item = self.build_queue_item_from_url(url);
        let item_id = item.id;
        self.queue_items.push(item);
        self.batch_input_push_unique(url);
        item_id
    }

    pub(super) fn batch_input_push_unique(&mut self, url: &str) {
        let source_key = canonical_queue_source_key(url);
        if self
            .batch_input
            .lines()
            .map(str::trim)
            .any(|line| canonical_queue_source_key(line) == source_key)
        {
            return;
        }
        if !self.batch_input.trim().is_empty() {
            self.batch_input.push('\n');
        }
        self.batch_input.push_str(url);
    }

    pub(super) fn alloc_queue_item_id(&mut self) -> QueueItemId {
        let id = self.next_queue_item_id;
        self.next_queue_item_id += 1;
        id
    }

    pub(super) fn alloc_workflow_run_id(&mut self) -> WorkflowRunId {
        let id = self.next_workflow_run_id;
        self.next_workflow_run_id += 1;
        id
    }

    pub(super) fn register_active_workflow(
        &mut self,
        item_id: QueueItemId,
        workflow_id: WorkflowRunId,
        kind: WorkflowKind,
        tool: ToolKind,
    ) {
        self.active_workflows.insert(
            workflow_id,
            ActiveWorkflow {
                item_id,
                workflow_id,
                kind,
                tool,
                download_child: None,
                cancel_requested: None,
            },
        );
    }

    pub(super) fn attach_active_download_process(
        &mut self,
        workflow_id: WorkflowRunId,
        child_handle: Arc<Mutex<Option<Child>>>,
        cancel_requested: Arc<AtomicBool>,
    ) {
        if let Some(workflow) = self.active_workflows.get_mut(&workflow_id) {
            workflow.download_child = Some(child_handle);
            workflow.cancel_requested = Some(cancel_requested);
        }
    }

    pub(super) fn unregister_active_workflow(&mut self, workflow_id: WorkflowRunId) {
        self.active_workflows.remove(&workflow_id);
    }

    pub(super) fn active_workflow_cancel_requested(&self, workflow_id: WorkflowRunId) -> bool {
        self.active_workflows
            .get(&workflow_id)
            .and_then(|workflow| workflow.cancel_requested.as_ref())
            .is_some_and(|flag| flag.load(Ordering::Relaxed))
    }

    pub(super) fn is_current_download_progress_event(
        &self,
        item_id: QueueItemId,
        workflow_id: WorkflowRunId,
    ) -> bool {
        self.active_workflows
            .get(&workflow_id)
            .is_some_and(|workflow| {
                workflow.item_id == item_id
                    && matches!(
                        workflow.kind,
                        WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia
                    )
            })
    }

    pub(super) fn tool_log_status_for_workflow_step(
        &self,
        workflow_id: WorkflowRunId,
        success: bool,
    ) -> ToolLogStatus {
        if success {
            ToolLogStatus::Success
        } else if self.active_workflow_cancel_requested(workflow_id) {
            ToolLogStatus::Skipped
        } else {
            ToolLogStatus::Failed
        }
    }

    pub(super) fn tool_log_status_for_batch_step(&self, success: bool) -> ToolLogStatus {
        if success {
            ToolLogStatus::Success
        } else if self.is_cancelling_batch_add
            || self
                .batch_add_cancel_requested
                .as_ref()
                .is_some_and(|flag| flag.load(Ordering::Relaxed))
        {
            ToolLogStatus::Skipped
        } else {
            ToolLogStatus::Failed
        }
    }

    pub(super) fn has_running_download_workflow(&self) -> bool {
        self.active_workflows.values().any(|workflow| {
            matches!(
                workflow.kind,
                WorkflowKind::DownloadMedia | WorkflowKind::ExportMedia | WorkflowKind::PostProcess
            )
        })
    }

    pub(super) fn queue_structure_mutation_blocked(&mut self, action: &str) -> bool {
        if self.is_adding_batch {
            self.last_action = format!("Wait for batch add to finish before {action}.");
            return true;
        }

        if !self.active_workflows.is_empty() {
            self.last_action = format!("Wait for current work to finish before {action}.");
            return true;
        }

        false
    }

    pub(super) fn maybe_start_builtin_transcode_post_process(
        &mut self,
        item_id: QueueItemId,
        input_path: &str,
    ) -> bool {
        if !self.config.post_download_conversion_enabled {
            return false;
        }
        let plan = resolve_transcode_plan(&self.config.transcode_intent);
        if !plan.is_executable() {
            return false;
        }
        let Some(profile) = plan.backend_profile else {
            return false;
        };

        let Some(item_index) = self.queue_item_index_by_id(item_id) else {
            return false;
        };
        let title = self.queue_items[item_index].title.clone();
        let workflow_id = self.alloc_workflow_run_id();
        self.register_active_workflow(
            item_id,
            workflow_id,
            WorkflowKind::PostProcess,
            ToolKind::Ffmpeg,
        );

        if let Some(item) = self.queue_items.get_mut(item_index) {
            item.progress.post_process = 0.0;
            let mut run = WorkflowRun::new(
                workflow_id,
                WorkflowKind::PostProcess,
                ToolKind::Ffmpeg,
                WorkflowState::Running,
            );
            run.detail = input_path.to_owned();
            item.workflows.push(run);
        }

        self.last_action = i18n::format_fixed_english(
            "Converting with {profile}: {title}",
            &[("{title}", title.as_str()), ("{profile}", profile.label())],
        );
        self.push_runtime_log(format!(
            "Post-process started: {title} -> {}",
            profile.label()
        ));

        let tool_paths = self.tool_paths.clone();
        let settings = self.config.transcode_intent.clone();
        let tx = self.post_process_result_tx.clone();
        let input_path = input_path.to_owned();
        let temp_root = self.transcode_temp_root_path();
        let child_handle = Arc::new(Mutex::new(None));
        let cancel_requested = Arc::new(AtomicBool::new(false));
        self.attach_active_download_process(
            workflow_id,
            child_handle.clone(),
            cancel_requested.clone(),
        );

        thread::spawn(move || {
            run_builtin_transcode_worker(
                tool_paths,
                settings,
                input_path,
                temp_root,
                item_id,
                workflow_id,
                tx,
                child_handle,
                cancel_requested,
            );
        });

        true
    }

    pub(super) fn start_next_queued_download_after(&mut self, finished_item_id: QueueItemId) {
        if self.has_running_download_workflow() {
            return;
        }

        let Some(next_item_id) = self
            .queue_items
            .iter()
            .find(|item| {
                item.id != finished_item_id
                    && item_latest_download_state(item).is_some_and(is_pending_download_state)
            })
            .map(|item| item.id)
        else {
            return;
        };

        if self.queue_mode_downloads_as_audio() {
            if let Some(choice) = self.music.active_music_download_choice {
                let _ = self.start_music_download_task_at(next_item_id, choice);
                return;
            }
        }

        let emit_json = self
            .queue_item_by_id(next_item_id)
            .is_some_and(|item| !item.metadata_loaded());
        let _ = self.start_download_task_at(next_item_id, emit_json);
    }

    pub fn active_workflow_count(&self) -> usize {
        self.active_workflows.len()
    }

    pub fn item_has_running_workflow(&self, item_id: QueueItemId, kind: WorkflowKind) -> bool {
        self.active_workflows
            .values()
            .any(|workflow| workflow.item_id == item_id && workflow.kind == kind)
    }

    pub fn item_has_cancellable_download_workflow(&self, item_id: QueueItemId) -> bool {
        self.active_workflows.values().any(|workflow| {
            workflow.item_id == item_id
                && matches!(
                    workflow.kind,
                    WorkflowKind::DownloadMedia
                        | WorkflowKind::ExportMedia
                        | WorkflowKind::PostProcess
                )
                && workflow.download_child.is_some()
        })
    }

    pub fn cancel_item_download(&mut self, item_id: QueueItemId) {
        let workflows = self
            .active_workflows
            .values()
            .filter(|workflow| {
                workflow.item_id == item_id
                    && matches!(
                        workflow.kind,
                        WorkflowKind::DownloadMedia
                            | WorkflowKind::ExportMedia
                            | WorkflowKind::PostProcess
                    )
            })
            .map(|workflow| workflow.workflow_id)
            .collect::<Vec<_>>();

        if workflows.is_empty() {
            self.last_action = "There is no download to stop.".to_owned();
            return;
        }

        for workflow_id in workflows {
            self.request_active_download_stop(workflow_id);
        }
        self.last_action = "Stopping download...".to_owned();
    }

    pub(super) fn request_active_download_stop(&self, workflow_id: WorkflowRunId) {
        let Some(workflow) = self.active_workflows.get(&workflow_id) else {
            return;
        };
        let (Some(child_handle), Some(cancel_requested)) = (
            workflow.download_child.as_ref(),
            workflow.cancel_requested.as_ref(),
        ) else {
            return;
        };
        if workflow.kind == WorkflowKind::PostProcess {
            request_post_process_stop(child_handle, cancel_requested);
        } else {
            request_download_stop(child_handle, cancel_requested);
        }
    }

    pub fn cleanup_active_download_processes(&mut self) {
        let workflows = self
            .active_workflows
            .values()
            .filter(|workflow| {
                matches!(
                    workflow.kind,
                    WorkflowKind::DownloadMedia
                        | WorkflowKind::ExportMedia
                        | WorkflowKind::PostProcess
                )
            })
            .map(|workflow| workflow.workflow_id)
            .collect::<Vec<_>>();
        for workflow_id in workflows {
            self.request_active_download_stop(workflow_id);
        }
    }

    pub fn item_is_busy(&self, item_index: usize) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };

        matches!(
            item.metadata_state,
            MetadataState::Queued | MetadataState::Running
        ) || item
            .workflows
            .iter()
            .any(|run| matches!(run.state, WorkflowState::Queued | WorkflowState::Running))
    }

    pub fn item_can_export(&self, item_index: usize, kind: DownloadTargetKind) -> bool {
        let Some(item) = self.queue_items.get(item_index) else {
            return false;
        };
        if !item.metadata_loaded() || self.item_is_busy(item_index) {
            return false;
        }

        match kind {
            DownloadTargetKind::Video => !item.selection.video_selector.trim().is_empty(),
            DownloadTargetKind::Audio => {
                let (_, format_selector) = self.resolve_download_format_selection(
                    &item.selection.video_selector,
                    &item.selection.audio_selector,
                    item.metadata(),
                );
                !format_selector.trim().is_empty()
            }
            DownloadTargetKind::Subtitle => self
                .subtitle_track_by_id(&item.selection.subtitle_selector, item.metadata())
                .is_some(),
            DownloadTargetKind::Normal => false,
        }
    }
}
