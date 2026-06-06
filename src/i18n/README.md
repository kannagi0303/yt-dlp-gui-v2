# i18n maintenance rules

This directory is optimized for both human editing and GPT-assisted maintenance.
Keep the structure predictable. Do not let translation work turn into a broad code refactor.

## Boundary

Do not add tooltip-only i18n keys. If an explanation is important, make it visible in the layout; otherwise keep the UI direct.

Translate only user-facing app UI:

- labels, buttons, tabs, menus, and dialogs
- user-facing status text that is intentionally part of the app UI
- user-facing error summaries that are written as UI copy
- the UI chrome around the Log page

Do **not** translate:

- raw `yt-dlp`, `ffmpeg`, `ffprobe`, `deno`, or external tool output
- raw command lines, CLI option names, and CLI argument values
- codec, container, extension, and media format tokens
- config keys, internal enum names, and developer/debug logs
- runtime/internal messages, process errors, diagnostic details, and generated tool-status strings

Short rule: app UI uses i18n; runtime/internal messages use fixed English; external tools and technical tokens stay raw.

## UI translation entrypoint

Use `ui_i18n_text_for_key("...")` / `ui_i18n_text_with_replacements("...", ...)`
for UI-owned text. The name is intentionally explicit: it marks that the current
zone owns stable visible UI copy and may use locale keys.

Do not introduce short aliases such as `tr(...)`, `trf(...)`, `ui_tr(...)`, or
`ui_trf(...)`. Short names make it too easy for GPT/Codex to translate runtime
messages, generated diagnostics, or external tool text by accident.

Per-zone rule:

1. Pick one UI zone or component.
2. Decide which visible labels/buttons/dialog copy that zone owns.
3. Convert only that zone to `ui_i18n_text_for_key` / `ui_i18n_text_with_replacements`.
4. Do not i18n nearby fixed-English status, tool output, codec/container tokens,
   generated summaries, or removed tooltip-style explanations.
5. If the zone needs a new key, add only the directly owned UI text and report the
   key-count change.

This keeps i18n as a local ownership decision, not a whole-file cleanup pass.

## Canonical source

`en_us.rs` is the canonical key source.

When adding a new i18n key:

1. Add it to `en_us.rs` first.
2. Add the same key, in the same position, to every release-coverage locale.
3. Keep release-coverage locale key order identical to `en_us.rs`.
4. Other selectable locales may be incomplete and may fall back to English for missing newer keys.

## Selectable vs release-coverage locales

Language picker policy lives in `catalog.rs`.

All compiled locale tables are shown in the language picker and can be selected manually.
Auto-detection also maps to compiled locales when the system locale matches one of them.

Current selectable locales:

- `ar-MA`
- `de-DE`
- `el-GR`
- `en-US`
- `es-ES`
- `fr-FR`
- `it-IT`
- `ja-JP`
- `ko-KR`
- `pl-PL`
- `pt-BR`
- `ru-RU`
- `uk-UA`
- `zh-CN`
- `zh-TW`

Current release-coverage locales checked strictly against `en_us.rs`:

- `ar-MA`
- `de-DE`
- `el-GR`
- `en-US`
- `es-ES`
- `fr-FR`
- `it-IT`
- `ja-JP`
- `ko-KR`
- `pl-PL`
- `pt-BR`
- `ru-RU`
- `uk-UA`
- `zh-CN`
- `zh-TW`

All selectable locales keep full canonical key coverage. English fallback remains only a safety net for unknown keys, not a normal draft-locale path.

## GPT workflow

For GPT-assisted changes:

1. Do not reorder keys unless the task is specifically key-order cleanup.
2. All compiled locales are release-coverage locales. Keep every locale in canonical key order and avoid reintroducing draft-only missing-key fallback as normal behavior.
3. Do not translate raw tool output, technical tokens, or runtime/internal messages.
4. For release-coverage key checks, use the Rust-only test:
   `cargo test i18n_keys`.
5. For direct UI key usage, use:
   `cargo test i18n_used_keys`.
6. If either test reports existing missing keys, fix those before broad wording polish.

The goal is not to translate every string in the program. The goal is that the released user interface does not mix languages, leak i18n keys, or expose obvious hardcoded UI text. Internal/runtime messages should remain stable fixed English instead of expanding the locale tables.


## Checks

This project intentionally does not use Python or scripts under `tools/` for i18n maintenance.
`tools/` is reserved for runtime tool dependencies such as `yt-dlp`, `ffmpeg`, and related executables.

Rust-only checks live next to the i18n module. The stricter release-locale key coverage test can be run directly during release-oriented i18n cleanup:

```bash
cargo test i18n_keys
cargo test i18n_used_keys
```


`i18n_keys` checks release-coverage locale key structure. It compares every
compiled locale against canonical `en_us.rs`, including key order. It catches
missing, extra, or reordered keys. It does not check translation quality,
hardcoded UI text, or raw external tool output.

`i18n_used_keys` scans direct literal `ui_i18n_text_for_key("...")` and
`ui_i18n_text_with_replacements("...", ...)` calls in Rust source and verifies
that those literals exist in `en_us.rs`. It also scans retired short wrappers if
they are reintroduced. It catches accidental raw text passed to i18n, such as
`state.ui_i18n_text_for_key("Add")`, which would show fixed English in
non-English UI instead of using a real key like `action.add`.

## Fixed-English operation feedback

Do not add `state.*` keys for operation feedback. Short-lived status messages, process progress summaries, tool deployment feedback, cache cleanup results, and generated diagnostics should stay fixed English unless they become stable UI labels/buttons/dialog copy.

Technical tokens such as codecs, containers, file extensions, raw command arguments, and file-dialog filter tokens should also stay fixed English/raw.


## UI-owned key rule

Only strings owned directly by the user-facing UI layer should live in locale tables.
Messages produced by state, config, domain models, tool installers, notifications,
prepare checks, and other non-UI modules should be fixed English unless the UI
layer explicitly owns the wording. This keeps i18n focused on visible controls,
not on internal status/default messages.

## Reviewed UI zones

### Titlebar escape menu

Zone status: reviewed.

Rules:

- App mode menu items are UI-owned and use `ui_i18n_text_for_key(mode.label_key())`.
- `Advanced`, `Options`, and optional `Log` menu items are UI-owned and use `ui_i18n_text_for_key(...)`.
- `Main` is not shown in the escape menu. The titlebar Home icon owns that navigation.
- `Prepare` is not shown in the escape menu. It is treated as a non-titlebar/legacy flow item for this zone.
- App title/product name and icon-only titlebar controls stay fixed/internal and do not use i18n.

### URL input row + primary action button

Zone status: reviewed.

Rules:

- URL input placeholder is UI-owned and uses `ui_i18n_text_for_key(UiText::URL_HINT)`.
- Primary action button text is UI-owned and uses `ui_i18n_text_for_key(state.primary_url_action_label_key())`.
- `state.rs` may decide the current primary action key, but it must not translate the label directly for this zone.
- Primary action icons, URL contents, and icon-only clipboard monitor controls stay fixed/internal and do not use i18n.
- Clipboard monitor hover/tooltip copy is not part of the product UI and should not have locale keys.
- Missing `yt-dlp` callout text is visible UI copy and uses `ui_i18n_text_for_key(...)`.


### Mode switch / main page entry

Zone status: reviewed.

Rules:

- App mode switch labels such as Origin, Standard, and Audio are UI-owned and use `ui_i18n_text_for_key(mode.label_key())`.
- Bottom-row stable controls such as `Output folder` and `Download` are UI-owned and use `ui_i18n_text_for_key(...)`.
- Home, escape-menu, and window control icons are icon-only/internal controls and do not need locale keys from this zone.
- App title/product name, output paths, config mode tokens, and operation feedback messages remain fixed/raw and do not get locale keys from this zone.
- `tab.main` and `tab.prepare` are removed from locale tables. Main navigation is represented by the Home icon, and Prepare uses `prepare.*` keys for its actual screen content rather than an old tab label.
- When bottom-row button text is used for measurement and rendering, resolve the key first into a variable such as `target_text` or `download_text`, then use that translated value for both width calculation and display.

### Standard Mode list

Zone status: reviewed.

Rules:

- Queue summary labels and the `Clear all` button are UI-owned and use `ui_i18n_text_for_key(...)`.
- Empty-list card title, format labels, format guidance copy, and file-name label are UI-owned and use `ui_i18n_text_for_key(...)`.
- Standard item-card format labels, section label, error label, file-name label, and output action menu items are UI-owned and use `ui_i18n_text_for_key(...)`.
- Item title, URL, duration, file name/path, format summary technical tokens, codec/container/extension text, progress numbers, and raw error bodies stay data/raw and do not get locale keys.
- Normal delete/cancel/export arrow controls are icon-only in this zone. `item.remove`, `item.stop_download`, and Standard-list `item.save_as` hover-only/dead text must not be used here.
- `item.opened_output_file` is fixed-English operation feedback, not a UI-owned label, and should not live in locale tables.

### Origin Mode main card

Zone status: reviewed.

Rules:

- Title and description placeholders are UI-owned and use `ui_i18n_text_for_key(...)`.
- Thumbnail fallback text, `Download thumbnail`, and the thumbnail context-menu `Save as` item are UI-owned and use `ui_i18n_text_for_key(...)`.
- Format labels and empty format guidance copy are UI-owned and use `ui_i18n_text_for_key(...)`.
- Right-side metadata labels such as channel, date, and views are UI-owned and use `ui_i18n_text_for_key(...)`.
- Video title, description body, channel/uploader values, URL, date value, view count value, duration badge, hashtag link base URL, thumbnail URL, and file-dialog filter tokens stay data/raw and do not get locale keys.
- Workflow/status lines shown while analyzing/downloading remain runtime/status detail and should not be pulled into UI i18n from this zone.
- When UI text is used for both measurement and rendering, resolve the key first into a variable such as `ui_text` or `ui_value`, then use that translated value for width calculation and display. Never measure the raw key while rendering the localized value.

### Audio Mode list

Zone status: reviewed.

Rules:

- Empty audio-list row title and the right-side `Audio` label are UI-owned and use `ui_i18n_text_for_key(...)`.
- Compact-row status labels such as resolving, buffering, ready, caching, playing, paused, failed, and completed are visible Audio Mode list UI and use `ui_i18n_text_for_key(...)`.
- Audio title, duration value, cache/download percentage, playback progress, thumbnail/cover source, and currently-playing visual markers stay data/raw or visual state and do not get locale keys.
- Play/remove controls inside compact rows are icon-only in this zone and do not need tooltip-only locale keys.
- Music playback bar controls, playback-mode labels, media-session integration, and runtime audio/cache messages belong to separate zones and should not be changed from this list review.



### Music playback controls

Zone status: reviewed.

Rules:

- Previous, play/pause, and next buttons are icon-only controls and must not use locale keys.
- Seek bar, cache/progress fill, time text, volume icon/slider, and playback-mode icon are visual/data controls and do not get locale keys from this zone.
- `music.previous`, `music.play`, `music.pause`, and `music.next` are removed from locale tables because the playback bar does not display those labels.
- `music.status.*` remains owned by the Audio Mode list zone, not by the playback bar.
- Playback-mode operation feedback, media-session integration, and runtime audio/cache errors remain fixed-English/status details unless a future reviewed UI zone promotes them to stable visible UI copy.

### Format Picker

Zone status: reviewed.

Rules:

- Picker header buttons, mode switches, picker titles, empty states, table headers, filter-chain labels, subtitle tabs, subtitle/section explanatory copy, and subtitle table headers are UI-owned and use `ui_i18n_text_for_key(...)`.
- `SubtitlePickerTab` must expose locale keys through `label_key()`, not English display text. The UI resolves those keys with `ui_i18n_text_for_key(...)` before rendering.
- Codec names, containers, file extensions, resolution/FPS/sample-rate values, file sizes, language codes, check markers, and mux/link markers stay data/raw or icon-like and do not get locale keys.
- `selected_format_summary(...)` remains a shared state summary path and is not owned by this zone review.
- When UI text is used for measurement and display, resolve the key into a local variable such as `ui_text`, `filter_text`, or `header_text` first. Use that translated value for width calculation and rendering; never measure a raw key while rendering a localized value.

### Options page

Zone status: reviewed.

Rules:

- Options page section titles, setting labels, checkboxes, buttons, combo-box values, and the language detail page are UI-owned and use `ui_i18n_text_for_key(...)`.
- File action, cache location, theme mode, and theme color combo values must use explicit UI label keys/helpers instead of passing English enum labels into translation lookup.
- Tool names such as `yt-dlp`, `Deno`, `FFmpeg`, and `Aria2`, tool paths, cache paths, executable names, product names, platform names, percentages, and language native names stay fixed/raw data and do not get locale keys from this zone.
- `yt-dlp-gui` and `Windows` remain fixed labels when used as product/platform names in cache location choices.
- Music download prompts and YouTube playlist prompts are separate prompt-dialog zones even if they live in `options_tab.rs`; do not mix them into the Options page review.
- Cache usage display remains a state-owned summary path for now and is not rewritten by this zone review.
- Obsolete tool-install status keys from the old Options page flow are removed from locale tables when no UI path uses them.
- When Options text is used for measurement and rendering, resolve it first into a variable such as `ui_text`, `auto_detect_text`, or `browse_text`, then use that translated value for width calculation and display.

### Prompt dialogs

Zone status: reviewed.

Rules:

- Music download prompt titles, labels, the `Best` preference chip, and prompt action buttons are UI-owned and use `ui_i18n_text_for_key(...)`.
- Audio codec/format chips such as `Opus`, `AAC`, and `MP3` stay fixed technical tokens and do not get locale keys.
- YouTube playlist prompt titles, headings, body copy, and action buttons are UI-owned and use `ui_i18n_text_for_key(...)`.
- High-risk YouTube playlist kind and note text must be exposed as locale keys through `label_key()` / `note_key()` helpers, not raw English enum labels or raw infrastructure notes.
- YouTube URLs, playlist source URLs, dialog IDs, icon-only controls, and product/platform names such as `YouTube` stay data/raw or fixed product names.
- Prompt text used for width measurement must first be resolved into variables such as `cancel_text`, `download_text`, `video_text`, or `playlist_text`; use the same translated value for measurement and rendering.

### Advanced page

Zone status: reviewed.

Rules:

- Advanced page section titles, setting labels, checkboxes, buttons, combo-box values, file-dialog titles, and stable placeholders are UI-owned and use `ui_i18n_text_for_key(...)`.
- `FileTimeMode` must expose locale keys through `label_key()`, not English display text. The UI resolves those keys with `ui_i18n_text_for_key(...)` before rendering.
- The cookies file source option uses the UI-owned key `advance.cookie_source.file`. Browser names such as `Chrome`, `Brave`, `Firefox`, `Edge`, `Opera`, and `Vivaldi` stay fixed product names and do not get locale keys.
- `Aria2`, `yt-dlp`, config paths, cookie file paths, browser profile names, proxy URL examples, command preview snippets, CLI flags, and rate-limit examples such as `2M` / `800K` stay fixed/raw and do not get locale keys from this zone.
- Advanced does not add tooltip-only locale keys. Existing titlebar `hover_text` parameters are internal interaction IDs, not visible tooltip copy.
- Download conversion detail page header controls are UI-owned here, but the inner `render_processing_settings_content(...)` remains owned by the Processing / Log zone and is not rewritten by this zone review.
- When Advanced text is used for measurement, resolve it first into variables such as `config_text`, `file_time_text`, or `download_conversion_text`, then use those translated values for width calculation and display.

### Processing / Log

Zone status: reviewed.

Rules:

- Post-download conversion setting labels such as Video, Audio, Container, Subtitles, Source, Embed, and Burn in are UI-owned and use `ui_i18n_text_for_key(...)`.
- Codec and container choices such as H.264, HEVC, AV1, AAC, Opus, FLAC, MP4, MKV, and MOV stay fixed technical tokens and do not get locale keys from this zone.
- Log Viewer headers, empty log text, action rows, step rows, command viewer contents, tool names, CLI arguments, status icons, and raw command output remain fixed-English/raw diagnostic UI. Do not reintroduce `log.*` locale keys for this zone.
- When processing setting labels are used for measurement, resolve the keys first into variables such as `video_text`, `audio_text`, `container_text`, or `subtitle_text`, then use those translated values for width calculation and display.


### Prepare / First-run

Zone status: reviewed.

Rules:

- Prepare/first-run language controls, back button, main instructional copy, severity labels, install-state labels, install action buttons, install-stage labels, and stable attention headings are UI-owned and use `ui_i18n_text_for_key(...)`.
- `prepare.*` and `tool_install.*` remain valid page/body keys. Do not reintroduce `tab.prepare`; the titlebar/tab entry has already been removed.
- Tool names such as `yt-dlp`, `Deno`, and `FFmpeg`, status icons such as `✓`, `×`, `○`, and `…`, percentages, system paths, `PATH`, file names, and raw OS/runtime error details stay fixed/raw and do not get locale keys from this zone.
- Environment issue titles/descriptions/recommendations use `prepare.req.*` keys when the text is stable app-owned repair guidance, such as writable folder checks or invalid output-folder advice. Raw paths, OS errors, probe details, stderr/stdout, and tool names stay fixed/raw.
- Environment issue details may still pass through `localize_message(...)` because a few details are stable key-backed app messages, but the broad prepare-check diagnostic layer must not be turned into full i18n.
- When Prepare text is used for measurement, resolve keys first into variables such as `required_text`, `install_all_text`, `skip_text`, or `status_text`, then use those translated values for width calculation and rendering.
### Legacy-assisted locale quality pass

The legacy `yt-dlp-gui.lang` files from the old project may be used as a human-translated glossary for compatible UI concepts.

Rules:
- Reuse legacy translations only when the current UI concept is equivalent.
- Do not import legacy keys directly; current Rust keys remain the only source of truth.
- Do not translate tool names, codec/container tokens, raw CLI output, paths, or diagnostic details.
- Languages not present in the legacy package must not be marked as legacy-backed.

Legacy-backed pass currently applies to: de-DE, el-GR, es-ES, fr-FR, it-IT, pl-PL, pt-BR, ru-RU.
### GitHub legacy language quality pass

The old project's GitHub `languages/` directory is treated as the preferred human-translated glossary source when a language exists there. The uploaded `languages.zip` is only a fallback snapshot.

Rules:
- Current Rust locale keys remain canonical; legacy `.lang` keys are never imported directly.
- Legacy wording may be reused only when the current UI concept is equivalent.
- New v2-only visible UI strings may be translated to match the legacy terminology style.
- Tool names, codec/container tokens, raw CLI output, file paths, and diagnostic details stay untranslated.
- All compiled locales now keep full key-order coverage; wording quality can still be improved by native speakers later.


## Translation quality pass

When raising locale quality, translate by UI meaning instead of literal English.
Use the polished source set (`en-US`, `zh-TW`, `zh-CN`, `ja-JP`, and strong legacy
community wording) to infer intent, then choose the shortest natural target label.
For example, `processing.choice.source` and `processing.subtitle.preserve` mean
“keep the original/source output”, not a generic data source. Prefer local UI
terms such as “Original”, “Oryginał”, “원본”, or “الأصلي” over literal “Source”.

Do not over-translate accepted technical loanwords. Words such as `Video`,
`Audio`, `Format`, `Codec`, `Cache`, or `Proxy` may be valid UI terms in some
languages. Change them only when the target-language UI normally uses a clearer
localized term, or when the previous value was raw English fallback rather than
a natural loanword.

During quality passes, long explanatory picker copy such as chapter compatibility and YouTube auto-translation notes should still be translated when it is stable visible UI copy. Keep YouTube/tool names raw, but translate the surrounding sentence naturally.
