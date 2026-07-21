use eframe::egui::{self, TextStyle, Ui};

use crate::app::state::AppState;

use super::measure::{
    WidthRange, max_text_height_for_lines, max_text_width, measured_column_width,
    measured_text_width, text_width, wrapped_text_height,
};

const STANDARD_BUTTON_WIDTH_GUARD_FROM_TEXT_METRICS: f32 = 4.0;
const SETTINGS_SCROLL_CONTENT_TRAILING_SAFE_GAP: f32 = 10.0;
const SETTINGS_FORM_SECTION_BEFORE_TITLE_VERTICAL_SPACING: f32 = 2.0;
const SETTINGS_FORM_SECTION_AFTER_SEPARATOR_VERTICAL_SPACING: f32 = 2.0;
const SETTINGS_FORM_SECTION_AFTER_BODY_VERTICAL_SPACING: f32 = 8.0;
const SETTINGS_FORM_LABEL_COLUMN_HORIZONTAL_PADDING_MULTIPLIER: f32 = 2.0;
const SETTINGS_FORM_ROW_VERTICAL_SPACING_SCALE_FROM_CONTROL_TEXT_DELTA: f32 = 0.16;
const SETTINGS_FORM_ROW_VERTICAL_SPACING_MINIMUM: f32 = 2.0;
const SETTINGS_FORM_ROW_VERTICAL_SPACING_MAXIMUM: f32 = 5.0;
const COOKIE_ACQUISITION_DIALOG_HORIZONTAL_SAFE_MARGIN_FROM_VIEWPORT: f32 = 24.0;
const COOKIE_ACQUISITION_DIALOG_TEXT_WIDTH_EXTRA_ROOM: f32 = 28.0;
const COOKIE_ACQUISITION_DIALOG_ACTION_ROW_WIDTH_GUARD: f32 = 24.0;
const COOKIE_ACQUISITION_DIALOG_FIELD_TO_BODY_VERTICAL_SPACING: f32 = 6.0;

const COOKIE_ACQUISITION_DIALOG_ITEM_HORIZONTAL_SPACING: f32 = 8.0;
const COOKIE_ACQUISITION_DIALOG_ITEM_VERTICAL_SPACING: f32 = 8.0;
const COOKIE_ACQUISITION_DIALOG_BUTTON_HORIZONTAL_PADDING: f32 = 10.0;
const COOKIE_ACQUISITION_DIALOG_BUTTON_VERTICAL_PADDING: f32 = 5.0;
const COOKIE_ACQUISITION_DIALOG_MINIMUM_WIDTH_FOR_MESSAGE_PHASES: f32 = 280.0;
const COOKIE_ACQUISITION_DIALOG_MINIMUM_WIDTH_FOR_START_PHASE: f32 = 360.0;
const COOKIE_ACQUISITION_DIALOG_MAXIMUM_WIDTH: f32 = 520.0;
const ADVANCE_FORM_STANDARD_TEXT_FIELD_WIDTH: f32 = 280.0;
const DOWNLOAD_CONVERSION_DETAIL_HEADER_TO_BODY_VERTICAL_SPACING: f32 = 10.0;

const STANDARD_ICON_SIZE_SCALE_FROM_CONTROL_HEIGHT: f32 = 0.72;
const STANDARD_ICON_TEXT_BUTTON_WIDTH_MINIMUM: f32 = 64.0;

const STANDARD_PAINT_SQUARE_CORNER_RADIUS: f32 = 0.0;
const STANDARD_PAINT_HAIRLINE_STROKE_WIDTH: f32 = 1.0;

const MODAL_PROMPT_VIEWPORT_HORIZONTAL_SAFE_MARGIN: f32 = 24.0;

const MUSIC_DOWNLOAD_PROMPT_ITEM_HORIZONTAL_SPACING: f32 = 8.0;
const MUSIC_DOWNLOAD_PROMPT_ITEM_VERTICAL_SPACING: f32 = 8.0;
const MUSIC_DOWNLOAD_PROMPT_ACTION_TO_PANEL_VERTICAL_SPACING: f32 = 8.0;
const MUSIC_DOWNLOAD_PROMPT_PREFERENCE_PANEL_VERTICAL_SPACING: f32 = 5.0;
const MUSIC_DOWNLOAD_PROMPT_MINIMUM_WIDTH: f32 = 210.0;
const MUSIC_DOWNLOAD_PROMPT_TITLE_WIDTH_MINIMUM: f32 = 48.0;
const MUSIC_DOWNLOAD_PROMPT_TITLE_WIDTH_MAXIMUM: f32 = 180.0;
const MUSIC_DOWNLOAD_PROMPT_CHOICE_CHIP_HEIGHT: f32 = 24.0;
const MUSIC_DOWNLOAD_PROMPT_CHOICE_CHIP_HORIZONTAL_PADDING: f32 = 8.0;
const MUSIC_DOWNLOAD_PROMPT_CHOICE_CHIP_WIDTH_MINIMUM: f32 = 36.0;
const MUSIC_DOWNLOAD_PROMPT_CHOICE_CHIP_WIDTH_MAXIMUM: f32 = 96.0;
const MUSIC_DOWNLOAD_PROMPT_CHOICE_CHIP_HORIZONTAL_SPACING: f32 = 6.0;
const MUSIC_DOWNLOAD_PROMPT_ACTION_BUTTON_HORIZONTAL_SPACING: f32 = 8.0;

const PLAYLIST_PROMPT_CONTENT_WIDTH: f32 = 320.0;
const PLAYLIST_PROMPT_ITEM_HORIZONTAL_SPACING: f32 = 6.0;
const PLAYLIST_PROMPT_ITEM_VERTICAL_SPACING: f32 = 6.0;
const PLAYLIST_PROMPT_BUTTON_HORIZONTAL_PADDING: f32 = 8.0;
const PLAYLIST_PROMPT_BUTTON_VERTICAL_PADDING: f32 = 4.0;
const PLAYLIST_PROMPT_ACTIONS_TO_BODY_VERTICAL_SPACING: f32 = 8.0;
const PLAYLIST_PROMPT_BODY_VERTICAL_SPACING: f32 = 5.0;
const PLAYLIST_PROMPT_TITLE_TEXT_SIZE: f32 = 16.0;
const PLAYLIST_PROMPT_BODY_TEXT_SIZE: f32 = 13.0;
const PLAYLIST_PROMPT_ACTION_BUTTON_HEIGHT: f32 = 30.0;
const PLAYLIST_PROMPT_ACTION_BUTTON_HORIZONTAL_SPACING: f32 = 8.0;
const PLAYLIST_PROMPT_CHOICE_SELECTED_CORNER_RADIUS: f32 = 6.0;
const PLAYLIST_PROMPT_CHOICE_SELECTED_STROKE_WIDTH: f32 = 1.0;

const OPTIONS_LANGUAGE_DETAIL_HEADER_TO_BODY_VERTICAL_SPACING: f32 = 10.0;
const OPTIONS_LANGUAGE_CHECKMARK_COLUMN_WIDTH: f32 = 18.0;
const TOOL_PATH_ROW_CONTROL_HORIZONTAL_SPACING: f32 = 6.0;
const TOOL_PATH_ROW_MINIMUM_PATH_TEXT_FIELD_WIDTH: f32 = 120.0;

const FORMAT_PICKER_HEADER_EXTRA_HEIGHT_FROM_CONTROL_METRICS: f32 = 12.0;
const FORMAT_PICKER_HEADER_SUMMARY_EXTRA_WIDTH_MULTIPLIER_FROM_ITEM_SPACING: f32 = 1.5;
const FORMAT_PICKER_HEADER_CENTER_TITLE_EXTRA_WIDTH_MULTIPLIER_FROM_ITEM_SPACING: f32 = 2.0;
const FORMAT_PICKER_FILTER_STAGE_EXTRA_WIDTH_FROM_TEXT_METRICS: f32 = 8.0;
const FORMAT_PICKER_FILTER_STAGE_NODE_WIDTH_MINIMUM: f32 = 64.0;
const FORMAT_PICKER_FILTER_STAGE_NODE_WIDTH_MAXIMUM: f32 = 220.0;
const FORMAT_PICKER_TABLE_COLUMN_EXTRA_WIDTH_FROM_TEXT_METRICS: f32 = 14.0;
const FORMAT_PICKER_TABLE_MARKER_COLUMN_WIDTH: f32 = 18.0;
const FORMAT_PICKER_TABLE_RESOLUTION_COLUMN_WIDTH_MINIMUM: f32 = 72.0;
const FORMAT_PICKER_TABLE_RESOLUTION_COLUMN_WIDTH_MAXIMUM: f32 = 128.0;
const FORMAT_PICKER_TABLE_DYNAMIC_RANGE_COLUMN_WIDTH_MINIMUM: f32 = 44.0;
const FORMAT_PICKER_TABLE_DYNAMIC_RANGE_COLUMN_WIDTH_MAXIMUM: f32 = 88.0;
const FORMAT_PICKER_TABLE_FPS_COLUMN_WIDTH_MINIMUM: f32 = 36.0;
const FORMAT_PICKER_TABLE_FPS_COLUMN_WIDTH_MAXIMUM: f32 = 72.0;
const FORMAT_PICKER_TABLE_SAMPLE_RATE_COLUMN_WIDTH_MINIMUM: f32 = 76.0;
const FORMAT_PICKER_TABLE_SAMPLE_RATE_COLUMN_WIDTH_MAXIMUM: f32 = 124.0;
const FORMAT_PICKER_TABLE_VIDEO_CODEC_COLUMN_WIDTH_MINIMUM: f32 = 84.0;
const FORMAT_PICKER_TABLE_AUDIO_CODEC_COLUMN_WIDTH_MINIMUM: f32 = 96.0;
const FORMAT_PICKER_TABLE_CODEC_COLUMN_WIDTH_MAXIMUM: f32 = 220.0;
const FORMAT_PICKER_TABLE_FILESIZE_COLUMN_WIDTH_MINIMUM: f32 = 70.0;
const FORMAT_PICKER_TABLE_FILESIZE_COLUMN_WIDTH_MAXIMUM: f32 = 112.0;
const FORMAT_PICKER_MUXED_MARKER_ICON_SIZE: f32 = 14.0;

const TITLEBAR_ESCAPE_MENU_WIDTH_GUARD_FROM_TEXT_METRICS: f32 = 2.0;
const TITLEBAR_HEIGHT: f32 = 26.0;
const TITLEBAR_APP_ICON_CELL_WIDTH: f32 = 25.0;
const TITLEBAR_APP_ICON_LEFT_MARGIN: f32 = 8.0;
const TITLEBAR_APP_ICON_SIZE: f32 = 16.0;
const TITLEBAR_TITLE_LEFT_PADDING: f32 = 5.0;
const TITLEBAR_WINDOW_BUTTON_WIDTH: f32 = 40.0;
const TITLEBAR_WINDOW_BUTTON_ICON_SIZE: f32 = 13.0;
const TITLEBAR_HOME_ICON_SIZE: f32 = 17.0;
const TITLEBAR_UPDATE_SIGNAL_ICON_SIZE: f32 = 18.0;
const TITLEBAR_ESCAPE_BUTTON_WIDTH: f32 = 40.0;
const TITLEBAR_ESCAPE_ICON_SIZE: f32 = 28.0;
const TITLEBAR_APP_TITLE_FONT_SIZE: f32 = 12.0;

const PROCESSING_LOG_TABLE_COLUMN_EXTRA_WIDTH_FROM_TEXT_METRICS: f32 = 12.0;
const PROCESSING_LOG_TABLE_TIME_COLUMN_WIDTH_MINIMUM: f32 = 74.0;
const PROCESSING_LOG_TABLE_TIME_COLUMN_WIDTH_MAXIMUM: f32 = 96.0;
const PROCESSING_LOG_TABLE_STATUS_COLUMN_WIDTH_MINIMUM: f32 = 36.0;
const PROCESSING_LOG_TABLE_STATUS_COLUMN_WIDTH_MAXIMUM: f32 = 58.0;
const PROCESSING_LOG_TABLE_ACTION_COLUMN_WIDTH_MINIMUM: f32 = 132.0;
const PROCESSING_LOG_TABLE_ACTION_COLUMN_WIDTH_MAXIMUM: f32 = 320.0;
const PROCESSING_LOG_TABLE_MODE_COLUMN_WIDTH_MINIMUM: f32 = 76.0;
const PROCESSING_LOG_TABLE_MODE_COLUMN_WIDTH_MAXIMUM: f32 = 124.0;

const PROCESSING_CONVERSION_CHOICE_BUTTON_EXTRA_HEIGHT_FROM_CONTROL_METRICS: f32 = 6.0;
const PROCESSING_LOG_TABLE_HEADER_ROW_HEIGHT: f32 = 20.0;
const PROCESSING_LOG_TABLE_ACTION_ROW_HEIGHT: f32 = 19.0;
const PROCESSING_LOG_TABLE_STEP_ROW_HEIGHT: f32 = 18.0;
const PROCESSING_LOG_TABLE_PARENT_ROW_HEIGHT: f32 = 22.0;
const PROCESSING_LOG_TABLE_SECTION_SEPARATOR_ROW_HEIGHT: f32 = 24.0;
const PROCESSING_LOG_TABLE_TEXT_LEFT_PADDING_WHEN_WIDE: f32 = 6.0;
const PROCESSING_LOG_TABLE_TEXT_WIDE_CELL_THRESHOLD: f32 = 12.0;
const PROCESSING_LOG_TABLE_TEXT_CLIP_HORIZONTAL_INSET: f32 = 3.0;
const PROCESSING_LOG_TABLE_TEXT_STRONG_FONT_SIZE: f32 = 13.0;
const PROCESSING_LOG_TABLE_TEXT_NORMAL_FONT_SIZE: f32 = 12.5;
const PROCESSING_LOG_TABLE_STATUS_ICON_FONT_SIZE: f32 = 14.0;
const PROCESSING_COMMAND_VIEWER_FONT_SIZE: f32 = 13.0;
const PROCESSING_COMMAND_VIEWER_TOKEN_SPACING_X: f32 = 5.0;
const PROCESSING_COMMAND_VIEWER_TOKEN_SPACING_Y: f32 = 5.0;
const PROCESSING_COMMAND_VIEWER_FRAME_MARGIN_X: f32 = 10.0;
const PROCESSING_COMMAND_VIEWER_FRAME_MARGIN_Y: f32 = 7.0;
const PROCESSING_COMMAND_VIEWER_FRAME_STROKE_WIDTH: f32 = 1.0;
const PROCESSING_COMMAND_VIEWER_FRAME_CORNER_RADIUS: f32 = 8.0;
const PROCESSING_COMMAND_VIEWER_TO_LOG_TABLE_VERTICAL_SPACING: f32 = 8.0;

const PREPARE_BOTTOM_ACTION_ROW_EXTRA_HEIGHT_FROM_CONTROL_METRICS: f32 = 12.0;
const PREPARE_TITLE_TEXT_SIZE: f32 = 18.0;
const PREPARE_BODY_TEXT_SIZE: f32 = 15.0;
const PREPARE_SMALL_TEXT_SIZE: f32 = 13.0;
const PREPARE_BOTTOM_ACTION_BUTTON_HORIZONTAL_SPACING: f32 = 12.0;
const PREPARE_ROOT_LANGUAGE_TO_HEADER_VERTICAL_SPACING: f32 = 16.0;
const PREPARE_ROOT_HEADER_TO_TOOL_ROWS_VERTICAL_SPACING: f32 = 24.0;
const PREPARE_LANGUAGE_DETAIL_HEADER_TO_CHOICES_VERTICAL_SPACING: f32 = 10.0;
const PREPARE_LANGUAGE_CHOICE_CHECKMARK_COLUMN_WIDTH: f32 = 18.0;
const PREPARE_TOOL_ROWS_CONTENT_LEFT_INDENT: f32 = 28.0;
const PREPARE_TOOL_ROW_EXTRA_HEIGHT_FROM_CONTROL_METRICS: f32 = 8.0;
const PREPARE_TOOL_ROW_ICON_WIDTH_SCALE_FROM_ROW_HEIGHT: f32 = 0.65;
const PREPARE_TOOL_ROW_NAME_TO_SEVERITY_GAP_MULTIPLIER: f32 = 2.8;
const PREPARE_TOOL_ROW_ICON_TO_NAME_GAP_MULTIPLIER: f32 = 0.8;
const PREPARE_TOOL_ROW_SEVERITY_TO_ACTION_GAP_MULTIPLIER: f32 = 1.4;
const PREPARE_TOOL_ROW_EXTRA_VERTICAL_SPACING_FROM_ITEM_SPACING: f32 = 6.0;
const PREPARE_TOOL_ROW_ACTION_WIDTH_EXTRA_FROM_BUTTON_PADDING: f32 = 1.0;
const PREPARE_ENVIRONMENT_ISSUES_HEADER_TO_ROWS_VERTICAL_SPACING: f32 = 4.0;
const PREPARE_ENVIRONMENT_ISSUES_BETWEEN_ROWS_VERTICAL_SPACING: f32 = 4.0;
const PREPARE_ENVIRONMENT_ISSUE_STATUS_COLUMN_WIDTH_SCALE_FROM_CONTROL_HEIGHT: f32 = 0.8;

const FORMAT_PICKER_SECTION_TABLE_MINIMUM_BODY_HEIGHT: f32 = 160.0;
const FORMAT_PICKER_SECTION_TABLE_MARKER_COLUMN_WIDTH: f32 = 20.0;
const FORMAT_PICKER_SECTION_TABLE_RANGE_COLUMN_MINIMUM_WIDTH: f32 = 160.0;
const FORMAT_PICKER_SUBTITLE_TARGET_COLUMN_MINIMUM_WIDTH: f32 = 180.0;
const FORMAT_PICKER_SUBTITLE_EXTENSION_COLUMN_MINIMUM_WIDTH: f32 = 48.0;
const FORMAT_PICKER_SUBTITLE_TABLE_ROW_HEIGHT: f32 = 24.0;
const FORMAT_PICKER_TABLE_REMAINDER_COLUMN_MINIMUM_WIDTH: f32 = 0.0;
const FORMAT_PICKER_TABLE_MINIMUM_SCROLLED_HEIGHT: f32 = 180.0;

const URL_ROW_SPINNER_ACTION_CELL_HORIZONTAL_SPACING: f32 = 0.0;

const MAIN_SECTION_VERTICAL_SPACING: f32 = 6.0;
const MAIN_BOTTOM_TRAILING_VERTICAL_SPACING: f32 = 2.0;
const MAIN_INLINE_CONTROL_GAP_SCALE_FROM_ITEM_SPACING: f32 = 0.5;
const MAIN_ORIGIN_CONTENT_OUTPUT_GAP_REDUCTION: f32 = 4.0;
const MAIN_MUSIC_PANEL_HORIZONTAL_PADDING: f32 = 8.0;
const MAIN_MUSIC_PANEL_VERTICAL_PADDING: f32 = 5.0;
const MAIN_MUSIC_PANEL_CORNER_RADIUS: f32 = 7.0;
const MAIN_MUSIC_PANEL_CONTROL_TO_LYRICS_VERTICAL_SPACING: f32 = 4.0;
const MAIN_MUSIC_LYRICS_FONT_SIZE_DELTA_FROM_BODY: f32 = 4.5;
const MAIN_MUSIC_LYRICS_ROW_EXTRA_HEIGHT_FROM_TEXT_METRICS: f32 = 8.0;
const MAIN_MUSIC_SEEK_BAR_HORIZONTAL_INSET: f32 = 2.0;
const MAIN_MUSIC_PLAYER_CONTROL_SPACING_MINIMUM: f32 = 8.0;
const MAIN_MUSIC_PLAYER_CONTROL_ROW_HEIGHT_MINIMUM: f32 = 28.0;
const MAIN_MUSIC_PLAYER_SEEK_ROW_HEIGHT: f32 = 12.0;
const MAIN_MUSIC_PLAYER_SEEK_TO_CONTROLS_SPACING: f32 = 6.0;
const MAIN_MUSIC_PLAYER_BPM_WIDTH: f32 = 72.0;
const MAIN_MUSIC_PLAYER_ANALYSIS_PEARLS_WIDTH: f32 = 58.0;
const MAIN_MUSIC_PLAYER_ANALYSIS_PEARL_RADIUS: f32 = 1.25;
const MAIN_MUSIC_PLAYER_ANALYSIS_PEARL_RADIUS_GAIN: f32 = 1.35;
const MAIN_MUSIC_PLAYER_ANALYSIS_PEARL_MAX_LIFT: f32 = 6.5;
const MAIN_MUSIC_PLAYER_TIME_TEXT_WIDTH: f32 = 108.0;
const MAIN_MUSIC_PLAYER_VOLUME_POPUP_WIDTH: f32 = 144.0;
const MAIN_MUSIC_ROUND_BUTTON_RADIUS_INSET: f32 = 1.0;
const MAIN_MUSIC_PLAYBACK_ICON_SCALE_FROM_BUTTON: f32 = 0.64;
const MAIN_MUSIC_STAGE_LABEL_SCALE_FROM_BUTTON: f32 = 0.46;
const MAIN_MUSIC_BUTTON_IDLE_FILL_ALPHA: u8 = 42;
const MAIN_MUSIC_BUTTON_HOVER_FILL_ALPHA: u8 = 76;
const MAIN_MUSIC_BUTTON_PRESSED_FILL_ALPHA: u8 = 104;
const MAIN_MUSIC_BUTTON_ACTIVE_FILL_ALPHA: u8 = 68;
const MAIN_MUSIC_BUTTON_ACTIVE_HOVER_FILL_ALPHA: u8 = 96;
const MAIN_MUSIC_BUTTON_ACTIVE_PRESSED_FILL_ALPHA: u8 = 124;
const MAIN_MUSIC_BUTTON_IDLE_STROKE_ALPHA: u8 = 54;
const MAIN_MUSIC_BUTTON_HOVER_STROKE_ALPHA: u8 = 88;
const MAIN_MUSIC_BUTTON_ACTIVE_STROKE_ALPHA: u8 = 154;
const MAIN_MUSIC_BUTTON_IDLE_FOREGROUND_ALPHA: u8 = 216;
const MAIN_MUSIC_BUTTON_HOVER_FOREGROUND_ALPHA: u8 = 244;
const MAIN_URL_ACTION_SPINNER_SIZE_SCALE_FROM_CONTROL_HEIGHT: f32 = 0.75;
const MAIN_URL_ACTION_SPINNER_TO_TEXT_HORIZONTAL_SPACING: f32 = 4.0;
const MAIN_MISSING_YT_DLP_CALLOUT_WIDTH: f32 = 320.0;
const MAIN_MISSING_YT_DLP_CALLOUT_VIEWPORT_EDGE_SAFE_MARGIN: f32 = 8.0;
const MAIN_MISSING_YT_DLP_CALLOUT_VERTICAL_OFFSET_FROM_ANCHOR: f32 = 6.0;

const MUSIC_COMPACT_ROW_HEIGHT: f32 = 40.0;
const MUSIC_COMPACT_ROW_SIDE_INSET: f32 = 1.0;
const MUSIC_COMPACT_COVER_SIZE: f32 = 32.0;
const MUSIC_COMPACT_ROW_CORNER_RADIUS: f32 = 6.0;
const MUSIC_COMPACT_ROW_HORIZONTAL_PADDING: f32 = 6.0;
const MUSIC_COMPACT_TITLE_TO_NEIGHBOR_HORIZONTAL_GAP: f32 = 6.0;
const MUSIC_COMPACT_STATUS_COLUMN_WIDTH: f32 = 48.0;
const MUSIC_COMPACT_CURRENT_MARKER_TOP_BOTTOM_INSET: f32 = 4.0;
const MUSIC_COMPACT_CURRENT_MARKER_WIDTH: f32 = 2.0;
const MUSIC_COMPACT_CURRENT_MARKER_CORNER_RADIUS: f32 = 1.0;
const MUSIC_COMPACT_COVER_CORNER_RADIUS: f32 = 5.0;
const MUSIC_COMPACT_PLACEHOLDER_ICON_SCALE_FROM_COVER_WIDTH: f32 = 0.50;
const MUSIC_COMPACT_PLAY_OVERLAY_RADIUS_SCALE_FROM_COVER_WIDTH: f32 = 0.32;
const MUSIC_COMPACT_PLAY_OVERLAY_ICON_SCALE_FROM_RADIUS: f32 = 1.15;
const MUSIC_COMPACT_TITLE_FONT_SIZE_DELTA_FROM_BODY: f32 = 1.0;
const MUSIC_COMPACT_REMOVE_ICON_SIZE: f32 = 14.0;
const MUSIC_COMPACT_REMOVE_BUTTON_CORNER_RADIUS: f32 = 2.0;

const SINGLE_MODE_TITLE_FIELD_EXTRA_HEIGHT_FROM_CONTROL_METRICS: f32 = 8.0;
const SINGLE_MODE_FIELD_TOP_VERTICAL_MARGIN: f32 = 2.0;
const SINGLE_MODE_THUMBNAIL_ROW_EXTRA_HEIGHT_FOR_FIELD_MARGIN: f32 = 1.0;
const SINGLE_MODE_INFO_LINE_HEIGHT: f32 = 14.0;
const SINGLE_MODE_INFO_BOTTOM_MARGIN: f32 = 4.0;
const SINGLE_MODE_INFO_LABEL_WIDTH: f32 = 66.0;
const SINGLE_MODE_INFO_LABEL_TO_VALUE_HORIZONTAL_GAP: f32 = 2.0;
const SINGLE_MODE_INFO_TEXT_FONT_SIZE: f32 = 10.0;
const SINGLE_MODE_RIGHT_INFO_VISIBLE_LINE_COUNT: usize = 8;
const SINGLE_MODE_THUMBNAIL_LOADING_SPINNER_SIZE: f32 = 48.0;
const SINGLE_MODE_THUMBNAIL_ASPECT_RATIO: f32 = 16.0 / 9.0;
const SINGLE_MODE_THUMBNAIL_FRAME_CORNER_RADIUS: f32 = 2.0;
const SINGLE_MODE_THUMBNAIL_INNER_FRAME_INSET: f32 = 1.0;
const SINGLE_MODE_DURATION_BADGE_MINIMUM_THUMBNAIL_WIDTH: f32 = 58.0;
const SINGLE_MODE_DURATION_BADGE_MINIMUM_THUMBNAIL_HEIGHT: f32 = 20.0;
const SINGLE_MODE_DURATION_BADGE_WIDTH: f32 = 54.0;
const SINGLE_MODE_DURATION_BADGE_HEIGHT: f32 = 16.0;
const SINGLE_MODE_DURATION_BADGE_RIGHT_INSET: f32 = 2.0;
const SINGLE_MODE_DURATION_BADGE_BOTTOM_INSET: f32 = 2.0;
const SINGLE_MODE_DURATION_BADGE_CORNER_RADIUS: f32 = 2.0;
const SINGLE_MODE_DURATION_BADGE_FONT_SIZE: f32 = 10.0;
const SINGLE_MODE_PLACEHOLDER_ICON_MAXIMUM_SIZE: f32 = 28.0;
const SINGLE_MODE_PLACEHOLDER_ICON_SCALE_FROM_THUMBNAIL: f32 = 0.45;
const SINGLE_MODE_PLACEHOLDER_ICON_MINIMUM_SIZE: f32 = 8.0;
const SINGLE_MODE_PLACEHOLDER_ICON_UPWARD_SHIFT_MAXIMUM: f32 = 10.0;
const SINGLE_MODE_PLACEHOLDER_ICON_UPWARD_SHIFT_SCALE_FROM_HEIGHT: f32 = 0.12;
const SINGLE_MODE_PLACEHOLDER_TEXT_MINIMUM_THUMBNAIL_HEIGHT: f32 = 34.0;
const SINGLE_MODE_PLACEHOLDER_TEXT_DOWNWARD_SHIFT_MAXIMUM: f32 = 18.0;
const SINGLE_MODE_PLACEHOLDER_TEXT_DOWNWARD_SHIFT_SCALE_FROM_HEIGHT: f32 = 0.2;
const SINGLE_MODE_PLACEHOLDER_TEXT_FONT_SIZE: f32 = 11.0;

const ITEM_CARD_TITLE_ROW_HEIGHT: f32 = 16.0;
const ITEM_CARD_TITLE_FONT_SIZE: f32 = 14.0;
const ITEM_CARD_TITLE_SPINNER_TOP_PADDING: f32 = 2.0;
const ITEM_CARD_TITLE_TEXT_TOP_PADDING: f32 = -3.0;
const ITEM_CARD_TITLE_DELETE_TOP_PADDING: f32 = -2.0;
const ITEM_CARD_TITLE_SPINNER_SIZE: f32 = 9.0;
const ITEM_CARD_TITLE_MAXIMUM_VISIBLE_LINES: usize = 2;
const ITEM_CARD_TITLE_MAXIMUM_LINE_HEIGHT_EXTRA_SPACING: f32 = 2.0;
const ITEM_CARD_THUMBNAIL_WIDTH: f32 = 128.0;
const ITEM_CARD_THUMBNAIL_HEIGHT_RATIO_FROM_WIDTH: f32 = 9.0 / 16.0;
const ITEM_CARD_FIELD_ROW_HEIGHT: f32 = 18.0;
const ITEM_CARD_FIELD_ROW_VERTICAL_PADDING: f32 = 0.0;
const ITEM_CARD_DETAIL_COLUMN_GAP: f32 = 3.0;
const ITEM_CARD_DETAIL_ROW_GAP: f32 = 3.0;
const ITEM_CARD_COLUMN_GAP: f32 = 8.0;
const ITEM_CARD_LABEL_TO_VALUE_HORIZONTAL_GAP: f32 = 4.0;
const ITEM_CARD_REMAINDER_COLUMN_MINIMUM_WIDTH: f32 = 120.0;
const ITEM_CARD_REMAINDER_COLUMN_MAXIMUM_WIDTH: f32 = 10_000.0;
const ITEM_CARD_ZERO_REMAINDER_COLUMN_MINIMUM_WIDTH: f32 = 0.0;
const ITEM_CARD_DELETE_ICON_SIZE: f32 = 14.0;
const ITEM_CARD_ICON_BUTTON_CORNER_RADIUS: f32 = 2.0;
const ITEM_CARD_DOWNLOAD_ICON_HEIGHT_REDUCTION_FROM_ROW_HEIGHT: f32 = 8.0;
const ITEM_CARD_DOWNLOAD_ICON_SIZE_MINIMUM: f32 = 10.0;
const ITEM_CARD_DURATION_BADGE_PADDING_X: f32 = 2.0;
const ITEM_CARD_DURATION_BADGE_PADDING_Y: f32 = 2.0;
const ITEM_CARD_DURATION_BADGE_RIGHT_INSET: f32 = 4.0;
const ITEM_CARD_DURATION_BADGE_BOTTOM_INSET: f32 = 4.0;
const ITEM_CARD_DURATION_BADGE_CORNER_RADIUS: f32 = 2.0;
const ITEM_CARD_FIELD_CORNER_RADIUS: f32 = 2.0;
const ITEM_CARD_OUTPUT_CONTAINER_PICKER_WIDTH: f32 = 68.0;
const ITEM_CARD_FIELD_TEXT_HORIZONTAL_INSET: f32 = 4.0;
const ITEM_CARD_FIELD_TEXT_WIDTH_REDUCTION_FROM_INSET: f32 = 8.0;

const ITEM_CARD_FIELD_TEXT_EDIT_MARGIN_X: i8 = 4;
const ITEM_CARD_FIELD_TEXT_EDIT_MARGIN_Y: i8 = 2;

const FORMAT_PICKER_EMPTY_MESSAGE_TOP_VERTICAL_SPACING: f32 = 12.0;
const FORMAT_PICKER_SECTION_ROW_HEIGHT: f32 = 24.0;
const FORMAT_PICKER_TIME_RANGE_TIMELINE_HEIGHT: f32 = 56.0;
const FORMAT_PICKER_TIME_RANGE_TIMELINE_HORIZONTAL_INSET: f32 = 8.0;
const FORMAT_PICKER_TIME_RANGE_TIMELINE_VERTICAL_INSET: f32 = 6.0;
const FORMAT_PICKER_TIME_RANGE_TRACK_STROKE_WIDTH: f32 = 6.0;
const FORMAT_PICKER_TIME_RANGE_CHAPTER_MARKER_RADIUS: f32 = 3.0;
const FORMAT_PICKER_TIME_RANGE_PLAYHEAD_STROKE_WIDTH: f32 = 2.0;
const FORMAT_PICKER_TIME_RANGE_PLAYHEAD_RADIUS: f32 = 6.0;
const FORMAT_PICKER_TIME_RANGE_SEGMENT_HALF_HEIGHT: f32 = 4.0;
const FORMAT_PICKER_TIME_RANGE_SEGMENT_CORNER_RADIUS: f32 = 2.0;
const FORMAT_PICKER_TIME_RANGE_BOUNDARY_NOTCH_HALF_HEIGHT: f32 = 9.0;
const FORMAT_PICKER_TIME_RANGE_SNAP_DISTANCE_PIXELS: f32 = 10.0;
const FORMAT_PICKER_TIME_RANGE_MARKER_HIT_RADIUS_SCALE: f32 = 4.0;
const FORMAT_PICKER_TIME_RANGE_MARKER_LABEL_FONT_SIZE: f32 = 10.0;
const FORMAT_PICKER_TIME_RANGE_TIMESTAMP_FONT_SIZE: f32 = 11.0;
const FORMAT_PICKER_FILTER_BUTTON_HORIZONTAL_PADDING: f32 = 5.0;
const FORMAT_PICKER_FILTER_BUTTON_VERTICAL_PADDING: f32 = 1.0;
const FORMAT_PICKER_FILTER_NODE_HEIGHT_REDUCTION_FROM_CONTROL_METRICS: f32 = 10.0;
const FORMAT_PICKER_FILTER_NODE_HEIGHT_MINIMUM: f32 = 20.0;
const FORMAT_PICKER_FILTER_NODE_HEIGHT_MAXIMUM: f32 = 24.0;
const FORMAT_PICKER_FILTER_VIEWPORT_HORIZONTAL_SAFE_MARGIN: f32 = 18.0;
const FORMAT_PICKER_FILTER_STAGE_SLOT_SAFE_MARGIN: f32 = 8.0;
const FORMAT_PICKER_FILTER_STAGE_WIDTH_MINIMUM_WITHIN_SLOT: f32 = 48.0;
const FORMAT_PICKER_FILTER_STAGE_TITLE_TO_NODES_VERTICAL_SPACING: f32 = 5.0;
const FORMAT_PICKER_FILTER_NODE_VERTICAL_SPACING: f32 = 3.0;
const FORMAT_PICKER_FILTER_CONNECTION_STROKE_WIDTH: f32 = 2.0;

pub(super) fn format_picker_section_table_minimum_body_height() -> f32 {
    FORMAT_PICKER_SECTION_TABLE_MINIMUM_BODY_HEIGHT
}

pub(super) fn format_picker_section_table_marker_column_width() -> f32 {
    FORMAT_PICKER_SECTION_TABLE_MARKER_COLUMN_WIDTH
}

pub(super) fn format_picker_section_table_range_column_minimum_width() -> f32 {
    FORMAT_PICKER_SECTION_TABLE_RANGE_COLUMN_MINIMUM_WIDTH
}

pub(super) fn format_picker_subtitle_target_column_minimum_width() -> f32 {
    FORMAT_PICKER_SUBTITLE_TARGET_COLUMN_MINIMUM_WIDTH
}

pub(super) fn format_picker_subtitle_extension_column_minimum_width() -> f32 {
    FORMAT_PICKER_SUBTITLE_EXTENSION_COLUMN_MINIMUM_WIDTH
}

pub(super) fn format_picker_subtitle_table_row_height() -> f32 {
    FORMAT_PICKER_SUBTITLE_TABLE_ROW_HEIGHT
}

pub(super) fn format_picker_table_remainder_column_minimum_width() -> f32 {
    FORMAT_PICKER_TABLE_REMAINDER_COLUMN_MINIMUM_WIDTH
}

pub(super) fn format_picker_table_minimum_scrolled_height() -> f32 {
    FORMAT_PICKER_TABLE_MINIMUM_SCROLLED_HEIGHT
}

pub(super) fn url_row_spinner_action_cell_horizontal_spacing() -> f32 {
    URL_ROW_SPINNER_ACTION_CELL_HORIZONTAL_SPACING
}

pub(super) fn main_section_vertical_spacing() -> f32 {
    MAIN_SECTION_VERTICAL_SPACING
}

pub(super) fn main_bottom_trailing_vertical_spacing() -> f32 {
    MAIN_BOTTOM_TRAILING_VERTICAL_SPACING
}

pub(super) fn main_content_to_output_vertical_spacing_for_origin_mode() -> f32 {
    (MAIN_SECTION_VERTICAL_SPACING - MAIN_ORIGIN_CONTENT_OUTPUT_GAP_REDUCTION).max(0.0)
}

pub(super) fn main_inline_control_gap_from_current_spacing(ui: &Ui) -> f32 {
    ui.spacing().item_spacing.x * MAIN_INLINE_CONTROL_GAP_SCALE_FROM_ITEM_SPACING
}

pub(super) fn main_music_lyrics_row_height_from_current_text_metrics(
    ui: &Ui,
    control_row_height: f32,
) -> f32 {
    (TextStyle::Body.resolve(ui.style()).size
        + MAIN_MUSIC_LYRICS_FONT_SIZE_DELTA_FROM_BODY
        + MAIN_MUSIC_LYRICS_ROW_EXTRA_HEIGHT_FROM_TEXT_METRICS)
        .max(control_row_height)
}

pub(super) fn main_music_panel_height_for_content(
    control_row_height: f32,
    lyrics_row_height: Option<f32>,
) -> f32 {
    MAIN_MUSIC_PANEL_VERTICAL_PADDING * 2.0
        + control_row_height
        + lyrics_row_height
            .map(|height| height + MAIN_MUSIC_PANEL_CONTROL_TO_LYRICS_VERTICAL_SPACING)
            .unwrap_or(0.0)
}

pub(super) fn main_music_panel_corner_radius() -> f32 {
    MAIN_MUSIC_PANEL_CORNER_RADIUS
}

pub(super) fn main_music_panel_content_rect(panel_rect: egui::Rect) -> egui::Rect {
    let content_width = (panel_rect.width() - MAIN_MUSIC_PANEL_HORIZONTAL_PADDING * 2.0).max(1.0);
    let content_height = (panel_rect.height() - MAIN_MUSIC_PANEL_VERTICAL_PADDING * 2.0).max(1.0);
    egui::Rect::from_min_size(
        egui::pos2(
            panel_rect.left() + MAIN_MUSIC_PANEL_HORIZONTAL_PADDING,
            panel_rect.top() + MAIN_MUSIC_PANEL_VERTICAL_PADDING,
        ),
        egui::vec2(content_width, content_height),
    )
}

pub(super) fn main_music_control_to_lyrics_vertical_spacing() -> f32 {
    MAIN_MUSIC_PANEL_CONTROL_TO_LYRICS_VERTICAL_SPACING
}

pub(super) fn main_music_lyrics_font_size_from_body(ui: &Ui) -> f32 {
    TextStyle::Body.resolve(ui.style()).size + MAIN_MUSIC_LYRICS_FONT_SIZE_DELTA_FROM_BODY
}

pub(super) fn main_music_seek_bar_inner_rect(rect: egui::Rect) -> egui::Rect {
    rect.shrink2(egui::vec2(MAIN_MUSIC_SEEK_BAR_HORIZONTAL_INSET, 0.0))
}

pub(super) fn main_music_player_control_spacing_from_current_spacing(ui: &Ui) -> f32 {
    ui.spacing()
        .item_spacing
        .x
        .max(MAIN_MUSIC_PLAYER_CONTROL_SPACING_MINIMUM)
}

pub(super) fn main_music_player_control_row_height(base_height: f32) -> f32 {
    base_height.max(MAIN_MUSIC_PLAYER_CONTROL_ROW_HEIGHT_MINIMUM)
}

pub(super) fn main_music_player_control_row_height_from_current_metrics(ui: &Ui) -> f32 {
    main_music_player_control_row_height(ui.spacing().interact_size.y)
}

pub(super) fn main_music_player_height_from_control_row(control_row_height: f32) -> f32 {
    MAIN_MUSIC_PLAYER_SEEK_ROW_HEIGHT
        + MAIN_MUSIC_PLAYER_SEEK_TO_CONTROLS_SPACING
        + control_row_height.max(1.0)
}

pub(super) fn main_music_player_seek_row_height() -> f32 {
    MAIN_MUSIC_PLAYER_SEEK_ROW_HEIGHT
}

pub(super) fn main_music_player_seek_to_controls_spacing() -> f32 {
    MAIN_MUSIC_PLAYER_SEEK_TO_CONTROLS_SPACING
}

pub(super) fn main_music_player_time_text_width() -> f32 {
    MAIN_MUSIC_PLAYER_TIME_TEXT_WIDTH
}

pub(super) fn main_music_player_bpm_width() -> f32 {
    MAIN_MUSIC_PLAYER_BPM_WIDTH
}

pub(super) fn main_music_player_analysis_pearls_width() -> f32 {
    MAIN_MUSIC_PLAYER_ANALYSIS_PEARLS_WIDTH
}

pub(super) fn main_music_player_analysis_pearl_radius(value: f32) -> f32 {
    MAIN_MUSIC_PLAYER_ANALYSIS_PEARL_RADIUS
        + value.clamp(0.0, 1.0) * MAIN_MUSIC_PLAYER_ANALYSIS_PEARL_RADIUS_GAIN
}

pub(super) fn main_music_player_analysis_pearl_max_lift() -> f32 {
    MAIN_MUSIC_PLAYER_ANALYSIS_PEARL_MAX_LIFT
}

pub(super) fn main_music_player_volume_popup_width() -> f32 {
    MAIN_MUSIC_PLAYER_VOLUME_POPUP_WIDTH
}

pub(super) fn main_music_round_button_radius_for_rect(rect: egui::Rect) -> f32 {
    (rect.width().min(rect.height()) * 0.5 - MAIN_MUSIC_ROUND_BUTTON_RADIUS_INSET).max(1.0)
}

pub(super) fn main_music_playback_icon_size_for_rect(rect: egui::Rect) -> f32 {
    rect.width().min(rect.height()) * MAIN_MUSIC_PLAYBACK_ICON_SCALE_FROM_BUTTON
}

pub(super) fn main_music_stage_label_size_for_rect(rect: egui::Rect) -> f32 {
    rect.width().min(rect.height()) * MAIN_MUSIC_STAGE_LABEL_SCALE_FROM_BUTTON
}

pub(super) fn main_music_button_fill(
    ui: &Ui,
    response: &egui::Response,
    active_color: Option<egui::Color32>,
) -> egui::Color32 {
    let alpha = match (
        active_color.is_some(),
        response.is_pointer_button_down_on(),
        response.hovered(),
    ) {
        (true, true, _) => MAIN_MUSIC_BUTTON_ACTIVE_PRESSED_FILL_ALPHA,
        (true, false, true) => MAIN_MUSIC_BUTTON_ACTIVE_HOVER_FILL_ALPHA,
        (true, false, false) => MAIN_MUSIC_BUTTON_ACTIVE_FILL_ALPHA,
        (false, true, _) => MAIN_MUSIC_BUTTON_PRESSED_FILL_ALPHA,
        (false, false, true) => MAIN_MUSIC_BUTTON_HOVER_FILL_ALPHA,
        (false, false, false) => MAIN_MUSIC_BUTTON_IDLE_FILL_ALPHA,
    };
    let base = active_color.unwrap_or_else(|| ui.visuals().widgets.inactive.fg_stroke.color);
    egui::Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), alpha)
}

pub(super) fn main_music_button_stroke(
    ui: &Ui,
    response: &egui::Response,
    active_color: Option<egui::Color32>,
) -> egui::Stroke {
    let base = active_color.unwrap_or_else(|| ui.visuals().widgets.inactive.fg_stroke.color);
    let alpha = if active_color.is_some() {
        MAIN_MUSIC_BUTTON_ACTIVE_STROKE_ALPHA
    } else if response.hovered() || response.is_pointer_button_down_on() {
        MAIN_MUSIC_BUTTON_HOVER_STROKE_ALPHA
    } else {
        MAIN_MUSIC_BUTTON_IDLE_STROKE_ALPHA
    };
    egui::Stroke::new(
        if active_color.is_some() { 1.25 } else { 1.0 },
        egui::Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), alpha),
    )
}

pub(super) fn main_music_button_foreground(
    ui: &Ui,
    response: &egui::Response,
    active_color: Option<egui::Color32>,
) -> egui::Color32 {
    let base = active_color.unwrap_or_else(|| ui.style().interact(response).fg_stroke.color);
    let alpha = if response.hovered() || response.is_pointer_button_down_on() {
        MAIN_MUSIC_BUTTON_HOVER_FOREGROUND_ALPHA
    } else {
        MAIN_MUSIC_BUTTON_IDLE_FOREGROUND_ALPHA
    };
    egui::Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), alpha)
}

pub(super) fn main_url_action_spinner_size_for_control_height(control_height: f32) -> f32 {
    control_height * MAIN_URL_ACTION_SPINNER_SIZE_SCALE_FROM_CONTROL_HEIGHT
}

pub(super) fn main_url_action_spinner_to_text_horizontal_spacing() -> f32 {
    MAIN_URL_ACTION_SPINNER_TO_TEXT_HORIZONTAL_SPACING
}

pub(super) fn main_missing_yt_dlp_callout_width() -> f32 {
    MAIN_MISSING_YT_DLP_CALLOUT_WIDTH
}

pub(super) fn main_missing_yt_dlp_callout_left_for_anchor(anchor: egui::Rect) -> f32 {
    (anchor.right() - MAIN_MISSING_YT_DLP_CALLOUT_WIDTH)
        .max(MAIN_MISSING_YT_DLP_CALLOUT_VIEWPORT_EDGE_SAFE_MARGIN)
}

pub(super) fn main_missing_yt_dlp_callout_top_for_anchor(anchor: egui::Rect) -> f32 {
    anchor.bottom() + MAIN_MISSING_YT_DLP_CALLOUT_VERTICAL_OFFSET_FROM_ANCHOR
}

pub(super) fn main_missing_tool_button_stroke_width() -> f32 {
    standard_paint_hairline_stroke_width()
}

pub(super) fn main_missing_tool_callout_stroke_width() -> f32 {
    standard_paint_hairline_stroke_width()
}

pub(super) fn music_compact_row_height() -> f32 {
    MUSIC_COMPACT_ROW_HEIGHT
}

pub(super) fn music_compact_row_side_inset() -> f32 {
    MUSIC_COMPACT_ROW_SIDE_INSET
}

pub(super) fn music_compact_cover_size() -> f32 {
    MUSIC_COMPACT_COVER_SIZE
}

pub(super) fn music_compact_row_corner_radius() -> f32 {
    MUSIC_COMPACT_ROW_CORNER_RADIUS
}

pub(super) fn music_compact_row_horizontal_padding() -> f32 {
    MUSIC_COMPACT_ROW_HORIZONTAL_PADDING
}

pub(super) fn music_compact_title_to_neighbor_horizontal_gap() -> f32 {
    MUSIC_COMPACT_TITLE_TO_NEIGHBOR_HORIZONTAL_GAP
}

pub(super) fn music_compact_status_column_width() -> f32 {
    MUSIC_COMPACT_STATUS_COLUMN_WIDTH
}

pub(super) fn music_compact_current_marker_rect_for_row(row_rect: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::pos2(
            row_rect.left(),
            row_rect.top() + MUSIC_COMPACT_CURRENT_MARKER_TOP_BOTTOM_INSET,
        ),
        egui::pos2(
            row_rect.left() + MUSIC_COMPACT_CURRENT_MARKER_WIDTH,
            row_rect.bottom() - MUSIC_COMPACT_CURRENT_MARKER_TOP_BOTTOM_INSET,
        ),
    )
}

pub(super) fn music_compact_current_marker_corner_radius() -> f32 {
    MUSIC_COMPACT_CURRENT_MARKER_CORNER_RADIUS
}

pub(super) fn music_compact_cover_corner_radius() -> f32 {
    MUSIC_COMPACT_COVER_CORNER_RADIUS
}

pub(super) fn music_compact_row_border_stroke_width() -> f32 {
    standard_paint_hairline_stroke_width()
}

pub(super) fn music_compact_cover_border_stroke_width() -> f32 {
    standard_paint_hairline_stroke_width()
}

pub(super) fn music_compact_placeholder_icon_size_for_cover_rect(rect: egui::Rect) -> f32 {
    rect.width() * MUSIC_COMPACT_PLACEHOLDER_ICON_SCALE_FROM_COVER_WIDTH
}

pub(super) fn music_compact_play_overlay_radius_for_cover_rect(rect: egui::Rect) -> f32 {
    rect.width() * MUSIC_COMPACT_PLAY_OVERLAY_RADIUS_SCALE_FROM_COVER_WIDTH
}

pub(super) fn music_compact_play_overlay_icon_size_for_radius(radius: f32) -> f32 {
    radius * MUSIC_COMPACT_PLAY_OVERLAY_ICON_SCALE_FROM_RADIUS
}

pub(super) fn music_compact_title_font_size_from_body(ui: &Ui) -> f32 {
    TextStyle::Body.resolve(ui.style()).size + MUSIC_COMPACT_TITLE_FONT_SIZE_DELTA_FROM_BODY
}

pub(super) fn music_compact_remove_icon_size() -> f32 {
    MUSIC_COMPACT_REMOVE_ICON_SIZE
}

pub(super) fn music_compact_remove_button_corner_radius() -> f32 {
    MUSIC_COMPACT_REMOVE_BUTTON_CORNER_RADIUS
}

pub(super) fn single_mode_title_field_height_for_control_height(control_height: f32) -> f32 {
    control_height.max(24.0) + SINGLE_MODE_TITLE_FIELD_EXTRA_HEIGHT_FROM_CONTROL_METRICS
}

pub(super) fn single_mode_thumbnail_row_height_for_right_column_width(
    right_column_width: f32,
) -> f32 {
    if right_column_width <= 0.0 || SINGLE_MODE_THUMBNAIL_ASPECT_RATIO <= 0.0 {
        return 0.0;
    }
    (right_column_width / SINGLE_MODE_THUMBNAIL_ASPECT_RATIO)
        + SINGLE_MODE_FIELD_TOP_VERTICAL_MARGIN
        + SINGLE_MODE_THUMBNAIL_ROW_EXTRA_HEIGHT_FOR_FIELD_MARGIN
}

pub(super) fn single_mode_right_info_slot_height() -> f32 {
    SINGLE_MODE_RIGHT_INFO_VISIBLE_LINE_COUNT as f32 * SINGLE_MODE_INFO_LINE_HEIGHT
        + SINGLE_MODE_INFO_BOTTOM_MARGIN
}

pub(super) fn single_mode_thumbnail_area_for_row_rect(rect: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_max(
        rect.min + egui::vec2(0.0, SINGLE_MODE_FIELD_TOP_VERTICAL_MARGIN),
        rect.max,
    )
}

pub(super) fn single_mode_thumbnail_aspect_ratio() -> f32 {
    SINGLE_MODE_THUMBNAIL_ASPECT_RATIO
}

pub(super) fn single_mode_info_line_height() -> f32 {
    SINGLE_MODE_INFO_LINE_HEIGHT
}

pub(super) fn single_mode_info_bottom_margin() -> f32 {
    SINGLE_MODE_INFO_BOTTOM_MARGIN
}

pub(super) fn single_mode_info_label_width_for_line_width(line_width: f32) -> f32 {
    SINGLE_MODE_INFO_LABEL_WIDTH.min(line_width)
}

pub(super) fn single_mode_info_label_to_value_horizontal_gap() -> f32 {
    SINGLE_MODE_INFO_LABEL_TO_VALUE_HORIZONTAL_GAP
}

pub(super) fn single_mode_info_text_font_size() -> f32 {
    SINGLE_MODE_INFO_TEXT_FONT_SIZE
}

pub(super) fn single_mode_info_text_underline_stroke_width() -> f32 {
    standard_paint_hairline_stroke_width()
}

pub(super) fn single_mode_info_text_underline_vertical_inset() -> f32 {
    standard_paint_hairline_stroke_width()
}

pub(super) fn single_mode_thumbnail_frame_corner_radius() -> f32 {
    SINGLE_MODE_THUMBNAIL_FRAME_CORNER_RADIUS
}

pub(super) fn single_mode_thumbnail_inner_rect(rect: egui::Rect) -> egui::Rect {
    rect.shrink(SINGLE_MODE_THUMBNAIL_INNER_FRAME_INSET)
}

pub(super) fn single_mode_thumbnail_loading_spinner_size_for_inner_rect(inner: egui::Rect) -> f32 {
    SINGLE_MODE_THUMBNAIL_LOADING_SPINNER_SIZE.min(inner.width().min(inner.height()).max(1.0))
}

pub(super) fn single_mode_duration_badge_should_be_visible(inner: egui::Rect) -> bool {
    inner.width() > SINGLE_MODE_DURATION_BADGE_MINIMUM_THUMBNAIL_WIDTH
        && inner.height() > SINGLE_MODE_DURATION_BADGE_MINIMUM_THUMBNAIL_HEIGHT
}

pub(super) fn single_mode_duration_badge_rect(inner: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(
            inner.right()
                - SINGLE_MODE_DURATION_BADGE_WIDTH
                - SINGLE_MODE_DURATION_BADGE_RIGHT_INSET,
            inner.bottom()
                - SINGLE_MODE_DURATION_BADGE_HEIGHT
                - SINGLE_MODE_DURATION_BADGE_BOTTOM_INSET,
        ),
        egui::vec2(
            SINGLE_MODE_DURATION_BADGE_WIDTH,
            SINGLE_MODE_DURATION_BADGE_HEIGHT,
        ),
    )
}

pub(super) fn single_mode_duration_badge_corner_radius() -> f32 {
    SINGLE_MODE_DURATION_BADGE_CORNER_RADIUS
}

pub(super) fn single_mode_duration_badge_font_size() -> f32 {
    SINGLE_MODE_DURATION_BADGE_FONT_SIZE
}

pub(super) fn single_mode_placeholder_icon_size_for_inner_rect(inner: egui::Rect) -> f32 {
    SINGLE_MODE_PLACEHOLDER_ICON_MAXIMUM_SIZE.min(
        (inner.width().min(inner.height()) * SINGLE_MODE_PLACEHOLDER_ICON_SCALE_FROM_THUMBNAIL)
            .max(SINGLE_MODE_PLACEHOLDER_ICON_MINIMUM_SIZE),
    )
}

pub(super) fn single_mode_placeholder_icon_center_for_inner_rect(inner: egui::Rect) -> egui::Pos2 {
    inner.center()
        - egui::vec2(
            0.0,
            SINGLE_MODE_PLACEHOLDER_ICON_UPWARD_SHIFT_MAXIMUM
                .min(inner.height() * SINGLE_MODE_PLACEHOLDER_ICON_UPWARD_SHIFT_SCALE_FROM_HEIGHT),
        )
}

pub(super) fn single_mode_placeholder_text_should_be_visible(inner: egui::Rect) -> bool {
    inner.height() > SINGLE_MODE_PLACEHOLDER_TEXT_MINIMUM_THUMBNAIL_HEIGHT
}

pub(super) fn single_mode_placeholder_text_center_for_inner_rect(inner: egui::Rect) -> egui::Pos2 {
    inner.center()
        + egui::vec2(
            0.0,
            SINGLE_MODE_PLACEHOLDER_TEXT_DOWNWARD_SHIFT_MAXIMUM.min(
                inner.height() * SINGLE_MODE_PLACEHOLDER_TEXT_DOWNWARD_SHIFT_SCALE_FROM_HEIGHT,
            ),
        )
}

pub(super) fn single_mode_placeholder_text_font_size() -> f32 {
    SINGLE_MODE_PLACEHOLDER_TEXT_FONT_SIZE
}

pub(super) fn item_card_title_row_height() -> f32 {
    ITEM_CARD_TITLE_ROW_HEIGHT
}

pub(super) fn item_card_title_font_size() -> f32 {
    ITEM_CARD_TITLE_FONT_SIZE
}

pub(super) fn item_card_title_spinner_top_padding() -> f32 {
    ITEM_CARD_TITLE_SPINNER_TOP_PADDING
}

pub(super) fn item_card_title_text_top_padding() -> f32 {
    ITEM_CARD_TITLE_TEXT_TOP_PADDING
}

pub(super) fn item_card_title_delete_top_padding() -> f32 {
    ITEM_CARD_TITLE_DELETE_TOP_PADDING
}

pub(super) fn item_card_title_spinner_size() -> f32 {
    ITEM_CARD_TITLE_SPINNER_SIZE
}

pub(super) fn item_card_title_spinner_width_from_current_spacing(ui: &Ui) -> f32 {
    ITEM_CARD_TITLE_SPINNER_SIZE + ui.spacing().item_spacing.x
}

pub(super) fn item_card_title_delete_button_width_from_current_control_metrics(ui: &Ui) -> f32 {
    ui.spacing().interact_size.y
}

pub(super) fn item_card_two_line_title_height_for_visible_text_and_width(
    ui: &Ui,
    text: &str,
    max_width: f32,
    font_id: egui::FontId,
) -> f32 {
    wrapped_text_height(
        ui,
        text,
        max_width,
        font_id,
        TextStyle::Body,
        Some(ITEM_CARD_TITLE_MAXIMUM_VISIBLE_LINES),
        true,
        Some('…'),
    )
}

pub(super) fn item_card_maximum_two_line_title_height_for_font(
    ui: &Ui,
    font_id: egui::FontId,
) -> f32 {
    max_text_height_for_lines(
        ui,
        font_id,
        TextStyle::Body,
        ITEM_CARD_TITLE_MAXIMUM_VISIBLE_LINES,
        ITEM_CARD_TITLE_MAXIMUM_LINE_HEIGHT_EXTRA_SPACING,
    )
    .max(ITEM_CARD_TITLE_ROW_HEIGHT)
}

pub(super) fn item_card_title_height_for_measured_parts(
    title_height: f32,
    spinner_height: f32,
) -> f32 {
    ITEM_CARD_TITLE_ROW_HEIGHT.max(title_height.max(spinner_height))
}

pub(super) fn item_card_loading_spinner_height() -> f32 {
    (ITEM_CARD_TITLE_SPINNER_TOP_PADDING.max(0.0) + ITEM_CARD_TITLE_SPINNER_SIZE)
        .max(ITEM_CARD_TITLE_ROW_HEIGHT)
}

pub(super) fn item_card_thumbnail_width() -> f32 {
    ITEM_CARD_THUMBNAIL_WIDTH
}

pub(super) fn item_card_thumbnail_height() -> f32 {
    ITEM_CARD_THUMBNAIL_WIDTH * ITEM_CARD_THUMBNAIL_HEIGHT_RATIO_FROM_WIDTH
}

pub(super) fn item_card_thumbnail_size() -> egui::Vec2 {
    egui::vec2(item_card_thumbnail_width(), item_card_thumbnail_height())
}

pub(super) fn item_card_detail_width_for_card_width(card_width: f32) -> f32 {
    (card_width - item_card_thumbnail_width() - ITEM_CARD_COLUMN_GAP).max(0.0)
}

pub(super) fn item_card_body_target_height_for_header_height(header_height: f32) -> f32 {
    (item_card_thumbnail_height() - header_height).max(0.0)
}

pub(super) fn item_card_body_rows_gap_for_visible_row_count(visible_body_rows: usize) -> f32 {
    visible_body_rows.saturating_sub(1) as f32 * ITEM_CARD_DETAIL_ROW_GAP
}

pub(super) fn item_card_body_content_height_for_visible_row_count(visible_body_rows: usize) -> f32 {
    visible_body_rows as f32 * item_card_row_block_height()
        + item_card_body_rows_gap_for_visible_row_count(visible_body_rows)
}

pub(super) fn item_card_body_spacer_height_for_target_and_content(
    target_height: f32,
    content_height: f32,
) -> f32 {
    (target_height - content_height).max(0.0)
}

pub(super) fn item_card_field_row_height() -> f32 {
    ITEM_CARD_FIELD_ROW_HEIGHT
}

pub(super) fn item_card_field_row_vertical_padding() -> f32 {
    ITEM_CARD_FIELD_ROW_VERTICAL_PADDING
}

pub(super) fn item_card_row_block_height() -> f32 {
    ITEM_CARD_FIELD_ROW_HEIGHT + ITEM_CARD_FIELD_ROW_VERTICAL_PADDING * 2.0
}

pub(super) fn item_card_detail_column_gap() -> f32 {
    ITEM_CARD_DETAIL_COLUMN_GAP
}

pub(super) fn item_card_detail_row_gap() -> f32 {
    ITEM_CARD_DETAIL_ROW_GAP
}

pub(super) fn item_card_column_gap() -> f32 {
    ITEM_CARD_COLUMN_GAP
}

pub(super) fn item_card_action_column_width_for_row_height(row_height: f32) -> f32 {
    row_height
}

pub(super) fn item_card_field_row_total_size_for_available_width(
    available_width: f32,
    row_height: f32,
) -> egui::Vec2 {
    egui::vec2(
        available_width,
        row_height + ITEM_CARD_FIELD_ROW_VERTICAL_PADDING * 2.0,
    )
}

pub(super) fn item_card_remainder_column_minimum_width() -> f32 {
    ITEM_CARD_REMAINDER_COLUMN_MINIMUM_WIDTH
}

pub(super) fn item_card_remainder_column_maximum_width() -> f32 {
    ITEM_CARD_REMAINDER_COLUMN_MAXIMUM_WIDTH
}

pub(super) fn item_card_zero_remainder_column_minimum_width() -> f32 {
    ITEM_CARD_ZERO_REMAINDER_COLUMN_MINIMUM_WIDTH
}

pub(super) fn item_card_label_inner_width_for_available_width(available_width: f32) -> f32 {
    (available_width - ITEM_CARD_LABEL_TO_VALUE_HORIZONTAL_GAP).max(0.0)
}

pub(super) fn item_card_delete_icon_size() -> f32 {
    ITEM_CARD_DELETE_ICON_SIZE
}

pub(super) fn item_card_icon_button_corner_radius() -> f32 {
    ITEM_CARD_ICON_BUTTON_CORNER_RADIUS
}

pub(super) fn item_card_download_icon_size_for_row_height(row_height: f32) -> f32 {
    (row_height - ITEM_CARD_DOWNLOAD_ICON_HEIGHT_REDUCTION_FROM_ROW_HEIGHT)
        .max(ITEM_CARD_DOWNLOAD_ICON_SIZE_MINIMUM)
}

pub(super) fn item_card_duration_badge_padding() -> egui::Vec2 {
    egui::vec2(
        ITEM_CARD_DURATION_BADGE_PADDING_X,
        ITEM_CARD_DURATION_BADGE_PADDING_Y,
    )
}

pub(super) fn item_card_duration_badge_rect_for_container_and_content_size(
    container: egui::Rect,
    badge_size: egui::Vec2,
) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(
            container.right() - badge_size.x - ITEM_CARD_DURATION_BADGE_RIGHT_INSET,
            container.bottom() - badge_size.y - ITEM_CARD_DURATION_BADGE_BOTTOM_INSET,
        ),
        badge_size,
    )
}

pub(super) fn item_card_duration_badge_corner_radius() -> f32 {
    ITEM_CARD_DURATION_BADGE_CORNER_RADIUS
}

pub(super) fn item_card_field_corner_radius() -> f32 {
    ITEM_CARD_FIELD_CORNER_RADIUS
}

pub(super) fn item_card_output_container_picker_width() -> f32 {
    ITEM_CARD_OUTPUT_CONTAINER_PICKER_WIDTH
}

pub(super) fn item_card_field_text_available_width(rect: egui::Rect) -> f32 {
    (rect.width() - ITEM_CARD_FIELD_TEXT_WIDTH_REDUCTION_FROM_INSET).max(0.0)
}

pub(super) fn item_card_field_text_position_for_galley(
    rect: egui::Rect,
    galley_size: egui::Vec2,
) -> egui::Pos2 {
    egui::pos2(
        rect.min.x + ITEM_CARD_FIELD_TEXT_HORIZONTAL_INSET,
        rect.center().y - galley_size.y * 0.5,
    )
}

pub(super) fn item_card_field_text_edit_margin() -> egui::Margin {
    egui::Margin::symmetric(
        ITEM_CARD_FIELD_TEXT_EDIT_MARGIN_X,
        ITEM_CARD_FIELD_TEXT_EDIT_MARGIN_Y,
    )
}

pub(super) fn item_card_status_message_stroke_width() -> f32 {
    standard_paint_hairline_stroke_width()
}

pub(super) fn standard_interactive_control_height_from_current_ui_metrics(ui: &Ui) -> f32 {
    ui.spacing().interact_size.y
}

pub(super) fn standard_action_button_width_for_visible_text(ui: &Ui, label: &str) -> f32 {
    let horizontal_padding = ui.spacing().button_padding.x * 2.0;
    let minimum_control_width = ui.spacing().interact_size.x;
    WidthRange::new(minimum_control_width, f32::INFINITY).clamp(
        text_width(ui, label, TextStyle::Button)
            + horizontal_padding
            + STANDARD_BUTTON_WIDTH_GUARD_FROM_TEXT_METRICS,
    )
}

pub(super) fn standard_icon_size_from_current_control_metrics(ui: &Ui) -> f32 {
    ui.spacing().interact_size.y * STANDARD_ICON_SIZE_SCALE_FROM_CONTROL_HEIGHT
}

pub(super) fn standard_icon_text_button_width_for_visible_text(ui: &Ui, label: &str) -> f32 {
    standard_action_button_width_for_visible_text(ui, label)
        + standard_icon_size_from_current_control_metrics(ui)
        + ui.spacing().icon_spacing
}

pub(super) fn standard_paint_square_corner_radius() -> f32 {
    STANDARD_PAINT_SQUARE_CORNER_RADIUS
}

pub(super) fn standard_paint_hairline_stroke_width() -> f32 {
    STANDARD_PAINT_HAIRLINE_STROKE_WIDTH
}

pub(super) fn prompt_action_button_width_for_icon_and_visible_text(ui: &Ui, label: &str) -> f32 {
    (standard_icon_text_button_width_for_visible_text(ui, label) + ui.spacing().button_padding.x)
        .max(STANDARD_ICON_TEXT_BUTTON_WIDTH_MINIMUM)
}

pub(super) fn prompt_action_button_height_for_playlist_decision() -> f32 {
    PLAYLIST_PROMPT_ACTION_BUTTON_HEIGHT
}

pub(super) fn modal_prompt_center_anchor_vector() -> egui::Vec2 {
    egui::Vec2::ZERO
}

pub(super) fn remaining_width_before_right_aligned_prompt_actions(
    prompt_width: f32,
    actions_width: f32,
) -> f32 {
    (prompt_width - actions_width).max(0.0)
}

pub(super) fn remaining_width_before_centered_prompt_content(
    prompt_width: f32,
    content_width: f32,
) -> f32 {
    ((prompt_width - content_width) * 0.5).max(0.0)
}

pub(super) fn settings_scroll_content_trailing_safe_gap(ui: &Ui) -> f32 {
    SETTINGS_SCROLL_CONTENT_TRAILING_SAFE_GAP.min(ui.available_width().max(0.0))
}

pub(super) fn settings_form_label_column_width_for_visible_texts(ui: &Ui, labels: &[&str]) -> f32 {
    max_text_width(ui, labels.iter().copied(), TextStyle::Body)
        + ui.spacing().item_spacing.x * SETTINGS_FORM_LABEL_COLUMN_HORIZONTAL_PADDING_MULTIPLIER
}

pub(super) fn settings_form_label_column_width_for_translated_label_keys(
    ui: &Ui,
    state: &AppState,
    keys: &[&'static str],
) -> f32 {
    let labels = keys
        .iter()
        .map(|key| state.ui_i18n_text_for_key(key))
        .collect::<Vec<_>>();
    settings_form_label_column_width_for_visible_texts(ui, &labels)
}

pub(super) fn item_card_visible_label_width_for_translated_label_keys(
    ui: &Ui,
    state: &AppState,
    keys: &[&'static str],
) -> f32 {
    keys.iter()
        .map(|key| state.ui_i18n_text_for_key(key))
        .map(|label| text_width(ui, label, TextStyle::Body))
        .fold(0.0, f32::max)
}

pub(super) fn settings_form_row_vertical_spacing_from_current_text_metrics(ui: &Ui) -> f32 {
    let body_text_height = ui.text_style_height(&TextStyle::Body);
    let standard_control_height = ui.spacing().interact_size.y;
    ((standard_control_height - body_text_height)
        * SETTINGS_FORM_ROW_VERTICAL_SPACING_SCALE_FROM_CONTROL_TEXT_DELTA)
        .clamp(
            SETTINGS_FORM_ROW_VERTICAL_SPACING_MINIMUM,
            SETTINGS_FORM_ROW_VERTICAL_SPACING_MAXIMUM,
        )
}

pub(super) fn settings_form_section_before_title_vertical_spacing() -> f32 {
    SETTINGS_FORM_SECTION_BEFORE_TITLE_VERTICAL_SPACING
}

pub(super) fn settings_form_section_after_separator_vertical_spacing() -> f32 {
    SETTINGS_FORM_SECTION_AFTER_SEPARATOR_VERTICAL_SPACING
}

pub(super) fn settings_form_section_after_body_vertical_spacing() -> f32 {
    SETTINGS_FORM_SECTION_AFTER_BODY_VERTICAL_SPACING
}

pub(super) fn download_conversion_detail_header_to_body_vertical_spacing() -> f32 {
    DOWNLOAD_CONVERSION_DETAIL_HEADER_TO_BODY_VERTICAL_SPACING
}

pub(super) fn advance_form_standard_text_field_width() -> f32 {
    ADVANCE_FORM_STANDARD_TEXT_FIELD_WIDTH
}

pub(super) fn music_download_prompt_item_spacing() -> egui::Vec2 {
    egui::vec2(
        MUSIC_DOWNLOAD_PROMPT_ITEM_HORIZONTAL_SPACING,
        MUSIC_DOWNLOAD_PROMPT_ITEM_VERTICAL_SPACING,
    )
}

pub(super) fn music_download_prompt_action_to_panel_vertical_spacing() -> f32 {
    MUSIC_DOWNLOAD_PROMPT_ACTION_TO_PANEL_VERTICAL_SPACING
}

pub(super) fn music_download_prompt_preference_panel_vertical_spacing() -> f32 {
    MUSIC_DOWNLOAD_PROMPT_PREFERENCE_PANEL_VERTICAL_SPACING
}

pub(super) fn music_download_prompt_title_width_for_audio_label(ui: &Ui, label: &str) -> f32 {
    measured_text_width(
        ui,
        std::iter::once(label),
        TextStyle::Button,
        0.0,
        WidthRange::new(
            MUSIC_DOWNLOAD_PROMPT_TITLE_WIDTH_MINIMUM,
            MUSIC_DOWNLOAD_PROMPT_TITLE_WIDTH_MAXIMUM,
        ),
    )
}

pub(super) fn music_download_prompt_choice_chip_width_for_visible_label(
    ui: &Ui,
    label: &str,
) -> f32 {
    measured_text_width(
        ui,
        std::iter::once(label),
        TextStyle::Button,
        MUSIC_DOWNLOAD_PROMPT_CHOICE_CHIP_HORIZONTAL_PADDING * 2.0,
        WidthRange::new(
            MUSIC_DOWNLOAD_PROMPT_CHOICE_CHIP_WIDTH_MINIMUM,
            MUSIC_DOWNLOAD_PROMPT_CHOICE_CHIP_WIDTH_MAXIMUM,
        ),
    )
}

pub(super) fn music_download_prompt_choice_chip_height() -> f32 {
    MUSIC_DOWNLOAD_PROMPT_CHOICE_CHIP_HEIGHT
}

pub(super) fn music_download_prompt_choice_chip_horizontal_spacing() -> f32 {
    MUSIC_DOWNLOAD_PROMPT_CHOICE_CHIP_HORIZONTAL_SPACING
}

pub(super) fn music_download_prompt_action_button_horizontal_spacing() -> f32 {
    MUSIC_DOWNLOAD_PROMPT_ACTION_BUTTON_HORIZONTAL_SPACING
}

pub(super) fn music_download_prompt_maximum_width_for_viewport(viewport_content_width: f32) -> f32 {
    (viewport_content_width - MODAL_PROMPT_VIEWPORT_HORIZONTAL_SAFE_MARGIN)
        .max(MUSIC_DOWNLOAD_PROMPT_MINIMUM_WIDTH)
}

pub(super) fn music_download_prompt_width_from_content_width(
    content_width: f32,
    max_width: f32,
) -> f32 {
    content_width.clamp(MUSIC_DOWNLOAD_PROMPT_MINIMUM_WIDTH, max_width)
}

pub(super) fn playlist_prompt_content_width() -> f32 {
    PLAYLIST_PROMPT_CONTENT_WIDTH
}

pub(super) fn playlist_prompt_item_spacing() -> egui::Vec2 {
    egui::vec2(
        PLAYLIST_PROMPT_ITEM_HORIZONTAL_SPACING,
        PLAYLIST_PROMPT_ITEM_VERTICAL_SPACING,
    )
}

pub(super) fn playlist_prompt_button_padding() -> egui::Vec2 {
    egui::vec2(
        PLAYLIST_PROMPT_BUTTON_HORIZONTAL_PADDING,
        PLAYLIST_PROMPT_BUTTON_VERTICAL_PADDING,
    )
}

pub(super) fn playlist_prompt_actions_to_body_vertical_spacing() -> f32 {
    PLAYLIST_PROMPT_ACTIONS_TO_BODY_VERTICAL_SPACING
}

pub(super) fn playlist_prompt_body_vertical_spacing() -> f32 {
    PLAYLIST_PROMPT_BODY_VERTICAL_SPACING
}

pub(super) fn playlist_prompt_title_text_size() -> f32 {
    PLAYLIST_PROMPT_TITLE_TEXT_SIZE
}

pub(super) fn playlist_prompt_body_text_size() -> f32 {
    PLAYLIST_PROMPT_BODY_TEXT_SIZE
}

pub(super) fn playlist_prompt_action_button_horizontal_spacing() -> f32 {
    PLAYLIST_PROMPT_ACTION_BUTTON_HORIZONTAL_SPACING
}

pub(super) fn playlist_prompt_choice_corner_radius() -> f32 {
    PLAYLIST_PROMPT_CHOICE_SELECTED_CORNER_RADIUS
}

pub(super) fn playlist_prompt_choice_selected_stroke_width() -> f32 {
    PLAYLIST_PROMPT_CHOICE_SELECTED_STROKE_WIDTH
}

pub(super) fn options_language_detail_header_to_body_vertical_spacing() -> f32 {
    OPTIONS_LANGUAGE_DETAIL_HEADER_TO_BODY_VERTICAL_SPACING
}

pub(super) fn options_language_checkmark_column_width() -> f32 {
    OPTIONS_LANGUAGE_CHECKMARK_COLUMN_WIDTH
}

pub(super) fn tool_path_row_control_horizontal_spacing() -> f32 {
    TOOL_PATH_ROW_CONTROL_HORIZONTAL_SPACING
}

pub(super) fn tool_path_row_standard_control_height_from_current_ui_metrics(ui: &Ui) -> f32 {
    ui.spacing().interact_size.y
}

pub(super) fn tool_path_row_path_text_field_width_for_available_width_and_buttons(
    available_width: f32,
    install_button_width: f32,
    pick_button_width: f32,
    horizontal_spacing: f32,
) -> f32 {
    let buttons_width = install_button_width + pick_button_width;
    let gap_width = horizontal_spacing * 2.0;
    (available_width - buttons_width - gap_width).max(TOOL_PATH_ROW_MINIMUM_PATH_TEXT_FIELD_WIDTH)
}

pub(super) fn prepare_bottom_action_row_height_from_current_control_metrics(ui: &Ui) -> f32 {
    ui.spacing().interact_size.y + PREPARE_BOTTOM_ACTION_ROW_EXTRA_HEIGHT_FROM_CONTROL_METRICS
}

pub(super) fn prepare_title_text_size() -> f32 {
    PREPARE_TITLE_TEXT_SIZE
}

pub(super) fn prepare_body_text_size() -> f32 {
    PREPARE_BODY_TEXT_SIZE
}

pub(super) fn prepare_small_text_size() -> f32 {
    PREPARE_SMALL_TEXT_SIZE
}

pub(super) fn prepare_bottom_action_button_horizontal_spacing() -> f32 {
    PREPARE_BOTTOM_ACTION_BUTTON_HORIZONTAL_SPACING
}

pub(super) fn prepare_root_language_to_header_vertical_spacing() -> f32 {
    PREPARE_ROOT_LANGUAGE_TO_HEADER_VERTICAL_SPACING
}

pub(super) fn prepare_primary_section_vertical_spacing() -> f32 {
    PREPARE_ROOT_HEADER_TO_TOOL_ROWS_VERTICAL_SPACING
}

pub(super) fn prepare_language_detail_header_to_choices_vertical_spacing() -> f32 {
    PREPARE_LANGUAGE_DETAIL_HEADER_TO_CHOICES_VERTICAL_SPACING
}

pub(super) fn prepare_language_choice_checkmark_column_width() -> f32 {
    PREPARE_LANGUAGE_CHOICE_CHECKMARK_COLUMN_WIDTH
}

pub(super) fn prepare_tool_rows_content_left_indent() -> f32 {
    PREPARE_TOOL_ROWS_CONTENT_LEFT_INDENT
}

pub(super) fn prepare_tool_row_height_from_current_control_metrics(ui: &Ui) -> f32 {
    ui.spacing().interact_size.y + PREPARE_TOOL_ROW_EXTRA_HEIGHT_FROM_CONTROL_METRICS
}

pub(super) fn prepare_tool_row_icon_width_from_row_height(row_height: f32) -> f32 {
    row_height * PREPARE_TOOL_ROW_ICON_WIDTH_SCALE_FROM_ROW_HEIGHT
}

pub(super) fn prepare_tool_row_name_width_for_visible_tool_names(ui: &Ui) -> f32 {
    settings_form_label_column_width_for_visible_texts(ui, &["yt-dlp", "Deno", "FFmpeg"])
}

pub(super) fn prepare_tool_row_severity_width_for_visible_labels(ui: &Ui, labels: &[&str]) -> f32 {
    settings_form_label_column_width_for_visible_texts(ui, labels)
}

pub(super) fn prepare_tool_row_status_width_for_visible_labels(ui: &Ui, labels: &[&str]) -> f32 {
    settings_form_label_column_width_for_visible_texts(ui, labels)
}

pub(super) fn prepare_tool_row_action_width_for_visible_labels(ui: &Ui, labels: &[&str]) -> f32 {
    labels
        .iter()
        .map(|label| standard_icon_text_button_width_for_visible_text(ui, label))
        .fold(0.0, f32::max)
        + ui.spacing().button_padding.x * PREPARE_TOOL_ROW_ACTION_WIDTH_EXTRA_FROM_BUTTON_PADDING
}

pub(super) fn prepare_tool_row_name_width_with_following_column_spacing(
    ui: &Ui,
    name_width: f32,
) -> f32 {
    name_width + ui.spacing().item_spacing.x * PREPARE_TOOL_ROW_NAME_TO_SEVERITY_GAP_MULTIPLIER
}

pub(super) fn prepare_tool_row_width_for_columns(
    ui: &Ui,
    icon_width: f32,
    name_width: f32,
    severity_width: f32,
    action_width: f32,
    status_width: f32,
) -> f32 {
    let gap = ui.spacing().item_spacing.x;
    icon_width
        + gap * PREPARE_TOOL_ROW_ICON_TO_NAME_GAP_MULTIPLIER
        + name_width
        + if severity_width > 0.0 {
            gap * PREPARE_TOOL_ROW_NAME_TO_SEVERITY_GAP_MULTIPLIER + severity_width
        } else {
            gap
        }
        + if action_width > 0.0 {
            gap * PREPARE_TOOL_ROW_SEVERITY_TO_ACTION_GAP_MULTIPLIER + action_width + gap
        } else {
            0.0
        }
        + status_width
}

pub(super) fn prepare_tool_row_vertical_spacing_after_each_row(ui: &Ui) -> f32 {
    ui.spacing().item_spacing.y + PREPARE_TOOL_ROW_EXTRA_VERTICAL_SPACING_FROM_ITEM_SPACING
}

pub(super) fn prepare_environment_issues_header_to_rows_vertical_spacing() -> f32 {
    PREPARE_ENVIRONMENT_ISSUES_HEADER_TO_ROWS_VERTICAL_SPACING
}

pub(super) fn prepare_environment_issues_between_rows_vertical_spacing() -> f32 {
    PREPARE_ENVIRONMENT_ISSUES_BETWEEN_ROWS_VERTICAL_SPACING
}

pub(super) fn prepare_environment_issue_status_column_width_from_current_control_metrics(
    ui: &Ui,
) -> f32 {
    ui.spacing().interact_size.y
        * PREPARE_ENVIRONMENT_ISSUE_STATUS_COLUMN_WIDTH_SCALE_FROM_CONTROL_HEIGHT
}

pub(super) fn format_picker_empty_message_top_vertical_spacing() -> f32 {
    FORMAT_PICKER_EMPTY_MESSAGE_TOP_VERTICAL_SPACING
}

pub(super) fn format_picker_section_row_height() -> f32 {
    FORMAT_PICKER_SECTION_ROW_HEIGHT
}

pub(super) fn format_picker_time_range_timeline_height() -> f32 {
    FORMAT_PICKER_TIME_RANGE_TIMELINE_HEIGHT
}

pub(super) fn format_picker_time_range_timeline_horizontal_inset() -> f32 {
    FORMAT_PICKER_TIME_RANGE_TIMELINE_HORIZONTAL_INSET
}

pub(super) fn format_picker_time_range_timeline_vertical_inset() -> f32 {
    FORMAT_PICKER_TIME_RANGE_TIMELINE_VERTICAL_INSET
}

pub(super) fn format_picker_time_range_track_stroke_width() -> f32 {
    FORMAT_PICKER_TIME_RANGE_TRACK_STROKE_WIDTH
}

pub(super) fn format_picker_time_range_chapter_marker_radius() -> f32 {
    FORMAT_PICKER_TIME_RANGE_CHAPTER_MARKER_RADIUS
}

pub(super) fn format_picker_time_range_playhead_stroke_width() -> f32 {
    FORMAT_PICKER_TIME_RANGE_PLAYHEAD_STROKE_WIDTH
}

pub(super) fn format_picker_time_range_playhead_radius() -> f32 {
    FORMAT_PICKER_TIME_RANGE_PLAYHEAD_RADIUS
}

pub(super) fn format_picker_time_range_segment_half_height() -> f32 {
    FORMAT_PICKER_TIME_RANGE_SEGMENT_HALF_HEIGHT
}

pub(super) fn format_picker_time_range_segment_corner_radius() -> f32 {
    FORMAT_PICKER_TIME_RANGE_SEGMENT_CORNER_RADIUS
}

pub(super) fn format_picker_time_range_boundary_notch_half_height() -> f32 {
    FORMAT_PICKER_TIME_RANGE_BOUNDARY_NOTCH_HALF_HEIGHT
}

pub(super) fn format_picker_time_range_snap_distance_pixels() -> f32 {
    FORMAT_PICKER_TIME_RANGE_SNAP_DISTANCE_PIXELS
}

pub(super) fn format_picker_time_range_marker_hit_radius_scale() -> f32 {
    FORMAT_PICKER_TIME_RANGE_MARKER_HIT_RADIUS_SCALE
}

pub(super) fn format_picker_time_range_marker_label_font_size() -> f32 {
    FORMAT_PICKER_TIME_RANGE_MARKER_LABEL_FONT_SIZE
}

pub(super) fn format_picker_time_range_timestamp_font_size() -> f32 {
    FORMAT_PICKER_TIME_RANGE_TIMESTAMP_FONT_SIZE
}

pub(super) fn format_picker_filter_button_padding() -> egui::Vec2 {
    egui::vec2(
        FORMAT_PICKER_FILTER_BUTTON_HORIZONTAL_PADDING,
        FORMAT_PICKER_FILTER_BUTTON_VERTICAL_PADDING,
    )
}

pub(super) fn format_picker_filter_node_height_from_current_control_metrics(ui: &Ui) -> f32 {
    (ui.spacing().interact_size.y - FORMAT_PICKER_FILTER_NODE_HEIGHT_REDUCTION_FROM_CONTROL_METRICS)
        .clamp(
            FORMAT_PICKER_FILTER_NODE_HEIGHT_MINIMUM,
            FORMAT_PICKER_FILTER_NODE_HEIGHT_MAXIMUM,
        )
}

pub(super) fn format_picker_filter_viewport_width_from_available_width(
    available_width: f32,
) -> f32 {
    (available_width - FORMAT_PICKER_FILTER_VIEWPORT_HORIZONTAL_SAFE_MARGIN).max(1.0)
}

pub(super) fn format_picker_filter_slot_width_for_viewport_width_and_stage_count(
    viewport_width: f32,
    stage_count: usize,
) -> f32 {
    (viewport_width / stage_count.max(1) as f32).max(1.0)
}

pub(super) fn format_picker_filter_stage_width_for_slot_width(
    measured_stage_width: f32,
    slot_width: f32,
) -> f32 {
    measured_stage_width.min(
        (slot_width - FORMAT_PICKER_FILTER_STAGE_SLOT_SAFE_MARGIN)
            .max(FORMAT_PICKER_FILTER_STAGE_WIDTH_MINIMUM_WITHIN_SLOT),
    )
}

pub(super) fn format_picker_filter_stage_title_to_nodes_vertical_spacing() -> f32 {
    FORMAT_PICKER_FILTER_STAGE_TITLE_TO_NODES_VERTICAL_SPACING
}

pub(super) fn format_picker_filter_node_vertical_spacing() -> f32 {
    FORMAT_PICKER_FILTER_NODE_VERTICAL_SPACING
}

pub(super) fn format_picker_filter_connection_stroke_width() -> f32 {
    FORMAT_PICKER_FILTER_CONNECTION_STROKE_WIDTH
}

pub(super) fn format_picker_header_height_from_current_control_metrics(ui: &Ui) -> f32 {
    ui.spacing().interact_size.y + FORMAT_PICKER_HEADER_EXTRA_HEIGHT_FROM_CONTROL_METRICS
}

pub(super) fn xaml_format_picker_header_row_contract_from_current_control_metrics(
    ui: &Ui,
) -> super::xaml_layout_contracts::SingleLineControlRowContract {
    xaml_single_line_control_row_contract_from_height(
        format_picker_header_height_from_current_control_metrics(ui),
    )
}

pub(super) fn format_picker_header_summary_width_for_visible_text(ui: &Ui, summary: &str) -> f32 {
    text_width(ui, summary, TextStyle::Body)
        + ui.spacing().item_spacing.x
            * FORMAT_PICKER_HEADER_SUMMARY_EXTRA_WIDTH_MULTIPLIER_FROM_ITEM_SPACING
}

pub(super) fn format_picker_header_center_title_width_for_visible_text(
    ui: &Ui,
    title: &str,
) -> f32 {
    text_width(ui, title, TextStyle::Body)
        + ui.spacing().item_spacing.x
            * FORMAT_PICKER_HEADER_CENTER_TITLE_EXTRA_WIDTH_MULTIPLIER_FROM_ITEM_SPACING
}

pub(super) fn format_picker_filter_stage_node_width_for_visible_values<'a>(
    ui: &Ui,
    values: impl IntoIterator<Item = &'a str>,
    horizontal_padding: f32,
) -> f32 {
    measured_text_width(
        ui,
        values,
        TextStyle::Button,
        horizontal_padding * 2.0 + FORMAT_PICKER_FILTER_STAGE_EXTRA_WIDTH_FROM_TEXT_METRICS,
        WidthRange::new(
            FORMAT_PICKER_FILTER_STAGE_NODE_WIDTH_MINIMUM,
            FORMAT_PICKER_FILTER_STAGE_NODE_WIDTH_MAXIMUM,
        ),
    )
}

pub(super) fn format_picker_table_marker_column_width() -> f32 {
    FORMAT_PICKER_TABLE_MARKER_COLUMN_WIDTH
}

pub(super) fn format_picker_table_resolution_column_width_for_header_and_values<'a>(
    ui: &Ui,
    header: &str,
    values: impl IntoIterator<Item = &'a str>,
) -> f32 {
    format_picker_table_column_width_for_header_and_values(
        ui,
        header,
        values,
        FORMAT_PICKER_TABLE_RESOLUTION_COLUMN_WIDTH_MINIMUM,
        FORMAT_PICKER_TABLE_RESOLUTION_COLUMN_WIDTH_MAXIMUM,
    )
}

pub(super) fn format_picker_table_dynamic_range_column_width_for_header_and_values<'a>(
    ui: &Ui,
    header: &str,
    values: impl IntoIterator<Item = &'a str>,
) -> f32 {
    format_picker_table_column_width_for_header_and_values(
        ui,
        header,
        values,
        FORMAT_PICKER_TABLE_DYNAMIC_RANGE_COLUMN_WIDTH_MINIMUM,
        FORMAT_PICKER_TABLE_DYNAMIC_RANGE_COLUMN_WIDTH_MAXIMUM,
    )
}

pub(super) fn format_picker_table_fps_column_width_for_header_and_values<'a>(
    ui: &Ui,
    header: &str,
    values: impl IntoIterator<Item = &'a str>,
) -> f32 {
    format_picker_table_column_width_for_header_and_values(
        ui,
        header,
        values,
        FORMAT_PICKER_TABLE_FPS_COLUMN_WIDTH_MINIMUM,
        FORMAT_PICKER_TABLE_FPS_COLUMN_WIDTH_MAXIMUM,
    )
}

pub(super) fn format_picker_table_sample_rate_column_width_for_header_and_values<'a>(
    ui: &Ui,
    header: &str,
    values: impl IntoIterator<Item = &'a str>,
) -> f32 {
    format_picker_table_column_width_for_header_and_values(
        ui,
        header,
        values,
        FORMAT_PICKER_TABLE_SAMPLE_RATE_COLUMN_WIDTH_MINIMUM,
        FORMAT_PICKER_TABLE_SAMPLE_RATE_COLUMN_WIDTH_MAXIMUM,
    )
}

pub(super) fn format_picker_table_video_codec_column_width_for_header_and_values<'a>(
    ui: &Ui,
    header: &str,
    values: impl IntoIterator<Item = &'a str>,
) -> f32 {
    format_picker_table_column_width_for_header_and_values(
        ui,
        header,
        values,
        FORMAT_PICKER_TABLE_VIDEO_CODEC_COLUMN_WIDTH_MINIMUM,
        FORMAT_PICKER_TABLE_CODEC_COLUMN_WIDTH_MAXIMUM,
    )
}

pub(super) fn format_picker_table_audio_codec_column_width_for_header_and_values<'a>(
    ui: &Ui,
    header: &str,
    values: impl IntoIterator<Item = &'a str>,
) -> f32 {
    format_picker_table_column_width_for_header_and_values(
        ui,
        header,
        values,
        FORMAT_PICKER_TABLE_AUDIO_CODEC_COLUMN_WIDTH_MINIMUM,
        FORMAT_PICKER_TABLE_CODEC_COLUMN_WIDTH_MAXIMUM,
    )
}

pub(super) fn format_picker_table_filesize_column_width_for_header_and_values<'a>(
    ui: &Ui,
    header: &str,
    values: impl IntoIterator<Item = &'a str>,
) -> f32 {
    format_picker_table_column_width_for_header_and_values(
        ui,
        header,
        values,
        FORMAT_PICKER_TABLE_FILESIZE_COLUMN_WIDTH_MINIMUM,
        FORMAT_PICKER_TABLE_FILESIZE_COLUMN_WIDTH_MAXIMUM,
    )
}

pub(super) fn format_picker_muxed_marker_icon_size() -> f32 {
    FORMAT_PICKER_MUXED_MARKER_ICON_SIZE
}

fn format_picker_table_column_width_for_header_and_values<'a>(
    ui: &Ui,
    header: &str,
    values: impl IntoIterator<Item = &'a str>,
    minimum_width: f32,
    maximum_width: f32,
) -> f32 {
    measured_column_width(
        ui,
        header,
        values,
        TextStyle::Body,
        ui.spacing().item_spacing.x * 2.0
            + FORMAT_PICKER_TABLE_COLUMN_EXTRA_WIDTH_FROM_TEXT_METRICS,
        WidthRange::new(minimum_width, maximum_width),
    )
}

pub(super) fn titlebar_escape_menu_width_for_visible_labels<'a>(
    ui: &Ui,
    labels: impl IntoIterator<Item = &'a str>,
) -> f32 {
    let horizontal_padding =
        ui.spacing().button_padding.x * 2.0 + TITLEBAR_ESCAPE_MENU_WIDTH_GUARD_FROM_TEXT_METRICS;
    measured_text_width(
        ui,
        labels,
        TextStyle::Button,
        horizontal_padding,
        WidthRange::new(ui.spacing().interact_size.x, f32::INFINITY),
    )
}

pub(super) fn titlebar_height() -> f32 {
    TITLEBAR_HEIGHT
}

pub(super) fn titlebar_app_icon_cell_width() -> f32 {
    TITLEBAR_APP_ICON_CELL_WIDTH
}

pub(super) fn titlebar_app_icon_left_margin() -> f32 {
    TITLEBAR_APP_ICON_LEFT_MARGIN
}

pub(super) fn titlebar_app_icon_size() -> f32 {
    TITLEBAR_APP_ICON_SIZE
}

pub(super) fn titlebar_title_left_padding() -> f32 {
    TITLEBAR_TITLE_LEFT_PADDING
}

pub(super) fn titlebar_window_button_width() -> f32 {
    TITLEBAR_WINDOW_BUTTON_WIDTH
}

pub(super) fn titlebar_window_button_icon_size() -> f32 {
    TITLEBAR_WINDOW_BUTTON_ICON_SIZE
}

pub(super) fn titlebar_context_action_button_width() -> f32 {
    TITLEBAR_ESCAPE_BUTTON_WIDTH
}

pub(super) fn titlebar_home_icon_size() -> f32 {
    TITLEBAR_HOME_ICON_SIZE
}

pub(super) fn titlebar_update_signal_icon_size() -> f32 {
    TITLEBAR_UPDATE_SIGNAL_ICON_SIZE
}

pub(super) fn titlebar_escape_button_width() -> f32 {
    TITLEBAR_ESCAPE_BUTTON_WIDTH
}

pub(super) fn titlebar_escape_icon_size() -> f32 {
    TITLEBAR_ESCAPE_ICON_SIZE
}

pub(super) fn titlebar_app_title_font_size() -> f32 {
    TITLEBAR_APP_TITLE_FONT_SIZE
}

pub(super) fn titlebar_separator_stroke_width() -> f32 {
    standard_paint_hairline_stroke_width()
}

pub(super) fn titlebar_background_corner_radius() -> f32 {
    standard_paint_square_corner_radius()
}

pub(super) fn titlebar_right_client_area_width(
    show_home_button: bool,
    show_escape_menu: bool,
) -> f32 {
    TITLEBAR_WINDOW_BUTTON_WIDTH * 3.0
        + if show_home_button {
            TITLEBAR_ESCAPE_BUTTON_WIDTH
        } else {
            0.0
        }
        + if show_escape_menu {
            TITLEBAR_ESCAPE_BUTTON_WIDTH
        } else {
            0.0
        }
}

pub(super) fn titlebar_menu_item_minimum_width_from_current_control_metrics(ui: &Ui) -> f32 {
    ui.available_width().max(ui.spacing().interact_size.x)
}

pub(super) fn cookie_acquisition_dialog_minimum_width_for_start_phase() -> f32 {
    COOKIE_ACQUISITION_DIALOG_MINIMUM_WIDTH_FOR_START_PHASE
}

pub(super) fn cookie_acquisition_dialog_minimum_width_for_message_phase() -> f32 {
    COOKIE_ACQUISITION_DIALOG_MINIMUM_WIDTH_FOR_MESSAGE_PHASES
}

pub(super) fn cookie_acquisition_dialog_maximum_width_for_viewport(
    viewport_content_width: f32,
    minimum_width: f32,
) -> f32 {
    (viewport_content_width - COOKIE_ACQUISITION_DIALOG_HORIZONTAL_SAFE_MARGIN_FROM_VIEWPORT)
        .clamp(minimum_width, COOKIE_ACQUISITION_DIALOG_MAXIMUM_WIDTH)
}

pub(super) fn cookie_acquisition_dialog_content_width_for_visible_texts<'a>(
    ui: &Ui,
    texts: impl IntoIterator<Item = &'a str>,
    minimum_width: f32,
    maximum_width: f32,
) -> f32 {
    measured_text_width(
        ui,
        texts.into_iter().filter(|text| !text.trim().is_empty()),
        TextStyle::Body,
        COOKIE_ACQUISITION_DIALOG_TEXT_WIDTH_EXTRA_ROOM,
        WidthRange::new(minimum_width, maximum_width),
    )
}

pub(super) fn cookie_acquisition_dialog_action_row_width_for_visible_labels(
    ui: &Ui,
    labels: &[&str],
    minimum_width: f32,
    maximum_width: f32,
) -> f32 {
    if labels.is_empty() {
        return 0.0;
    }

    let horizontal_spacing_between_actions =
        ui.spacing().item_spacing.x * labels.len().saturating_sub(1) as f32;
    let action_widths = labels
        .iter()
        .map(|label| standard_action_button_width_for_visible_text(ui, label))
        .sum::<f32>();

    (action_widths
        + horizontal_spacing_between_actions
        + COOKIE_ACQUISITION_DIALOG_ACTION_ROW_WIDTH_GUARD)
        .clamp(minimum_width, maximum_width)
}

pub(super) fn cookie_acquisition_dialog_field_to_body_vertical_spacing() -> f32 {
    COOKIE_ACQUISITION_DIALOG_FIELD_TO_BODY_VERTICAL_SPACING
}

pub(super) fn cookie_acquisition_dialog_center_anchor_vector() -> egui::Vec2 {
    egui::Vec2::ZERO
}

pub(super) fn cookie_acquisition_dialog_item_spacing() -> egui::Vec2 {
    egui::vec2(
        COOKIE_ACQUISITION_DIALOG_ITEM_HORIZONTAL_SPACING,
        COOKIE_ACQUISITION_DIALOG_ITEM_VERTICAL_SPACING,
    )
}

pub(super) fn cookie_acquisition_dialog_button_padding() -> egui::Vec2 {
    egui::vec2(
        COOKIE_ACQUISITION_DIALOG_BUTTON_HORIZONTAL_PADDING,
        COOKIE_ACQUISITION_DIALOG_BUTTON_VERTICAL_PADDING,
    )
}

pub(super) fn processing_log_table_time_column_width_for_visible_timestamps<'a>(
    ui: &Ui,
    values: impl IntoIterator<Item = &'a str>,
) -> f32 {
    processing_log_table_column_width_for_header_and_values(
        ui,
        "Time",
        values,
        PROCESSING_LOG_TABLE_TIME_COLUMN_WIDTH_MINIMUM,
        PROCESSING_LOG_TABLE_TIME_COLUMN_WIDTH_MAXIMUM,
    )
}

pub(super) fn processing_log_table_status_column_width(ui: &Ui) -> f32 {
    processing_log_table_column_width_for_header_and_values(
        ui,
        "Status",
        ["✓", "✕", "·", "…"],
        PROCESSING_LOG_TABLE_STATUS_COLUMN_WIDTH_MINIMUM,
        PROCESSING_LOG_TABLE_STATUS_COLUMN_WIDTH_MAXIMUM,
    )
}

pub(super) fn processing_log_table_action_column_width_for_visible_actions<'a>(
    ui: &Ui,
    values: impl IntoIterator<Item = &'a str>,
) -> f32 {
    processing_log_table_column_width_for_header_and_values(
        ui,
        "Action",
        values,
        PROCESSING_LOG_TABLE_ACTION_COLUMN_WIDTH_MINIMUM,
        PROCESSING_LOG_TABLE_ACTION_COLUMN_WIDTH_MAXIMUM,
    )
}

pub(super) fn processing_log_table_mode_column_width(ui: &Ui) -> f32 {
    processing_log_table_column_width_for_header_and_values(
        ui,
        "Mode",
        [
            "origin", "audio", "normal", "yt-dlp", "ffmpeg", "ffprobe", "app",
        ],
        PROCESSING_LOG_TABLE_MODE_COLUMN_WIDTH_MINIMUM,
        PROCESSING_LOG_TABLE_MODE_COLUMN_WIDTH_MAXIMUM,
    )
}

pub(super) fn processing_conversion_choice_button_minimum_size_from_current_control_metrics(
    ui: &Ui,
) -> egui::Vec2 {
    egui::vec2(
        0.0,
        ui.spacing().interact_size.y
            + PROCESSING_CONVERSION_CHOICE_BUTTON_EXTRA_HEIGHT_FROM_CONTROL_METRICS,
    )
}

pub(super) fn processing_log_table_header_row_height() -> f32 {
    PROCESSING_LOG_TABLE_HEADER_ROW_HEIGHT
}

pub(super) fn processing_command_viewer_to_log_table_vertical_spacing() -> f32 {
    PROCESSING_COMMAND_VIEWER_TO_LOG_TABLE_VERTICAL_SPACING
}

pub(super) fn processing_log_table_action_row_height() -> f32 {
    PROCESSING_LOG_TABLE_ACTION_ROW_HEIGHT
}

pub(super) fn processing_log_table_step_row_height() -> f32 {
    PROCESSING_LOG_TABLE_STEP_ROW_HEIGHT
}

pub(super) fn processing_log_table_parent_row_height() -> f32 {
    PROCESSING_LOG_TABLE_PARENT_ROW_HEIGHT
}

pub(super) fn processing_log_table_section_separator_row_height() -> f32 {
    PROCESSING_LOG_TABLE_SECTION_SEPARATOR_ROW_HEIGHT
}

pub(super) fn processing_log_table_background_corner_radius() -> f32 {
    standard_paint_square_corner_radius()
}

pub(super) fn processing_log_table_row_separator_stroke_width() -> f32 {
    standard_paint_hairline_stroke_width()
}

pub(super) fn processing_log_table_status_icon_font_size() -> f32 {
    PROCESSING_LOG_TABLE_STATUS_ICON_FONT_SIZE
}

pub(super) fn processing_log_table_text_x_for_alignment(rect: egui::Rect) -> f32 {
    if rect.width() <= PROCESSING_LOG_TABLE_TEXT_WIDE_CELL_THRESHOLD {
        rect.left()
    } else {
        rect.left() + PROCESSING_LOG_TABLE_TEXT_LEFT_PADDING_WHEN_WIDE
    }
}

pub(super) fn processing_log_table_text_font_size(strong: bool) -> f32 {
    if strong {
        PROCESSING_LOG_TABLE_TEXT_STRONG_FONT_SIZE
    } else {
        PROCESSING_LOG_TABLE_TEXT_NORMAL_FONT_SIZE
    }
}

pub(super) fn processing_log_table_text_clip_rect(rect: egui::Rect) -> egui::Rect {
    rect.shrink2(egui::vec2(
        PROCESSING_LOG_TABLE_TEXT_CLIP_HORIZONTAL_INSET,
        0.0,
    ))
}

pub(super) fn processing_command_viewer_font_size() -> f32 {
    PROCESSING_COMMAND_VIEWER_FONT_SIZE
}

pub(super) fn processing_command_viewer_token_spacing() -> egui::Vec2 {
    egui::vec2(
        PROCESSING_COMMAND_VIEWER_TOKEN_SPACING_X,
        PROCESSING_COMMAND_VIEWER_TOKEN_SPACING_Y,
    )
}

pub(super) fn processing_command_viewer_content_width_for_available_width(
    available_width: f32,
) -> f32 {
    (available_width
        - PROCESSING_COMMAND_VIEWER_FRAME_MARGIN_X * 2.0
        - PROCESSING_COMMAND_VIEWER_FRAME_STROKE_WIDTH * 2.0)
        .max(1.0)
}

pub(super) fn processing_command_viewer_natural_height_for_content_height(
    content_height: f32,
) -> f32 {
    content_height
        + PROCESSING_COMMAND_VIEWER_FRAME_MARGIN_Y * 2.0
        + PROCESSING_COMMAND_VIEWER_FRAME_STROKE_WIDTH * 2.0
}

pub(super) fn processing_command_viewer_line_stack_height(
    line_height: f32,
    line_count: usize,
) -> f32 {
    line_height * line_count as f32
        + PROCESSING_COMMAND_VIEWER_TOKEN_SPACING_Y * line_count.saturating_sub(1) as f32
}

pub(super) fn processing_command_viewer_token_spacing_x() -> f32 {
    PROCESSING_COMMAND_VIEWER_TOKEN_SPACING_X
}

pub(super) fn processing_command_viewer_token_spacing_y() -> f32 {
    PROCESSING_COMMAND_VIEWER_TOKEN_SPACING_Y
}

pub(super) fn processing_command_viewer_frame_stroke_width() -> f32 {
    PROCESSING_COMMAND_VIEWER_FRAME_STROKE_WIDTH
}

pub(super) fn processing_command_viewer_frame_corner_radius() -> f32 {
    PROCESSING_COMMAND_VIEWER_FRAME_CORNER_RADIUS
}

pub(super) fn processing_command_viewer_frame_inner_margin() -> egui::Margin {
    egui::Margin::symmetric(
        PROCESSING_COMMAND_VIEWER_FRAME_MARGIN_X as i8,
        PROCESSING_COMMAND_VIEWER_FRAME_MARGIN_Y as i8,
    )
}

fn processing_log_table_column_width_for_header_and_values<'a>(
    ui: &Ui,
    header: &str,
    values: impl IntoIterator<Item = &'a str>,
    minimum_width: f32,
    maximum_width: f32,
) -> f32 {
    measured_column_width(
        ui,
        header,
        values,
        TextStyle::Body,
        ui.spacing().item_spacing.x * 2.0
            + PROCESSING_LOG_TABLE_COLUMN_EXTRA_WIDTH_FROM_TEXT_METRICS,
        WidthRange::new(minimum_width, maximum_width),
    )
}

pub(super) fn xaml_single_line_control_row_contract_from_height(
    height: f32,
) -> super::xaml_layout_contracts::SingleLineControlRowContract {
    super::xaml_layout_contracts::SingleLineControlRowContract::new(height)
}

pub(super) fn xaml_button_intrinsic_size_for_visible_text(
    ui: &Ui,
    label: &str,
) -> super::xaml_layout_contracts::LayoutSize {
    super::xaml_layout_contracts::LayoutSize::new(
        standard_action_button_width_for_visible_text(ui, label),
        standard_interactive_control_height_from_current_ui_metrics(ui),
    )
}

pub(super) fn xaml_button_ui_element_for_visible_text(
    ui: &Ui,
    label: &str,
) -> super::xaml_layout_contracts::UiElement {
    super::xaml_layout_contracts::UiElement::button(xaml_button_intrinsic_size_for_visible_text(
        ui, label,
    ))
}

pub(super) fn xaml_icon_text_button_intrinsic_size_for_visible_text(
    ui: &Ui,
    label: &str,
) -> super::xaml_layout_contracts::LayoutSize {
    super::xaml_layout_contracts::LayoutSize::new(
        standard_icon_text_button_width_for_visible_text(ui, label),
        standard_interactive_control_height_from_current_ui_metrics(ui),
    )
}

pub(super) fn xaml_icon_text_button_ui_element_for_visible_text(
    ui: &Ui,
    label: &str,
) -> super::xaml_layout_contracts::UiElement {
    super::xaml_layout_contracts::UiElement::icon_text_button(
        xaml_icon_text_button_intrinsic_size_for_visible_text(ui, label),
    )
}

pub(super) fn xaml_selectable_button_ui_element_for_visible_text(
    ui: &Ui,
    label: &str,
) -> super::xaml_layout_contracts::UiElement {
    super::xaml_layout_contracts::UiElement::selectable_button(
        xaml_button_intrinsic_size_for_visible_text(ui, label),
    )
}

pub(super) fn xaml_single_line_text_input_ui_element_from_row_contract(
    row: super::xaml_layout_contracts::SingleLineControlRowContract,
) -> super::xaml_layout_contracts::UiElement {
    super::xaml_layout_contracts::UiElement::single_line_text_input(row)
}

pub(super) fn xaml_text_box_ui_element_from_row_contract(
    row: super::xaml_layout_contracts::SingleLineControlRowContract,
) -> super::xaml_layout_contracts::UiElement {
    super::xaml_layout_contracts::UiElement::text_box(row)
}

pub(super) fn xaml_icon_button_ui_element_from_row_contract(
    row: super::xaml_layout_contracts::SingleLineControlRowContract,
) -> super::xaml_layout_contracts::UiElement {
    super::xaml_layout_contracts::UiElement::icon_button_square(row)
}

pub(super) fn xaml_spinner_ui_element_for_square_size(
    size: f32,
) -> super::xaml_layout_contracts::UiElement {
    super::xaml_layout_contracts::UiElement::spinner_square(size)
}

pub(super) fn xaml_label_ui_element_from_row_contract_and_width(
    row: super::xaml_layout_contracts::SingleLineControlRowContract,
    width: f32,
) -> super::xaml_layout_contracts::UiElement {
    super::xaml_layout_contracts::UiElement::label(super::xaml_layout_contracts::LayoutSize::new(
        width, row.height,
    ))
}

pub(super) fn xaml_stretch_width_ui_element_from_row_contract(
    row: super::xaml_layout_contracts::SingleLineControlRowContract,
) -> super::xaml_layout_contracts::UiElement {
    super::xaml_layout_contracts::UiElement::stretch_width_stretch_height(
        super::xaml_layout_contracts::LayoutSize::new(0.0, row.height),
    )
}

pub(super) fn xaml_spacer_ui_element_from_row_contract_and_width(
    row: super::xaml_layout_contracts::SingleLineControlRowContract,
    width: f32,
) -> super::xaml_layout_contracts::UiElement {
    super::xaml_layout_contracts::UiElement::spacer(width, row)
}

pub(super) fn xaml_dialog_action_button_shared_size_group_for_translated_label_keys(
    ui: &Ui,
    state: &AppState,
    keys: &[&'static str],
) -> super::xaml_layout_contracts::SharedSizeGroupContract {
    let sizes = keys
        .iter()
        .map(|key| xaml_button_intrinsic_size_for_visible_text(ui, state.ui_i18n_text_for_key(key)))
        .collect::<Vec<_>>();
    super::xaml_layout_contracts::SharedSizeGroupContract::from_intrinsic_sizes(&sizes)
}

pub(super) fn xaml_dialog_action_row_contract_from_current_control_metrics(
    ui: &Ui,
) -> super::xaml_layout_contracts::SingleLineControlRowContract {
    xaml_single_line_control_row_contract_from_height(
        standard_interactive_control_height_from_current_ui_metrics(ui),
    )
}

pub(super) fn xaml_settings_form_single_line_row_contract_from_current_control_metrics(
    ui: &Ui,
) -> super::xaml_layout_contracts::SingleLineControlRowContract {
    xaml_single_line_control_row_contract_from_height(
        standard_interactive_control_height_from_current_ui_metrics(ui),
    )
}

pub(super) fn xaml_settings_form_row_minimum_height_from_row_contract_and_vertical_spacing(
    row: super::xaml_layout_contracts::SingleLineControlRowContract,
    vertical_spacing: f32,
) -> f32 {
    row.height + vertical_spacing.max(0.0) * 2.0
}
