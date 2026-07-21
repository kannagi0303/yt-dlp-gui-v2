//! Frame-addressed PCM reservoir for Stage Mix handoff.
//!
//! This is not a generic cache. It records PCM that has already entered the
//! playback timeline, keyed by source frames, so Prepared Mix can request a
//! precise A range instead of guessing from the current queue head.

use std::collections::VecDeque;
use std::time::Duration;

use crate::app::music_mix_timeline::{MusicMixFrameCount, MusicMixSourceFrame};

#[derive(Clone, Debug)]
pub(crate) struct MusicPcmReservoirSnapshot {
    pub(crate) samples: Vec<f32>,
    pub(crate) start_frame: MusicMixSourceFrame,
    pub(crate) frame_count: MusicMixFrameCount,
    pub(crate) sample_rate: u32,
    pub(crate) channels: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MusicPcmReservoirCoverage {
    pub(crate) start_frame: MusicMixSourceFrame,
    pub(crate) end_frame: MusicMixSourceFrame,
    pub(crate) sample_rate: u32,
    pub(crate) channels: usize,
}

#[derive(Debug)]
pub(crate) struct MusicPcmReservoir {
    max_duration: Duration,
    sample_rate: u32,
    channels: usize,
    start_frame: MusicMixSourceFrame,
    samples: VecDeque<f32>,
}

impl MusicPcmReservoir {
    pub(crate) fn new(max_duration: Duration) -> Self {
        Self {
            max_duration,
            sample_rate: 44_100,
            channels: 2,
            start_frame: MusicMixSourceFrame::new(0),
            samples: VecDeque::new(),
        }
    }

    pub(crate) fn clear_from(
        &mut self,
        sample_rate: u32,
        channels: usize,
        start_frame: MusicMixSourceFrame,
    ) {
        self.sample_rate = sample_rate.max(1);
        self.channels = channels.max(1);
        self.start_frame = start_frame;
        self.samples.clear();
    }

    pub(crate) fn append_interleaved(
        &mut self,
        start_frame: MusicMixSourceFrame,
        samples: &[f32],
        channels: usize,
        sample_rate: u32,
    ) {
        let channels = channels.max(1);
        let sample_rate = sample_rate.max(1);
        let input_frames = samples.len() / channels;
        if input_frames == 0 {
            return;
        }
        let aligned_samples = input_frames.saturating_mul(channels);
        if self.samples.is_empty() || self.channels != channels || self.sample_rate != sample_rate {
            self.clear_from(sample_rate, channels, start_frame);
        }

        let current_end = self.end_frame_value();
        let append_start = start_frame.get();
        let append_end = append_start.saturating_add(input_frames as u64);
        if append_end <= current_end {
            return;
        }
        if append_start > current_end {
            self.clear_from(sample_rate, channels, start_frame);
        }

        let skip_frames = current_end
            .saturating_sub(append_start)
            .min(input_frames as u64) as usize;
        let skip_samples = skip_frames.saturating_mul(channels).min(aligned_samples);
        self.samples
            .extend(samples[skip_samples..aligned_samples].iter().copied());
        self.trim_to_max_duration();
    }

    pub(crate) fn snapshot_range(
        &self,
        start_frame: MusicMixSourceFrame,
        frame_count: MusicMixFrameCount,
    ) -> Option<MusicPcmReservoirSnapshot> {
        if frame_count.is_zero() || self.samples.is_empty() {
            return None;
        }
        let start = start_frame.get();
        let frames = frame_count.get();
        let end = start.checked_add(frames)?;
        let coverage_start = self.start_frame.get();
        let coverage_end = self.end_frame_value();
        if start < coverage_start || end > coverage_end {
            return None;
        }

        let offset_frames = start.saturating_sub(coverage_start) as usize;
        let offset_samples = offset_frames.saturating_mul(self.channels);
        let requested_samples = (frames as usize).saturating_mul(self.channels);
        if offset_samples.saturating_add(requested_samples) > self.samples.len() {
            return None;
        }

        Some(MusicPcmReservoirSnapshot {
            samples: self
                .samples
                .iter()
                .skip(offset_samples)
                .take(requested_samples)
                .copied()
                .collect(),
            start_frame,
            frame_count,
            sample_rate: self.sample_rate,
            channels: self.channels,
        })
    }

    pub(crate) fn coverage(&self) -> Option<MusicPcmReservoirCoverage> {
        if self.samples.is_empty() {
            return None;
        }
        Some(MusicPcmReservoirCoverage {
            start_frame: self.start_frame,
            end_frame: MusicMixSourceFrame::new(self.end_frame_value()),
            sample_rate: self.sample_rate,
            channels: self.channels,
        })
    }

    fn frame_count(&self) -> usize {
        self.samples.len() / self.channels.max(1)
    }

    fn end_frame_value(&self) -> u64 {
        self.start_frame
            .get()
            .saturating_add(self.frame_count() as u64)
    }

    fn max_frames(&self) -> usize {
        (self.max_duration.as_secs_f64() * f64::from(self.sample_rate.max(1)))
            .round()
            .clamp(1.0, usize::MAX as f64) as usize
    }

    fn trim_to_max_duration(&mut self) {
        let max_frames = self.max_frames();
        let frame_count = self.frame_count();
        if frame_count <= max_frames {
            return;
        }
        let drop_frames = frame_count.saturating_sub(max_frames);
        let drop_samples = drop_frames
            .saturating_mul(self.channels)
            .min(self.samples.len());
        self.samples.drain(..drop_samples);
        self.start_frame = self
            .start_frame
            .saturating_add(MusicMixFrameCount::new(drop_frames as u64));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reservoir_slices_exact_frame_range() {
        let mut reservoir = MusicPcmReservoir::new(Duration::from_secs(20));
        let samples: Vec<f32> = (0..20).map(|value| value as f32).collect();

        reservoir.append_interleaved(MusicMixSourceFrame::new(100), &samples, 2, 1_000);
        let snapshot = reservoir
            .snapshot_range(MusicMixSourceFrame::new(103), MusicMixFrameCount::new(3))
            .expect("covered range");

        assert_eq!(snapshot.start_frame, MusicMixSourceFrame::new(103));
        assert_eq!(snapshot.frame_count, MusicMixFrameCount::new(3));
        assert_eq!(snapshot.samples, vec![6.0, 7.0, 8.0, 9.0, 10.0, 11.0]);
    }

    #[test]
    fn reservoir_deduplicates_overlapping_appends() {
        let mut reservoir = MusicPcmReservoir::new(Duration::from_secs(20));
        reservoir.append_interleaved(
            MusicMixSourceFrame::new(10),
            &[1.0, 2.0, 3.0, 4.0],
            2,
            1_000,
        );
        reservoir.append_interleaved(
            MusicMixSourceFrame::new(11),
            &[3.0, 4.0, 5.0, 6.0],
            2,
            1_000,
        );

        let snapshot = reservoir
            .snapshot_range(MusicMixSourceFrame::new(10), MusicMixFrameCount::new(3))
            .expect("merged range");

        assert_eq!(snapshot.samples, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn reservoir_resets_on_gap() {
        let mut reservoir = MusicPcmReservoir::new(Duration::from_secs(20));
        reservoir.append_interleaved(MusicMixSourceFrame::new(10), &[1.0, 2.0], 2, 1_000);
        reservoir.append_interleaved(MusicMixSourceFrame::new(20), &[3.0, 4.0], 2, 1_000);

        assert!(
            reservoir
                .snapshot_range(MusicMixSourceFrame::new(10), MusicMixFrameCount::new(1))
                .is_none()
        );
        assert_eq!(
            reservoir
                .snapshot_range(MusicMixSourceFrame::new(20), MusicMixFrameCount::new(1))
                .expect("new range")
                .samples,
            vec![3.0, 4.0]
        );
    }

    #[test]
    fn reservoir_keeps_recent_frames_with_duration_cap() {
        let mut reservoir = MusicPcmReservoir::new(Duration::from_millis(3));
        let samples: Vec<f32> = (0..10).map(|value| value as f32).collect();

        reservoir.append_interleaved(MusicMixSourceFrame::new(0), &samples, 2, 1_000);
        let coverage = reservoir.coverage().expect("coverage");

        assert_eq!(coverage.start_frame, MusicMixSourceFrame::new(2));
        assert_eq!(coverage.end_frame, MusicMixSourceFrame::new(5));
    }
}
