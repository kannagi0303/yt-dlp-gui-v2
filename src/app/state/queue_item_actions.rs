use super::*;

impl AppState {
    pub(super) fn queue_item_index_by_id(&self, item_id: QueueItemId) -> Option<usize> {
        self.queue_items.iter().position(|item| item.id == item_id)
    }

    pub(super) fn queue_item_by_id(&self, item_id: QueueItemId) -> Option<&QueueItem> {
        self.queue_items.iter().find(|item| item.id == item_id)
    }

    pub(super) fn queue_item_mut_by_id(&mut self, item_id: QueueItemId) -> Option<&mut QueueItem> {
        self.queue_items.iter_mut().find(|item| item.id == item_id)
    }

    pub(super) fn should_use_cookies_for_item(&self, item_id: QueueItemId) -> bool {
        self.queue_item_by_id(item_id)
            .map(|item| item.selection.use_cookies)
            .unwrap_or(false)
    }

    pub(super) fn mark_download_preflight_failed(&mut self, item_id: QueueItemId, error: &str) {
        if let Some(item) = self.queue_item_mut_by_id(item_id) {
            item.last_error = Some(error.to_owned());
            item.completed_selection = None;
            if let Some(run) = item.workflows.iter_mut().rev().find(|run| {
                run.kind == WorkflowKind::DownloadMedia
                    && matches!(run.state, WorkflowState::Queued | WorkflowState::Failed)
            }) {
                run.state = WorkflowState::Failed;
                run.progress = 0.0;
                run.error = Some(error.to_owned());
            }
        }
    }

    pub(super) fn item_metadata(&self, item_index: usize) -> Option<&VideoMetadata> {
        self.queue_items
            .get(item_index)
            .and_then(QueueItem::metadata)
    }

    pub(super) fn current_picker_metadata(&self) -> &VideoMetadata {
        self.format_picker
            .target_item_id
            .and_then(|index| self.item_metadata(index))
            .unwrap_or(&self.empty_item_preview)
    }

    pub fn item_thumbnail_url(&self, item_index: usize) -> &str {
        self.item_metadata(item_index)
            .map(|metadata| metadata.thumbnail_url.as_str())
            .or_else(|| {
                self.queue_items
                    .get(item_index)
                    .map(|item| item.thumbnail_url.as_str())
            })
            .unwrap_or_default()
    }

    pub fn item_thumbnail_hint(&self, item_index: usize) -> &str {
        self.item_metadata(item_index)
            .map(|metadata| metadata.thumbnail_hint.as_str())
            .or_else(|| {
                self.queue_items
                    .get(item_index)
                    .map(|item| item.thumbnail_hint.as_str())
            })
            .unwrap_or("item.thumbnail")
    }

    pub fn localized_thumbnail_hint<'a>(&self, hint: &'a str) -> std::borrow::Cow<'a, str> {
        match hint {
            "item.thumbnail" => {
                std::borrow::Cow::Borrowed(self.ui_i18n_text_for_key("item.thumbnail"))
            }
            "Thumbnail preview" => {
                std::borrow::Cow::Borrowed(self.ui_i18n_text_for_key("item.thumbnail_preview"))
            }
            _ => std::borrow::Cow::Borrowed(hint),
        }
    }

    pub fn item_duration_text(&self, item_index: usize) -> &str {
        self.item_metadata(item_index)
            .map(|metadata| metadata.duration_text.as_str())
            .or_else(|| {
                self.queue_items
                    .get(item_index)
                    .map(|item| item.duration_text.as_str())
            })
            .unwrap_or_default()
    }

    pub fn poll_thumbnail_work(&mut self, ctx: &eframe::egui::Context) {
        while let Ok(event) = self.thumbnail_result_rx.try_recv() {
            let entry = match event.result {
                Ok(image) => {
                    let texture = ctx.load_texture(
                        thumbnail_texture_id(&event.key),
                        image,
                        eframe::egui::TextureOptions::LINEAR,
                    );
                    ThumbnailCacheEntry::Ready(texture)
                }
                Err(error) => ThumbnailCacheEntry::Failed(error),
            };
            self.thumbnail_cache.insert(event.key, entry);
            ctx.request_repaint();
        }
    }

    pub fn has_loading_thumbnails(&self) -> bool {
        self.thumbnail_cache
            .values()
            .any(|entry| matches!(entry, ThumbnailCacheEntry::Loading))
    }

    pub fn thumbnail_render_source_for_url(
        &mut self,
        ctx: &eframe::egui::Context,
        url: &str,
    ) -> ThumbnailRenderSource {
        let url = url.trim();
        if url.is_empty() {
            return ThumbnailRenderSource::None;
        }

        let Some(proxy_url) = self.tool_paths.effective_proxy_url().map(str::to_owned) else {
            return ThumbnailRenderSource::DirectUrl;
        };

        self.thumbnail_render_source_with_proxy(ctx, url, proxy_url)
    }

    pub fn single_thumbnail_render_source_for_url(
        &mut self,
        ctx: &eframe::egui::Context,
        url: &str,
    ) -> ThumbnailRenderSource {
        let url = url.trim();
        if url.is_empty() {
            return ThumbnailRenderSource::None;
        }

        let proxy_url = self
            .tool_paths
            .effective_proxy_url()
            .map(str::to_owned)
            .unwrap_or_default();
        self.thumbnail_render_source_with_proxy(ctx, url, proxy_url)
    }

    pub(super) fn thumbnail_render_source_with_proxy(
        &mut self,
        ctx: &eframe::egui::Context,
        url: &str,
        proxy_url: String,
    ) -> ThumbnailRenderSource {
        if !thumbnail_needs_memory_loader(url) {
            return ThumbnailRenderSource::DirectUrl;
        }

        self.poll_thumbnail_work(ctx);
        let no_check_certificates = self.tool_paths.no_check_certificates;
        let key = thumbnail_cache_key(url, &proxy_url, no_check_certificates);
        match self.thumbnail_cache.get(&key) {
            Some(ThumbnailCacheEntry::Ready(texture)) => {
                ThumbnailRenderSource::Texture(texture.clone())
            }
            Some(ThumbnailCacheEntry::Loading) => {
                ctx.request_repaint_after(Duration::from_millis(250));
                ThumbnailRenderSource::Loading
            }
            Some(ThumbnailCacheEntry::Failed(error)) => {
                ThumbnailRenderSource::Failed(error.clone())
            }
            None => {
                self.thumbnail_cache
                    .insert(key.clone(), ThumbnailCacheEntry::Loading);
                run_thumbnail_fetch_worker(
                    key,
                    url.to_owned(),
                    proxy_url,
                    no_check_certificates,
                    self.thumbnail_result_tx.clone(),
                );
                ctx.request_repaint_after(Duration::from_millis(250));
                ThumbnailRenderSource::Loading
            }
        }
    }

    pub fn has_active_work(&self) -> bool {
        self.is_adding_batch
            || !self.active_workflows.is_empty()
            || self.music.music_playback.is_some()
            || self.queue_items.iter().any(|item| {
                matches!(
                    item.compact_music_state,
                    Some(CompactMusicState::Resolving | CompactMusicState::Buffering)
                )
            })
            || self.component_update_snapshot.running
            || self.cookie_rescue.is_running()
    }

    pub fn save_thumbnail_url_to_path(&mut self, url: &str, path: &Path) -> Result<(), String> {
        let url = url.trim();
        if url.is_empty() {
            return Err("Thumbnail load failed: empty URL".to_owned());
        }

        let proxy_url = self
            .tool_paths
            .effective_proxy_url()
            .map(str::to_owned)
            .unwrap_or_default();
        let bytes = fetch_thumbnail_bytes(url, &proxy_url, self.tool_paths.no_check_certificates)?;
        Self::save_thumbnail_bytes_as(&bytes, path)?;
        let display_path = path.display().to_string();
        self.last_action = i18n::format_fixed_english(
            "Thumbnail saved: {path}",
            &[("{path}", display_path.as_str())],
        );
        Ok(())
    }

    pub(super) fn save_thumbnail_bytes_as(bytes: &[u8], path: &Path) -> Result<(), String> {
        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            fs::write(path, bytes).map_err(|error| format!("Could not save thumbnail: {error}"))?;
            return Ok(());
        };

        match extension.trim().to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" => Self::save_thumbnail_as_jpeg(bytes, path),
            "png" => Self::save_thumbnail_as_image_format(bytes, path, image::ImageFormat::Png),
            "webp" => Self::save_thumbnail_as_image_format(bytes, path, image::ImageFormat::WebP),
            _ => {
                fs::write(path, bytes).map_err(|error| format!("Could not save thumbnail: {error}"))
            }
        }
    }

    pub(super) fn save_thumbnail_as_jpeg(bytes: &[u8], path: &Path) -> Result<(), String> {
        let image = image::load_from_memory(bytes)
            .map_err(|error| format!("Could not decode thumbnail: {error}"))?;
        let rgb = image.to_rgb8();
        let mut file =
            fs::File::create(path).map_err(|error| format!("Could not save thumbnail: {error}"))?;
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, 92);
        encoder
            .encode_image(&rgb)
            .map_err(|error| format!("Could not encode thumbnail: {error}"))
    }

    pub(super) fn save_thumbnail_as_image_format(
        bytes: &[u8],
        path: &Path,
        format: image::ImageFormat,
    ) -> Result<(), String> {
        let image = image::load_from_memory(bytes)
            .map_err(|error| format!("Could not decode thumbnail: {error}"))?;
        let mut file =
            fs::File::create(path).map_err(|error| format!("Could not save thumbnail: {error}"))?;
        image
            .write_to(&mut file, format)
            .map_err(|error| format!("Could not encode thumbnail: {error}"))
    }
}
