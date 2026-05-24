use eframe::egui::{IconData, Theme};
use image::ImageFormat;

const DARK_APP_ICON_BYTES: &[u8] = include_bytes!("../../assets/logo.ico");
const LIGHT_APP_ICON_BYTES: &[u8] = include_bytes!("../../assets/logo_light.ico");

pub(crate) fn app_window_icon(theme: Theme) -> IconData {
    decode_app_icon(theme_icon_bytes(theme))
}

pub(crate) fn app_window_icon_crossfade(from: Theme, to: Theme, progress: f32) -> IconData {
    if from == to {
        return app_window_icon(to);
    }

    let from_icon = app_window_icon(from);
    let to_icon = app_window_icon(to);
    if from_icon.width != to_icon.width
        || from_icon.height != to_icon.height
        || from_icon.rgba.len() != to_icon.rgba.len()
    {
        return to_icon;
    }

    let progress = progress.clamp(0.0, 1.0);
    let rgba = from_icon
        .rgba
        .iter()
        .zip(to_icon.rgba.iter())
        .map(|(from, to)| mix_byte(*from, *to, progress))
        .collect();

    IconData {
        rgba,
        width: to_icon.width,
        height: to_icon.height,
    }
}

fn theme_icon_bytes(theme: Theme) -> &'static [u8] {
    match theme {
        Theme::Dark => DARK_APP_ICON_BYTES,
        Theme::Light => LIGHT_APP_ICON_BYTES,
    }
}

fn decode_app_icon(bytes: &[u8]) -> IconData {
    let image =
        image::load_from_memory_with_format(bytes, ImageFormat::Ico).expect("failed to decode app icon");
    let rgba = image.into_rgba8();
    let (width, height) = rgba.dimensions();
    IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    }
}

fn mix_byte(from: u8, to: u8, amount: f32) -> u8 {
    let from = from as f32;
    let to = to as f32;
    (from + (to - from) * amount)
        .round()
        .clamp(0.0, 255.0) as u8
}
