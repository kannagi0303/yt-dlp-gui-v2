use std::borrow::Cow;
use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui::{self, Color32, RichText, Sense, Stroke, Ui};
use egui_commonmark::CommonMarkViewer;

use crate::app::state::{AboutDetailTarget, AppState};
use crate::infrastructure::{
    ComponentOwnership, ComponentUpdateEntry, ComponentUpdateStatus, ManagedComponentId,
};

use super::common::icon_text_button;
use super::semantic_ui_metrics;
use crate::app::widgets::icon::AppIcon;

// ABOUT PAINTER RULES — this file is not a layout owner.
//
// The About page layout lives in `about_tab_template.rs`.
// Row/cell UiElement placement lives in `about_tab_controls.rs`.
// Functions in this file receive one already allocated slot and only draw the
// widget/text/Markdown for that slot.
//
// Forbidden here:
// - rebuilding component rows with `ui.horizontal` or manual spacer math;
// - owning left/right/center alignment for component columns;
// - creating another taffy/template root;
// - adding local i18n fallback dictionaries;
// - hand-calculated offset/padding fixes such as fixed rect shrink, baseline
//   nudges, or width subtraction.  About cells can be squeezed to zero by the
//   template, so every slot-local size must be derived from allocated Rects and
//   clamped before passing it back into egui.
//
// 每次處理中，如有發現不符合地方應強力修正；尤其不要把欄位對齊退回
// painter 內部；若需要改欄位，先改 about_tab_template.rs / about_tab_controls.rs。

pub(super) fn render_about_tab(ui: &mut Ui, state: &mut AppState) {
    super::about_tab_template::render_about_tab_template(ui, state);
}

pub(super) fn about_component_ids(state: &AppState) -> Vec<ManagedComponentId> {
    [
        ManagedComponentId::App,
        ManagedComponentId::YtDlp,
        ManagedComponentId::Deno,
        ManagedComponentId::Ffmpeg,
        ManagedComponentId::Aria2c,
    ]
    .into_iter()
    .filter(|id| {
        if *id != ManagedComponentId::Aria2c {
            return true;
        }
        should_show_aria2c(&selected_entry_for_id(state, *id))
    })
    .collect()
}

pub(super) fn about_current_version_width(
    ui: &Ui,
    state: &AppState,
    ids: &[ManagedComponentId],
) -> f32 {
    let versions: Vec<ComponentVersionCells> = ids
        .iter()
        .copied()
        .map(|id| component_version_cells(&selected_entry_for_id(state, id)))
        .collect();
    let current: Vec<&str> = versions
        .iter()
        .map(|version| version.current.as_str())
        .collect();
    semantic_ui_metrics::settings_form_label_column_width_for_visible_texts(ui, &current)
}

pub(super) fn about_latest_version_width(
    ui: &Ui,
    state: &AppState,
    ids: &[ManagedComponentId],
) -> f32 {
    let versions: Vec<ComponentVersionCells> = ids
        .iter()
        .copied()
        .map(|id| component_version_cells(&selected_entry_for_id(state, id)))
        .collect();
    let latest: Vec<&str> = versions
        .iter()
        .map(|version| version.latest.as_str())
        .collect();
    semantic_ui_metrics::settings_form_label_column_width_for_visible_texts(ui, &latest)
}

pub(super) fn about_arrow_width(ui: &Ui) -> f32 {
    semantic_ui_metrics::settings_form_label_column_width_for_visible_texts(ui, &["→"])
}

pub(super) fn about_action_slot_width(
    ui: &Ui,
    state: &AppState,
    ids: &[ManagedComponentId],
) -> f32 {
    let button_width = action_button_width(ui, state);
    let status_width = about_status_width(ui, state, ids);
    button_width.max(status_width)
}

pub(super) fn about_last_check_width(ui: &Ui, state: &AppState) -> f32 {
    // Fixed width keeps the header row on one line. An `auto(...)` slot can be
    // measured too narrow by taffy and make CJK labels wrap vertically.
    semantic_ui_metrics::settings_form_label_column_width_for_visible_texts(
        ui,
        &[last_check_line(state).as_str()],
    ) + ui.spacing().item_spacing.x
}

pub(super) fn about_check_button_width(ui: &Ui, state: &AppState) -> f32 {
    semantic_ui_metrics::standard_icon_text_button_width_for_visible_text(
        ui,
        state.ui_i18n_text_for_key("about.check_updates"),
    )
}

pub(super) fn about_update_all_button_width(ui: &Ui, state: &AppState) -> f32 {
    semantic_ui_metrics::standard_icon_text_button_width_for_visible_text(
        ui,
        state.ui_i18n_text_for_key("about.update_all"),
    )
}

pub(super) fn render_header_lead(ui: &mut Ui, state: &AppState) {
    ui.label(RichText::new(state.ui_i18n_text_for_key("about.tools")).strong());
}

pub(super) fn render_last_check_inline(ui: &mut Ui, state: &AppState) {
    ui.label(RichText::new(last_check_line(state)).color(subtle_text_color(ui)));
}

pub(super) fn render_check_updates_button(ui: &mut Ui, state: &mut AppState) {
    let running = state.component_update_running();
    let button_size = slot_size(ui);
    if ui
        .add_enabled(
            !running,
            icon_text_button(
                ui,
                AppIcon::Magnify,
                state.ui_i18n_text_for_key("about.check_updates"),
            )
            .min_size(button_size),
        )
        .clicked()
    {
        state.check_component_updates();
    }
}

pub(super) fn render_update_all_button(ui: &mut Ui, state: &mut AppState) {
    let running = state.component_update_running();
    let button_size = slot_size(ui);
    if ui
        .add_enabled(
            !running,
            icon_text_button(
                ui,
                AppIcon::Download,
                state.ui_i18n_text_for_key("about.update_all"),
            )
            .min_size(button_size),
        )
        .clicked()
    {
        state.update_all_managed_components();
    }
}

pub(super) fn render_component_name_cell(
    ui: &mut Ui,
    state: &mut AppState,
    id: ManagedComponentId,
) {
    selectable_component_text_cell(
        ui,
        state,
        id,
        "name",
        component_display_name(id),
        ui.visuals().text_color(),
        true,
    );
}

pub(super) fn render_component_current_version_cell(
    ui: &mut Ui,
    state: &mut AppState,
    id: ManagedComponentId,
) {
    let versions = component_version_cells(&selected_entry_for_id(state, id));
    selectable_component_text_cell(
        ui,
        state,
        id,
        "current",
        &versions.current,
        ui.visuals().text_color(),
        false,
    );
}

pub(super) fn render_component_arrow_cell(
    ui: &mut Ui,
    state: &mut AppState,
    id: ManagedComponentId,
) {
    let versions = component_version_cells(&selected_entry_for_id(state, id));
    selectable_component_text_cell(
        ui,
        state,
        id,
        "arrow",
        if versions.show_arrow { "→" } else { "" },
        ui.visuals().text_color(),
        false,
    );
}

pub(super) fn render_component_latest_version_cell(
    ui: &mut Ui,
    state: &mut AppState,
    id: ManagedComponentId,
) {
    let versions = component_version_cells(&selected_entry_for_id(state, id));
    selectable_component_text_cell(
        ui,
        state,
        id,
        "latest",
        &versions.latest,
        ui.visuals().text_color(),
        false,
    );
}

pub(super) fn render_component_action_cell(
    ui: &mut Ui,
    state: &mut AppState,
    id: ManagedComponentId,
) {
    let entry = selected_entry_for_id(state, id);
    if shows_component_action_button(&entry) {
        render_component_action(ui, state, &entry);
    } else {
        let status = status_text(state, &entry);
        let color = subtle_text_color(ui);
        selectable_component_text_cell(ui, state, id, "status", &status, color, false);
    }
}

pub(super) fn render_component_row_overlay(
    ui: &mut Ui,
    state: &mut AppState,
    id: ManagedComponentId,
) {
    let rect = ui.max_rect();
    let response = ui.interact(
        rect,
        ui.id().with(("about-component-row-overlay", id)),
        Sense::click(),
    );
    let fill = interactive_row_fill(ui, state, id, response.hovered());
    if fill != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, 0.0, fill);
    }
    if response.clicked() {
        state.select_about_detail(detail_target_for_id(id));
    }
}

pub(super) fn render_release_notes(ui: &mut Ui, state: &mut AppState) {
    let entry = selected_detail_entry(state);
    let raw_markdown = entry
        .release_notes_markdown
        .as_deref()
        .unwrap_or_else(|| state.ui_i18n_text_for_key("about.no_release_notes_loaded"));
    let markdown = strip_legacy_release_notes_summary(raw_markdown);

    let stroke = Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color);
    egui::Frame::NONE
        .stroke(stroke)
        .inner_margin(egui::Margin::symmetric(6, 4))
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("about-release-notes-scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let width = ui.available_width().max(1.0) as usize;
                    CommonMarkViewer::new()
                        .default_width(Some(width))
                        .max_image_width(Some(width))
                        .show(ui, &mut state.about_markdown_cache, markdown.as_ref());
                });
        });
}

fn strip_legacy_release_notes_summary(markdown: &str) -> Cow<'_, str> {
    let trimmed = markdown.trim_start();
    if !trimmed.starts_with("# ") {
        return Cow::Borrowed(markdown);
    }

    let Some(separator_index) = trimmed.find("\n---") else {
        return Cow::Borrowed(markdown);
    };
    let header = &trimmed[..separator_index];
    if !header.contains("Current:") || !header.contains("Latest:") {
        return Cow::Borrowed(markdown);
    }

    let mut rest = &trimmed[separator_index..];
    if let Some(after_separator) = rest.strip_prefix("\n---\n") {
        rest = after_separator;
    } else if let Some(after_separator) = rest.strip_prefix("\n---\r\n") {
        rest = after_separator;
    } else if let Some(after_separator) = rest.strip_prefix("\n---") {
        rest = after_separator;
    }
    Cow::Owned(rest.trim_start_matches(['\r', '\n']).to_owned())
}

fn selectable_component_text_cell(
    ui: &mut Ui,
    state: &mut AppState,
    id: ManagedComponentId,
    cell_id: &'static str,
    text: &str,
    color: Color32,
    strong: bool,
) {
    let rect = ui.max_rect();
    let mut text = RichText::new(text).color(color);
    if strong {
        text = text.strong();
    }
    ui.add(egui::Label::new(text).selectable(false));

    let response = ui.interact(
        rect,
        ui.id().with(("about-component-row-cell", id, cell_id)),
        Sense::click(),
    );
    if response.clicked() {
        state.select_about_detail(detail_target_for_id(id));
    }
}

fn slot_size(ui: &Ui) -> egui::Vec2 {
    egui::vec2(
        ui.available_width().max(0.0),
        ui.available_height().max(0.0),
    )
}

fn interactive_row_fill(
    ui: &Ui,
    state: &AppState,
    id: ManagedComponentId,
    hovered: bool,
) -> Color32 {
    if is_selected_entry(state, id) {
        ui.visuals().selection.bg_fill.linear_multiply(0.55)
    } else if hovered {
        ui.visuals().widgets.hovered.weak_bg_fill
    } else {
        Color32::TRANSPARENT
    }
}

fn should_show_aria2c(entry: &ComponentUpdateEntry) -> bool {
    entry.local_version.is_some()
        || matches!(
            entry.ownership,
            ComponentOwnership::ManagedPortable | ComponentOwnership::External
        )
        || matches!(
            entry.status,
            ComponentUpdateStatus::Downloading
                | ComponentUpdateStatus::Staged
                | ComponentUpdateStatus::PendingRestart
                | ComponentUpdateStatus::Applying
                | ComponentUpdateStatus::Installed
                | ComponentUpdateStatus::UpdateAvailable
        )
}

fn render_component_action(ui: &mut Ui, state: &mut AppState, entry: &ComponentUpdateEntry) {
    let running = state.component_update_running();
    let action_size = slot_size(ui);
    if entry.id == ManagedComponentId::App && entry.status == ComponentUpdateStatus::PendingRestart
    {
        if ui
            .add_enabled(
                !running,
                egui::Button::new(state.ui_i18n_text_for_key("about.restart"))
                    .min_size(action_size),
            )
            .clicked()
        {
            match state.restart_to_apply_app_update() {
                Ok(()) => ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close),
                Err(error) => state.last_action = error,
            }
        }
        return;
    }

    if should_show_update_button(entry) {
        if ui
            .add_enabled(
                !running,
                egui::Button::new(action_label(state, entry)).min_size(action_size),
            )
            .clicked()
        {
            state.update_component(entry.id);
        }
    } else {
        ui.label(RichText::new(status_text(state, entry)).color(subtle_text_color(ui)));
    }
}

fn selected_detail_entry(state: &AppState) -> ComponentUpdateEntry {
    let selected = match state.about_detail_target {
        AboutDetailTarget::App => ManagedComponentId::App,
        AboutDetailTarget::Tool(id) => id,
    };
    selected_entry_for_id(state, selected)
}

fn selected_entry_for_id(state: &AppState, id: ManagedComponentId) -> ComponentUpdateEntry {
    state
        .component_update_snapshot
        .entry(id)
        .cloned()
        .unwrap_or_else(|| ComponentUpdateEntry::new(id))
}

fn is_selected_entry(state: &AppState, id: ManagedComponentId) -> bool {
    match state.about_detail_target {
        AboutDetailTarget::App => id == ManagedComponentId::App,
        AboutDetailTarget::Tool(selected) => id == selected,
    }
}

fn detail_target_for_id(id: ManagedComponentId) -> AboutDetailTarget {
    if id == ManagedComponentId::App {
        AboutDetailTarget::App
    } else {
        AboutDetailTarget::Tool(id)
    }
}

fn component_display_name(id: ManagedComponentId) -> &'static str {
    match id {
        ManagedComponentId::App => "yt-dlp-gui",
        ManagedComponentId::YtDlp => "yt-dlp",
        ManagedComponentId::Deno => "Deno",
        ManagedComponentId::Ffmpeg => "FFmpeg",
        ManagedComponentId::Aria2c => "Aria2",
    }
}

fn action_button_width(ui: &Ui, state: &AppState) -> f32 {
    [
        state.ui_i18n_text_for_key("about.update"),
        state.ui_i18n_text_for_key("about.install"),
        state.ui_i18n_text_for_key("about.restart"),
    ]
    .into_iter()
    .map(|text| semantic_ui_metrics::standard_action_button_width_for_visible_text(ui, text))
    .fold(0.0, f32::max)
}

fn about_status_width(ui: &Ui, state: &AppState, ids: &[ManagedComponentId]) -> f32 {
    let labels: Vec<String> = ids
        .iter()
        .copied()
        .map(|id| status_text(state, &selected_entry_for_id(state, id)))
        .collect();
    let label_refs: Vec<&str> = labels.iter().map(String::as_str).collect();
    semantic_ui_metrics::settings_form_label_column_width_for_visible_texts(ui, &label_refs)
}

struct ComponentVersionCells {
    current: String,
    latest: String,
    show_arrow: bool,
}

fn component_version_cells(entry: &ComponentUpdateEntry) -> ComponentVersionCells {
    let current = entry
        .local_version
        .as_deref()
        .and_then(|version| display_component_version(entry.id, version))
        .unwrap_or_else(|| "-".to_owned());
    let latest_available = should_show_latest_version(entry);
    let latest = latest_available
        .then(|| {
            entry
                .latest_version
                .as_deref()
                .and_then(|version| display_component_version(entry.id, version))
        })
        .flatten()
        .unwrap_or_else(|| "-".to_owned());
    let show_arrow = latest_available
        && current.trim() != "-"
        && latest.trim() != "-"
        && !current.trim().is_empty()
        && !latest.trim().is_empty();

    ComponentVersionCells {
        current,
        latest,
        show_arrow,
    }
}

fn display_component_version(id: ManagedComponentId, version: &str) -> Option<String> {
    let version = version.trim();
    if version.is_empty() || version == "-" {
        return None;
    }

    if let Some(date) = dotted_date_from_yyyymmdd_tail(version) {
        return Some(date);
    }
    if let Some(date) = dotted_date_from_iso8601_prefix(version) {
        return Some(date);
    }

    let normalized = version
        .strip_prefix('v')
        .or_else(|| version.strip_prefix("release-"))
        .unwrap_or(version)
        .trim();

    if id == ManagedComponentId::Ffmpeg && normalized.eq_ignore_ascii_case("latest") {
        return None;
    }

    Some(normalized.to_owned())
}

fn dotted_date_from_iso8601_prefix(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes.len() < 10 {
        return None;
    }
    if bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(u8::is_ascii_digit)
    {
        return Some(format!(
            "{}.{}.{}",
            &value[0..4],
            &value[5..7],
            &value[8..10]
        ));
    }
    None
}

fn dotted_date_from_yyyymmdd_tail(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes.len() < 8 {
        return None;
    }
    for start in (0..=bytes.len() - 8).rev() {
        let end = start + 8;
        if !bytes[start..end].iter().all(u8::is_ascii_digit) {
            continue;
        }
        if start > 0 && bytes[start - 1].is_ascii_digit() {
            continue;
        }
        if end < bytes.len() && bytes[end].is_ascii_digit() {
            continue;
        }
        let date = &value[start..end];
        return Some(format!("{}.{}.{}", &date[0..4], &date[4..6], &date[6..8]));
    }
    None
}

fn should_show_latest_version(entry: &ComponentUpdateEntry) -> bool {
    matches!(
        entry.status,
        ComponentUpdateStatus::UpdateAvailable
            | ComponentUpdateStatus::Missing
            | ComponentUpdateStatus::PendingRestart
            | ComponentUpdateStatus::Downloading
            | ComponentUpdateStatus::Staged
            | ComponentUpdateStatus::Applying
            | ComponentUpdateStatus::Failed
    ) || (entry.id == ManagedComponentId::App && entry.local_version.is_none())
}

fn should_show_update_button(entry: &ComponentUpdateEntry) -> bool {
    matches!(
        entry.status,
        ComponentUpdateStatus::UpdateAvailable | ComponentUpdateStatus::Missing
    ) && matches!(
        entry.ownership,
        ComponentOwnership::ManagedPortable | ComponentOwnership::Missing
    )
}

fn shows_component_action_button(entry: &ComponentUpdateEntry) -> bool {
    (entry.id == ManagedComponentId::App && entry.status == ComponentUpdateStatus::PendingRestart)
        || should_show_update_button(entry)
}

fn action_label<'a>(state: &'a AppState, entry: &ComponentUpdateEntry) -> &'a str {
    if entry.status == ComponentUpdateStatus::Missing {
        state.ui_i18n_text_for_key("about.install")
    } else {
        state.ui_i18n_text_for_key("about.update")
    }
}

pub(super) fn last_check_line(state: &AppState) -> String {
    if state.component_update_snapshot.running {
        return state.ui_i18n_text_for_key("about.running").to_owned();
    }
    if let Some(time) = state.component_update_snapshot.checked_at_unix {
        format!(
            "{} {}",
            state.ui_i18n_text_for_key("about.last_check"),
            format_last_check_value(state, time)
        )
    } else {
        state.ui_i18n_text_for_key("about.never_checked").to_owned()
    }
}

fn format_last_check_value(state: &AppState, checked_at_unix: u64) -> String {
    let now = now_unix_seconds().unwrap_or(checked_at_unix);
    let age = now.saturating_sub(checked_at_unix);
    let minute = 60;
    let hour = 60 * minute;
    let day = 24 * hour;

    if age < hour {
        let minutes = (age / minute).max(1).to_string();
        return state.ui_i18n_text_with_replacements(
            "about.relative.minutes",
            &[("{count}", minutes.as_str())],
        );
    }
    if age < day {
        let hours = (age / hour).max(1);
        return if hours == 1 {
            state.ui_i18n_text_for_key("about.relative.hour").to_owned()
        } else {
            let hours = hours.to_string();
            state.ui_i18n_text_with_replacements(
                "about.relative.hours",
                &[("{count}", hours.as_str())],
            )
        };
    }
    if age < 7 * day {
        let days = (age / day).max(1);
        return if days == 1 {
            state.ui_i18n_text_for_key("about.relative.day").to_owned()
        } else {
            let days = days.to_string();
            state.ui_i18n_text_with_replacements(
                "about.relative.days",
                &[("{count}", days.as_str())],
            )
        };
    }

    format_unix_date_utc(checked_at_unix)
}

fn now_unix_seconds() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}

fn format_unix_date_utc(unix_seconds: u64) -> String {
    let days = (unix_seconds / 86_400) as i64;
    let (year, month, day) = civil_from_days_utc(days);
    format!("{year:04}-{month:02}-{day:02}")
}

fn civil_from_days_utc(days_since_unix_epoch: i64) -> (i32, u32, u32) {
    // Howard Hinnant civil calendar conversion. Kept local to avoid adding a
    // date/time dependency for one compact relative timestamp in the About page.
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    (year as i32, m as u32, d as u32)
}

fn subtle_text_color(ui: &Ui) -> Color32 {
    ui.visuals().text_color().linear_multiply(0.68)
}

fn status_text(state: &AppState, entry: &ComponentUpdateEntry) -> String {
    match entry.status {
        ComponentUpdateStatus::Unknown => state
            .ui_i18n_text_for_key("about.status.unknown")
            .to_owned(),
        ComponentUpdateStatus::Checking => state
            .ui_i18n_text_for_key("about.status.checking")
            .to_owned(),
        ComponentUpdateStatus::UpToDate => state
            .ui_i18n_text_for_key("about.status.up_to_date")
            .to_owned(),
        ComponentUpdateStatus::UpdateAvailable => state
            .ui_i18n_text_for_key("about.status.update_available")
            .to_owned(),
        ComponentUpdateStatus::Missing => state
            .ui_i18n_text_for_key("about.status.missing")
            .to_owned(),
        ComponentUpdateStatus::Downloading => entry
            .progress
            .map(|percent| {
                let percent = percent.to_string();
                state.ui_i18n_text_with_replacements(
                    "about.status.downloading_percent",
                    &[("{percent}", percent.as_str())],
                )
            })
            .unwrap_or_else(|| {
                state
                    .ui_i18n_text_for_key("about.status.downloading")
                    .to_owned()
            }),
        ComponentUpdateStatus::Staged => {
            state.ui_i18n_text_for_key("about.status.staged").to_owned()
        }
        ComponentUpdateStatus::PendingRestart => state
            .ui_i18n_text_for_key("about.status.pending_restart")
            .to_owned(),
        ComponentUpdateStatus::Applying if !entry.message.trim().is_empty() => {
            state.localize_message(&entry.message)
        }
        ComponentUpdateStatus::Applying => state
            .ui_i18n_text_for_key("about.status.applying")
            .to_owned(),
        ComponentUpdateStatus::Installed => state
            .ui_i18n_text_for_key("about.status.installed")
            .to_owned(),
        ComponentUpdateStatus::Skipped => state
            .ui_i18n_text_for_key("about.status.skipped")
            .to_owned(),
        ComponentUpdateStatus::Failed => {
            state.ui_i18n_text_for_key("about.status.failed").to_owned()
        }
    }
}
