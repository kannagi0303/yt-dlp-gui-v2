use crate::app::state::{MusicPlayerAuraDisplay, MusicPlayerAuraPulse, MusicPlayerAuraTrackField};

const MAX_TRACK_STATES: usize = 4;

pub(super) struct MusicPlayerAuraDynamics {
    tracks: Vec<AuraTrackDynamics>,
}

impl MusicPlayerAuraDynamics {
    pub(super) fn new() -> Self {
        Self { tracks: Vec::new() }
    }

    pub(super) fn advance(
        &mut self,
        mut display: MusicPlayerAuraDisplay,
        frame_seconds: f32,
    ) -> MusicPlayerAuraDisplay {
        let dt = if display.animating {
            frame_seconds.clamp(0.0, 0.05)
        } else {
            0.0
        };

        if let (Some(item_id), Some(target)) = (display.primary_item_id, display.primary) {
            display.primary = Some(self.advance_track(item_id, target, dt));
        }
        if let (Some(item_id), Some(target)) = (display.secondary_item_id, display.secondary) {
            display.secondary = Some(self.advance_track(item_id, target, dt));
        }
        display
    }

    fn advance_track(
        &mut self,
        item_id: u64,
        target: MusicPlayerAuraTrackField,
        dt: f32,
    ) -> MusicPlayerAuraTrackField {
        let index = self
            .tracks
            .iter()
            .position(|track| track.item_id == item_id)
            .unwrap_or_else(|| {
                if self.tracks.len() >= MAX_TRACK_STATES {
                    self.tracks.remove(0);
                }
                self.tracks.push(AuraTrackDynamics::new(item_id, target));
                self.tracks.len() - 1
            });
        self.tracks[index].advance(target, dt)
    }
}

struct AuraTrackDynamics {
    item_id: u64,
    palette_hue: f32,
    section_hue_offset: f32,
    field: MusicPlayerAuraTrackField,
    lobes: [AuraMotionLobe; 4],
}

impl AuraTrackDynamics {
    fn new(item_id: u64, mut field: MusicPlayerAuraTrackField) -> Self {
        let seed = unit_seed(item_id);
        let palette_hue = track_palette_hue(item_id);
        let section_hue_offset =
            section_palette_offset(field.section_color_unit, field.section_color_strength);
        field.chroma_hue =
            (resolved_track_hue(palette_hue, field.chroma_hue, field.chroma_coherence)
                + section_hue_offset)
                .rem_euclid(1.0);
        Self {
            item_id,
            palette_hue,
            section_hue_offset,
            field,
            lobes: [
                AuraMotionLobe::new((seed + 0.07).fract(), 0.014, 0.0, false),
                AuraMotionLobe::new((seed + 0.53).fract(), -0.011, 0.46, false),
                AuraMotionLobe::new((seed + 0.29).fract(), -0.021, 0.21, true),
                AuraMotionLobe::new((seed + 0.81).fract(), 0.024, 0.72, true),
            ],
        }
    }

    fn advance(&mut self, target: MusicPlayerAuraTrackField, dt: f32) -> MusicPlayerAuraTrackField {
        let material_alpha = smoothing_alpha(dt, 0.24);
        let structure_alpha = smoothing_alpha(dt, 0.38);
        let color_alpha = smoothing_alpha(dt, 0.85);
        let section_alpha = smoothing_alpha(dt, 1.35);

        self.field.bar_phase =
            smooth_unit_phase(self.field.bar_phase, target.bar_phase, material_alpha);
        self.field.beat_phase =
            smooth_unit_phase(self.field.beat_phase, target.beat_phase, material_alpha);
        self.field.downbeat_strength = lerp(
            self.field.downbeat_strength,
            target.downbeat_strength,
            structure_alpha,
        );
        self.field.energy = lerp(self.field.energy, target.energy, material_alpha);
        self.field.energy_momentum = lerp(
            self.field.energy_momentum,
            target.energy_momentum,
            structure_alpha,
        );
        self.field.boundary = lerp(self.field.boundary, target.boundary, structure_alpha);
        self.field.novelty = lerp(self.field.novelty, target.novelty, structure_alpha);
        self.field.recurrence = lerp(self.field.recurrence, target.recurrence, structure_alpha);
        self.field.chorusness = lerp(self.field.chorusness, target.chorusness, structure_alpha);
        self.field.section_color_unit = smooth_unit_phase(
            self.field.section_color_unit,
            target.section_color_unit,
            section_alpha,
        );
        self.field.section_color_strength = lerp(
            self.field.section_color_strength,
            target.section_color_strength,
            section_alpha,
        );
        let target_bass = (target.spectrum_bands[0] + target.spectrum_bands[1]) * 0.5;
        let target_air = (target.spectrum_bands[6] + target.spectrum_bands[7]) * 0.5;
        let track_hue =
            resolved_track_hue(self.palette_hue, target.chroma_hue, target.chroma_coherence);
        let target_section_offset =
            section_palette_offset(target.section_color_unit, target.section_color_strength);
        self.section_hue_offset = lerp(
            self.section_hue_offset,
            target_section_offset,
            section_alpha,
        );
        let spectral_hue = (track_hue
            + self.section_hue_offset
            + (target_air - target_bass).clamp(-1.0, 1.0) * 0.045)
            .rem_euclid(1.0);
        self.field.chroma_hue = smooth_unit_phase(self.field.chroma_hue, spectral_hue, color_alpha);
        self.field.chroma_coherence = lerp(
            self.field.chroma_coherence,
            target.chroma_coherence,
            color_alpha,
        );
        for index in 0..8 {
            self.field.spectrum_bands[index] = lerp(
                self.field.spectrum_bands[index],
                target.spectrum_bands[index],
                material_alpha,
            );
            self.field.spectrum_peaks[index] = lerp(
                self.field.spectrum_peaks[index],
                target.spectrum_peaks[index],
                structure_alpha,
            );
        }

        let bass_drive = (self.field.spectrum_peaks[0].max(self.field.spectrum_peaks[1]) * 0.72
            + self.field.spectrum_bands[0].max(self.field.spectrum_bands[1]) * 0.28)
            .clamp(0.0, 1.0);
        let air_drive = (self.field.spectrum_peaks[6].max(self.field.spectrum_peaks[7]) * 0.68
            + self.field.spectrum_bands[6].max(self.field.spectrum_bands[7]) * 0.32)
            .clamp(0.0, 1.0);
        let momentum = self.field.energy_momentum.clamp(-1.0, 1.0);

        for (index, lobe) in self.lobes.iter_mut().enumerate() {
            let drive = if lobe.air { air_drive } else { bass_drive };
            let weight = if index % 2 == 0 { 0.82 } else { 0.64 };
            lobe.advance(dt, drive * weight, momentum);
            self.field.pulses[index] = lobe.as_pulse();
        }
        self.field
    }
}

struct AuraMotionLobe {
    origin: f32,
    angular_velocity: f32,
    direction: f32,
    phase: f32,
    strength: f32,
    air: bool,
}

impl AuraMotionLobe {
    fn new(origin: f32, angular_velocity: f32, phase: f32, air: bool) -> Self {
        Self {
            origin,
            angular_velocity,
            direction: angular_velocity.signum(),
            phase,
            strength: 0.0,
            air,
        }
    }

    fn advance(&mut self, dt: f32, drive: f32, momentum: f32) {
        if dt <= 0.0 {
            return;
        }
        let target_speed = self.direction
            * (if self.air { 0.018 } else { 0.012 }
                + drive * if self.air { 0.014 } else { 0.012 }
                + momentum * 0.002);
        self.angular_velocity = lerp(
            self.angular_velocity,
            target_speed,
            smoothing_alpha(dt, 1.25),
        )
        .clamp(-0.035, 0.035);
        self.origin = (self.origin + self.angular_velocity * dt).rem_euclid(1.0);
        let phase_speed = if self.air { 0.22 } else { 0.14 } + drive * 0.06;
        self.phase = (self.phase + phase_speed * dt).rem_euclid(1.0);
        let response = if drive > self.strength { 0.22 } else { 0.68 };
        self.strength = lerp(self.strength, drive, smoothing_alpha(dt, response));
    }

    fn as_pulse(&self) -> MusicPlayerAuraPulse {
        MusicPlayerAuraPulse {
            origin: self.origin,
            age: self.phase,
            strength: self.strength,
            air: f32::from(self.air),
        }
    }
}

fn smoothing_alpha(dt: f32, time_constant: f32) -> f32 {
    if dt <= 0.0 {
        0.0
    } else {
        1.0 - (-dt / time_constant.max(0.001)).exp()
    }
}

fn smooth_unit_phase(current: f32, target: f32, alpha: f32) -> f32 {
    let delta = (target - current + 0.5).rem_euclid(1.0) - 0.5;
    (current + delta * alpha).rem_euclid(1.0)
}

fn lerp(current: f32, target: f32, alpha: f32) -> f32 {
    current + (target - current) * alpha.clamp(0.0, 1.0)
}

fn mixed_item_seed(item_id: u64) -> u64 {
    let mut value = item_id ^ 0x9E37_79B9_7F4A_7C15;
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 31;
    value
}

fn unit_seed(item_id: u64) -> f32 {
    (mixed_item_seed(item_id) as u32) as f32 / u32::MAX as f32
}

fn track_palette_hue(item_id: u64) -> f32 {
    const CURATED_HUES: [f32; 9] = [
        0.535, // cyan
        0.600, // sapphire
        0.690, // violet
        0.805, // amethyst
        0.925, // rose
        0.055, // vermilion
        0.115, // amber
        0.285, // lime
        0.430, // emerald
    ];
    CURATED_HUES[mixed_item_seed(item_id) as usize % CURATED_HUES.len()]
}

fn resolved_track_hue(palette_hue: f32, harmonic_hue: f32, coherence: f32) -> f32 {
    let harmonic_weight = 0.26 + coherence.clamp(0.0, 1.0) * 0.14;
    smooth_unit_phase(palette_hue, harmonic_hue, harmonic_weight)
}

fn section_palette_offset(section_unit: f32, strength: f32) -> f32 {
    // Hue-only movement preserves saturation and avoids the gray/low-chroma
    // detour that made earlier Aura variants look foggy.
    (section_unit.clamp(0.0, 1.0) - 0.5) * 0.30 * strength.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn display_with_spectrum(item_id: u64, bass: f32, air: f32) -> MusicPlayerAuraDisplay {
        let mut field = MusicPlayerAuraTrackField::default();
        field.spectrum_bands[0] = bass;
        field.spectrum_peaks[0] = bass;
        field.spectrum_bands[7] = air;
        field.spectrum_peaks[7] = air;
        MusicPlayerAuraDisplay {
            animating: true,
            primary_item_id: Some(item_id),
            primary: Some(field),
            ..Default::default()
        }
    }

    fn display(item_id: u64, bass: f32) -> MusicPlayerAuraDisplay {
        display_with_spectrum(item_id, bass, 0.0)
    }

    #[test]
    fn dynamics_advances_from_previous_frame_without_teleporting() {
        let mut dynamics = MusicPlayerAuraDynamics::new();
        let first = dynamics.advance(display(7, 1.0), 1.0 / 60.0);
        let second = dynamics.advance(display(7, 1.0), 1.0 / 60.0);
        let first_lobe = first.primary.unwrap().pulses[0];
        let second_lobe = second.primary.unwrap().pulses[0];

        let distance = (second_lobe.origin - first_lobe.origin).abs();
        assert!(distance > 0.0);
        assert!(distance < 0.01);
        assert!(second_lobe.strength > first_lobe.strength);
    }

    #[test]
    fn dynamics_reuses_secondary_state_after_mix_handoff() {
        let mut dynamics = MusicPlayerAuraDynamics::new();
        let mut mixed = display(10, 0.8);
        mixed.secondary_item_id = Some(20);
        mixed.secondary = display(20, 0.9).primary;
        let mixed = dynamics.advance(mixed, 1.0 / 60.0);
        let secondary_before = mixed.secondary.unwrap().pulses[0];

        let handed_off = dynamics.advance(display(20, 0.9), 1.0 / 60.0);
        let primary_after = handed_off.primary.unwrap().pulses[0];
        let distance = (primary_after.origin - secondary_before.origin).abs();

        assert!(distance > 0.0);
        assert!(distance < 0.01);
        assert!(primary_after.strength >= secondary_before.strength);
    }

    #[test]
    fn unit_phase_smoothing_crosses_wrap_by_the_short_path() {
        let smoothed = smooth_unit_phase(0.98, 0.02, 0.5);
        assert!(smoothed < 0.01 || smoothed > 0.99);
    }

    #[test]
    fn dynamics_caps_lobe_speed_even_at_full_drive() {
        let mut dynamics = MusicPlayerAuraDynamics::new();
        for _ in 0..600 {
            dynamics.advance(display_with_spectrum(7, 1.0, 1.0), 1.0 / 60.0);
        }

        assert!(
            dynamics.tracks[0]
                .lobes
                .iter()
                .all(|lobe| lobe.angular_velocity.abs() <= 0.035)
        );
    }

    #[test]
    fn spectral_color_moves_gradually_instead_of_jumping() {
        let mut dynamics = MusicPlayerAuraDynamics::new();
        let mut bass_hue = 0.0;
        for _ in 0..90 {
            bass_hue = dynamics
                .advance(display_with_spectrum(7, 1.0, 0.0), 1.0 / 60.0)
                .primary
                .unwrap()
                .chroma_hue;
        }
        let first_air_hue = dynamics
            .advance(display_with_spectrum(7, 0.0, 1.0), 1.0 / 60.0)
            .primary
            .unwrap()
            .chroma_hue;
        let first_step = ((first_air_hue - bass_hue + 0.5).rem_euclid(1.0) - 0.5).abs();
        assert!(first_step < 0.005);

        let mut settled_air_hue = first_air_hue;
        for _ in 0..120 {
            settled_air_hue = dynamics
                .advance(display_with_spectrum(7, 0.0, 1.0), 1.0 / 60.0)
                .primary
                .unwrap()
                .chroma_hue;
        }
        let total_shift = ((settled_air_hue - bass_hue + 0.5).rem_euclid(1.0) - 0.5).abs();
        assert!(total_shift > 0.03);
    }

    #[test]
    fn section_palette_changes_are_large_but_eased() {
        let mut dynamics = MusicPlayerAuraDynamics::new();
        let mut first_section = display(7, 0.0);
        let first_field = first_section.primary.as_mut().unwrap();
        first_field.section_color_unit = 0.0;
        first_field.section_color_strength = 1.0;
        let mut before = 0.0;
        for _ in 0..180 {
            before = dynamics
                .advance(first_section, 1.0 / 60.0)
                .primary
                .unwrap()
                .chroma_hue;
        }

        let mut next_section = first_section;
        next_section.primary.as_mut().unwrap().section_color_unit = 1.0;
        let first_frame = dynamics
            .advance(next_section, 1.0 / 60.0)
            .primary
            .unwrap()
            .chroma_hue;
        let first_step = ((first_frame - before + 0.5).rem_euclid(1.0) - 0.5).abs();
        assert!(first_step < 0.005);

        let mut settled = first_frame;
        for _ in 0..240 {
            settled = dynamics
                .advance(next_section, 1.0 / 60.0)
                .primary
                .unwrap()
                .chroma_hue;
        }
        let total_shift = ((settled - before + 0.5).rem_euclid(1.0) - 0.5).abs();
        assert!(total_shift > 0.12);
    }

    #[test]
    fn curated_track_palette_is_stable_and_varied_across_item_identity() {
        let hues = (1..=32).map(track_palette_hue).collect::<Vec<_>>();
        assert_eq!(track_palette_hue(7), track_palette_hue(7));
        let unique = hues.iter().fold(Vec::<f32>::new(), |mut unique, hue| {
            if !unique
                .iter()
                .any(|value| (*value - *hue).abs() < f32::EPSILON)
            {
                unique.push(*hue);
            }
            unique
        });
        assert!(unique.len() >= 7);
        assert!(hues.iter().all(|hue| (0.0..1.0).contains(hue)));
    }
}
