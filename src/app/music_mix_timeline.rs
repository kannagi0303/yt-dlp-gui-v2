//! Stage Mix frame units and timeline conversions.
//!
//! Codex handoff rule:
//! - music analysis and UI may describe cue positions in seconds;
//! - decode APIs may require seconds because the container API is time-based;
//! - once a transition enters the playback/render path, use the explicit frame
//!   types in this module and do not introduce `Instant`, milliseconds, or raw
//!   interleaved-sample counts as audio timeline state.

use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct MusicMixOutputFrame(u64);

impl MusicMixOutputFrame {
    pub(crate) const ZERO: Self = Self(0);

    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    pub(crate) const fn get(self) -> u64 {
        self.0
    }

    pub(crate) const fn saturating_add(self, count: MusicMixFrameCount) -> Self {
        Self(self.0.saturating_add(count.0))
    }

    pub(crate) const fn saturating_sub(self, earlier: Self) -> MusicMixFrameCount {
        MusicMixFrameCount(self.0.saturating_sub(earlier.0))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct MusicMixSourceFrame(u64);

impl MusicMixSourceFrame {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    pub(crate) const fn get(self) -> u64 {
        self.0
    }

    pub(crate) const fn saturating_add(self, count: MusicMixFrameCount) -> Self {
        Self(self.0.saturating_add(count.0))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct MusicMixFrameCount(u64);

impl MusicMixFrameCount {
    pub(crate) const ZERO: Self = Self(0);

    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    pub(crate) const fn get(self) -> u64 {
        self.0
    }

    pub(crate) const fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub(crate) const fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    pub(crate) const fn min(self, other: Self) -> Self {
        if self.0 <= other.0 { self } else { other }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MusicMixFrameClock {
    sample_rate: u32,
}

impl MusicMixFrameClock {
    pub(crate) const fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate: if sample_rate == 0 { 1 } else { sample_rate },
        }
    }

    pub(crate) const fn sample_rate(self) -> u32 {
        self.sample_rate
    }

    pub(crate) fn frame_count_from_seconds(self, seconds: f64) -> MusicMixFrameCount {
        if !seconds.is_finite() || seconds <= 0.0 {
            return MusicMixFrameCount::ZERO;
        }
        MusicMixFrameCount::new(
            (seconds * f64::from(self.sample_rate))
                .round()
                .clamp(0.0, u64::MAX as f64) as u64,
        )
    }

    pub(crate) fn frame_count_from_duration(self, duration: Duration) -> MusicMixFrameCount {
        self.frame_count_from_seconds(duration.as_secs_f64())
    }

    pub(crate) fn source_frame_from_seconds(self, seconds: f64) -> MusicMixSourceFrame {
        MusicMixSourceFrame::new(self.frame_count_from_seconds(seconds).get())
    }

    pub(crate) fn seconds_from_frame_count(self, frames: MusicMixFrameCount) -> f64 {
        frames.get() as f64 / f64::from(self.sample_rate)
    }

    pub(crate) fn seconds_from_source_frame(self, frame: MusicMixSourceFrame) -> f64 {
        frame.get() as f64 / f64::from(self.sample_rate)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MusicMixSourcePosition {
    pub(crate) frame: MusicMixSourceFrame,
    pub(crate) sample_rate: u32,
}

impl MusicMixSourcePosition {
    pub(crate) fn seconds(self) -> f64 {
        MusicMixFrameClock::new(self.sample_rate).seconds_from_source_frame(self.frame)
    }
}

pub(crate) fn source_position_after_rendered_frames(
    source_start: MusicMixSourceFrame,
    source_sample_rate: u32,
    rendered_elapsed: MusicMixFrameCount,
    rendered_sample_rate: u32,
    transition_rendered_frames: MusicMixFrameCount,
    transition_source_frames: MusicMixFrameCount,
) -> MusicMixSourcePosition {
    let source_clock = MusicMixFrameClock::new(source_sample_rate);
    let rendered_clock = MusicMixFrameClock::new(rendered_sample_rate);
    let transition_elapsed = rendered_elapsed.min(transition_rendered_frames);
    let natural_elapsed = MusicMixFrameCount::new(
        rendered_elapsed
            .get()
            .saturating_sub(transition_elapsed.get()),
    );

    // The rendered transition can contain multiple tempo regions and natural
    // anchors. Keep its total source consumption as an integer frame contract;
    // reconstructing it from one aggregate floating-point rate drifts at the
    // preview -> promoted-main boundary.
    let transition_source_elapsed = if transition_rendered_frames.is_zero() {
        MusicMixFrameCount::ZERO
    } else {
        let numerator = u128::from(transition_source_frames.get())
            .saturating_mul(u128::from(transition_elapsed.get()));
        let rounded = numerator.saturating_add(u128::from(transition_rendered_frames.get()) / 2)
            / u128::from(transition_rendered_frames.get());
        MusicMixFrameCount::new(rounded.min(u128::from(u64::MAX)) as u64)
    };
    let natural_source_elapsed = source_clock
        .frame_count_from_seconds(rendered_clock.seconds_from_frame_count(natural_elapsed));
    let source_delta = transition_source_elapsed.saturating_add(natural_source_elapsed);

    MusicMixSourcePosition {
        frame: source_start.saturating_add(source_delta),
        sample_rate: source_clock.sample_rate(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_round_trip_uses_frames_as_the_only_execution_unit() {
        let clock = MusicMixFrameClock::new(48_000);
        let frames = clock.frame_count_from_seconds(1.25);

        assert_eq!(frames, MusicMixFrameCount::new(60_000));
        assert!((clock.seconds_from_frame_count(frames) - 1.25).abs() < f64::EPSILON);
    }

    #[test]
    fn source_position_applies_stretch_only_inside_transition_frames() {
        let position = source_position_after_rendered_frames(
            MusicMixSourceFrame::new(48_000),
            48_000,
            MusicMixFrameCount::new(96_000),
            48_000,
            MusicMixFrameCount::new(48_000),
            MusicMixFrameCount::new(48_960),
        );

        assert_eq!(position.frame, MusicMixSourceFrame::new(144_960));
        assert!((position.seconds() - 3.02).abs() < 0.000_001);
    }

    #[test]
    fn source_position_rounds_partial_transition_in_source_frames() {
        let position = source_position_after_rendered_frames(
            MusicMixSourceFrame::new(1_000),
            44_100,
            MusicMixFrameCount::new(24_000),
            48_000,
            MusicMixFrameCount::new(48_000),
            MusicMixFrameCount::new(44_541),
        );

        assert_eq!(position.frame, MusicMixSourceFrame::new(23_271));
    }
}
