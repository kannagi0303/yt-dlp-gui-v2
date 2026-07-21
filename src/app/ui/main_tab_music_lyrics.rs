use eframe::egui::{self, Color32, RichText, Ui};

use crate::app::state::{
    MusicLyricsDisplayLine, MusicPlayerAuraDisplay, MusicPlayerAuraTrackField,
};

use super::semantic_ui_metrics;

pub(super) fn render_music_lyrics_at(
    ui: &mut Ui,
    rect: egui::Rect,
    line: &MusicLyricsDisplayLine,
    aura_display: MusicPlayerAuraDisplay,
) {
    let visual = music_lyrics_visual(ui, aura_display);
    let fade = line.fade.clamp(0.0, 1.0);
    if fade < 1.0 {
        ui.ctx().request_repaint();
    }
    if let Some(previous) = line.previous.as_deref().filter(|_| fade < 1.0) {
        render_music_lyrics_text_at(ui, rect, previous, 1.0 - fade, visual);
    }
    render_music_lyrics_text_at(ui, rect, &line.current, fade.max(0.001), visual);
}

#[derive(Clone, Copy, Debug, Default)]
struct MusicLyricsVisual {
    outline_drive: f32,
    brightness: f32,
    sheen: f32,
    hue: f32,
}

impl MusicLyricsVisual {
    fn advance(&mut self, target: Self, dt: f32) {
        self.outline_drive = smooth(self.outline_drive, target.outline_drive, dt, 0.18);
        self.brightness = smooth(self.brightness, target.brightness, dt, 0.34);
        self.sheen = smooth(self.sheen, target.sheen, dt, 0.48);
        self.hue = smooth_unit(self.hue, target.hue, dt, 0.90);
    }
}

fn music_lyrics_visual(ui: &mut Ui, display: MusicPlayerAuraDisplay) -> MusicLyricsVisual {
    let target = music_lyrics_visual_target(display);
    let dt = ui.input(|input| input.stable_dt).clamp(0.0, 0.05);
    let memory_id = ui.make_persistent_id("music-lyrics-visual-envelope");
    let mut visual = ui.ctx().data_mut(|data| {
        data.get_temp::<MusicLyricsVisual>(memory_id)
            .unwrap_or(target)
    });
    visual.advance(target, dt);
    ui.ctx()
        .data_mut(|data| data.insert_temp(memory_id, visual));
    if display.animating {
        ui.ctx().request_repaint();
    }
    visual
}

fn music_lyrics_visual_target(display: MusicPlayerAuraDisplay) -> MusicLyricsVisual {
    let Some(primary) = display.primary else {
        return MusicLyricsVisual::default();
    };
    let secondary = display.secondary;
    let progress = display.mix_progress.clamp(0.0, 1.0);
    let weight_a = (progress * std::f32::consts::FRAC_PI_2).cos();
    let weight_b = (progress * std::f32::consts::FRAC_PI_2).sin();
    let blend = |a: f32, b: f32| {
        if secondary.is_some() {
            (a * weight_a + b * weight_b) / (weight_a + weight_b).max(0.001)
        } else {
            a
        }
    };
    let secondary = secondary.unwrap_or(primary);
    let bass = blend(non_vocal_bass(primary), non_vocal_bass(secondary));
    let air = blend(non_vocal_air(primary), non_vocal_air(secondary));
    let energy = blend(primary.energy, secondary.energy).clamp(0.0, 1.0);
    let hue = smooth_unit_phase(
        primary.chroma_hue,
        secondary.chroma_hue,
        weight_b / (weight_a + weight_b).max(0.001),
    );

    MusicLyricsVisual {
        outline_drive: (bass * 0.72 + energy * 0.28).clamp(0.0, 1.0),
        brightness: (energy * 0.68 + air * 0.32).clamp(0.0, 1.0),
        sheen: (air * 0.74
            + blend(
                primary.section_color_strength,
                secondary.section_color_strength,
            ) * 0.26)
            .clamp(0.0, 1.0),
        hue,
    }
}

fn non_vocal_bass(field: MusicPlayerAuraTrackField) -> f32 {
    (field.spectrum_peaks[0].max(field.spectrum_peaks[1]) * 0.72
        + field.spectrum_bands[0].max(field.spectrum_bands[1]) * 0.28)
        .clamp(0.0, 1.0)
}

fn non_vocal_air(field: MusicPlayerAuraTrackField) -> f32 {
    (field.spectrum_peaks[6].max(field.spectrum_peaks[7]) * 0.66
        + field.spectrum_bands[6].max(field.spectrum_bands[7]) * 0.34)
        .clamp(0.0, 1.0)
}

fn render_music_lyrics_text_at(
    ui: &mut Ui,
    rect: egui::Rect,
    line: &str,
    alpha: f32,
    visual: MusicLyricsVisual,
) {
    let font_size = semantic_ui_metrics::main_music_lyrics_font_size_from_body(ui);
    let text = RichText::new(line).size(font_size);
    let galley = egui::WidgetText::from(text).into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        rect.width(),
        egui::TextStyle::Body,
    );
    let pos = egui::pos2(
        rect.center().x - galley.size().x * 0.5,
        rect.center().y - galley.size().y * 0.5,
    );
    let outline_alpha = alpha * (0.22 + visual.outline_drive * 0.13);
    let outline_color = Color32::from_rgba_unmultiplied(
        0,
        0,
        0,
        (outline_alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
    );
    let radius = (0.72 / ui.ctx().pixels_per_point()).clamp(0.38, 0.72);
    for offset in [
        egui::vec2(-radius, 0.0),
        egui::vec2(radius, 0.0),
        egui::vec2(0.0, -radius),
        egui::vec2(0.0, radius),
    ] {
        ui.painter()
            .galley(pos + offset, galley.clone(), outline_color);
    }

    let base = ui.visuals().text_color();
    let brighter = mix_color(base, Color32::WHITE, 0.02 + visual.brightness * 0.025);
    let aura_tint = saturated_hue_color(visual.hue);
    let text_color = mix_color(brighter, aura_tint, visual.sheen * 0.045);
    ui.painter()
        .galley(pos, galley, color_with_alpha(text_color, alpha));
}

fn smooth(current: f32, target: f32, dt: f32, time_constant: f32) -> f32 {
    let alpha = if dt > 0.0 {
        1.0 - (-dt / time_constant.max(0.001)).exp()
    } else {
        0.0
    };
    current + (target - current) * alpha
}

fn smooth_unit(current: f32, target: f32, dt: f32, time_constant: f32) -> f32 {
    smooth_unit_phase(
        current,
        target,
        if dt > 0.0 {
            1.0 - (-dt / time_constant.max(0.001)).exp()
        } else {
            0.0
        },
    )
}

fn smooth_unit_phase(current: f32, target: f32, alpha: f32) -> f32 {
    let delta = (target - current + 0.5).rem_euclid(1.0) - 0.5;
    (current + delta * alpha.clamp(0.0, 1.0)).rem_euclid(1.0)
}

fn saturated_hue_color(hue: f32) -> Color32 {
    let h = hue.rem_euclid(1.0) * 6.0;
    let x = 1.0 - (h.rem_euclid(2.0) - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    Color32::from_rgb(
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

fn mix_color(from: Color32, to: Color32, amount: f32) -> Color32 {
    let amount = amount.clamp(0.0, 1.0);
    let channel =
        |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * amount).round() as u8;
    Color32::from_rgba_unmultiplied(
        channel(from.r(), to.r()),
        channel(from.g(), to.g()),
        channel(from.b(), to.b()),
        channel(from.a(), to.a()),
    )
}

fn color_with_alpha(color: Color32, alpha: f32) -> Color32 {
    let alpha = (f32::from(color.a()) * alpha.clamp(0.0, 1.0)).round() as u8;
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lyric_reactivity_ignores_voice_band_energy() {
        let mut field = MusicPlayerAuraTrackField::default();
        field.spectrum_bands[3] = 1.0;
        field.spectrum_bands[4] = 1.0;
        field.spectrum_peaks[3] = 1.0;
        field.spectrum_peaks[4] = 1.0;
        let visual = music_lyrics_visual_target(MusicPlayerAuraDisplay {
            animating: true,
            primary_item_id: Some(1),
            primary: Some(field),
            ..Default::default()
        });

        assert_eq!(visual.outline_drive, 0.0);
        assert_eq!(visual.brightness, 0.0);
        assert_eq!(visual.sheen, 0.0);
    }

    #[test]
    fn lyric_envelope_eases_instead_of_teleporting() {
        let mut visual = MusicLyricsVisual::default();
        visual.advance(
            MusicLyricsVisual {
                outline_drive: 1.0,
                brightness: 1.0,
                sheen: 1.0,
                hue: 0.8,
            },
            1.0 / 60.0,
        );

        assert!((0.0..0.2).contains(&visual.outline_drive));
        assert!((0.0..0.1).contains(&visual.brightness));
        assert!((0.0..0.1).contains(&visual.sheen));
    }
}
