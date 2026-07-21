use std::thread;
#[cfg(debug_assertions)]
use std::time::Duration;

const MUSIC_MIX_STRETCH_MIN_RATE: f64 = 0.965;
const MUSIC_MIX_STRETCH_MAX_RATE: f64 = 1.035;
const MUSIC_MIX_STRETCH_MIN_GAP: f64 = 0.0065;

#[cfg(debug_assertions)]
const MUSIC_MIX_STRETCH_MAX_SECONDS: f64 = 13.5;
#[cfg(not(debug_assertions))]
const MUSIC_MIX_STRETCH_MAX_SECONDS: f64 = 15.0;

#[cfg(debug_assertions)]
const MUSIC_MIX_STRETCH_MAX_FRAMES: usize = 648_000;
#[cfg(not(debug_assertions))]
const MUSIC_MIX_STRETCH_MAX_FRAMES: usize = 720_000;

#[cfg(debug_assertions)]
const MUSIC_MIX_STRETCH_CHUNK_FRAMES: usize = 256;
#[cfg(not(debug_assertions))]
const MUSIC_MIX_STRETCH_CHUNK_FRAMES: usize = 512;

#[cfg(debug_assertions)]
const MUSIC_MIX_STRETCH_CHUNK_SLEEP_EVERY: usize = 2;
#[cfg(not(debug_assertions))]
const MUSIC_MIX_STRETCH_CHUNK_SLEEP_EVERY: usize = 4;

#[derive(Clone, Debug)]
pub(crate) struct MusicMixStretchResult {
    pub samples: Vec<f32>,
    pub applied_rate: f64,
    pub preserve_pitch: bool,
    pub detail: String,
}

pub(crate) fn stretch_mix_transition_preserve_pitch(
    source: &[f32],
    sample_rate: u32,
    channels: usize,
    transition_source_rate: f64,
) -> Result<MusicMixStretchResult, String> {
    stretch_mix_transition_preserve_pitch_dynamic(
        source,
        sample_rate,
        channels,
        transition_source_rate,
        &[],
    )
}

pub(crate) fn stretch_mix_transition_preserve_pitch_to_frames(
    source: &[f32],
    sample_rate: u32,
    channels: usize,
    transition_source_rate: f64,
    target_output_frames: usize,
) -> Result<MusicMixStretchResult, String> {
    stretch_mix_transition_preserve_pitch_dynamic_to_frames(
        source,
        sample_rate,
        channels,
        transition_source_rate,
        &[],
        target_output_frames,
    )
}

pub(crate) fn stretch_mix_transition_preserve_pitch_high_quality(
    source: &[f32],
    sample_rate: u32,
    channels: usize,
    transition_source_rate: f64,
) -> Result<MusicMixStretchResult, String> {
    let channels = channels.max(1);
    if source.is_empty() || sample_rate == 0 || source.len() < channels {
        return Err("empty transition buffer".to_owned());
    }

    let source_frames = source.len() / channels;
    let source_seconds = source_frames as f64 / sample_rate.max(1) as f64;
    if source_seconds > MUSIC_MIX_STRETCH_MAX_SECONDS
        || source_frames > MUSIC_MIX_STRETCH_MAX_FRAMES
    {
        return Err(format!(
            "preserve-pitch high-quality skipped: preview budget ({source_seconds:.1}s)"
        ));
    }

    let active_rate =
        transition_source_rate.clamp(MUSIC_MIX_STRETCH_MIN_RATE, MUSIC_MIX_STRETCH_MAX_RATE);
    if (active_rate - 1.0).abs() < MUSIC_MIX_STRETCH_MIN_GAP {
        return Ok(MusicMixStretchResult {
            samples: source.to_vec(),
            applied_rate: 1.0,
            preserve_pitch: false,
            detail: "tempo neutral".to_owned(),
        });
    }

    // Preview-worker path: render the short B transition as one offline buffer
    // instead of feeding many tiny realtime chunks.  This is intentionally only
    // used for the mix preview segment, never for the main playback stream.
    let stretch_ratio = (1.0 / active_rate).clamp(0.25, 4.0);
    let params = timestretch::StretchParams::new(stretch_ratio)
        .with_sample_rate(sample_rate.max(1))
        .with_channels(channels.clamp(1, u32::MAX as usize) as u32)
        .with_quality_mode(timestretch::QualityMode::MaxQuality)
        .with_normalize(false);

    let mut stretched = timestretch::stretch(source, &params)
        .map_err(|error| format!("preserve-pitch high-quality stretch failed: {error}"))?;

    trim_denormals_and_non_finite(&mut stretched);
    if stretched.len() < channels {
        return Err("preserve-pitch high-quality stretch produced an empty buffer".to_owned());
    }

    Ok(MusicMixStretchResult {
        samples: stretched,
        applied_rate: active_rate,
        preserve_pitch: true,
        detail: format!(
            "preserve-hq B render {:+.1}% · MaxQuality",
            (active_rate - 1.0) * 100.0
        ),
    })
}

pub(crate) fn stretch_mix_transition_preserve_pitch_dynamic(
    source: &[f32],
    sample_rate: u32,
    channels: usize,
    initial_transition_source_rate: f64,
    rate_changes: &[(usize, f64)],
) -> Result<MusicMixStretchResult, String> {
    let channels = channels.max(1);
    if source.is_empty() || sample_rate == 0 || source.len() < channels {
        return Err("empty transition buffer".to_owned());
    }

    let source_frames = source.len() / channels;
    let source_seconds = source_frames as f64 / sample_rate.max(1) as f64;
    if source_seconds > MUSIC_MIX_STRETCH_MAX_SECONDS
        || source_frames > MUSIC_MIX_STRETCH_MAX_FRAMES
    {
        return Err(format!(
            "preserve-pitch stretch skipped: realtime budget ({source_seconds:.1}s)"
        ));
    }

    let initial_rate = initial_transition_source_rate
        .clamp(MUSIC_MIX_STRETCH_MIN_RATE, MUSIC_MIX_STRETCH_MAX_RATE);
    let active_rate = rate_changes
        .iter()
        .fold(initial_rate, |rate, (_, next_rate)| {
            if (next_rate - 1.0).abs() > (rate - 1.0).abs() {
                next_rate.clamp(MUSIC_MIX_STRETCH_MIN_RATE, MUSIC_MIX_STRETCH_MAX_RATE)
            } else {
                rate
            }
        });
    if (active_rate - 1.0).abs() < MUSIC_MIX_STRETCH_MIN_GAP {
        return Ok(MusicMixStretchResult {
            samples: source.to_vec(),
            applied_rate: 1.0,
            preserve_pitch: false,
            detail: "tempo neutral".to_owned(),
        });
    }

    let changes = normalized_rate_changes(rate_changes, source_frames);
    let output_frames = dynamic_output_frame_count(source_frames, initial_rate, &changes);
    let mut stretched = render_dynamic_keylock(
        source,
        sample_rate,
        channels,
        initial_rate,
        &changes,
        output_frames,
    )?;

    trim_denormals_and_non_finite(&mut stretched);
    if stretched.len() < channels {
        return Err("preserve-pitch stretch produced an empty buffer".to_owned());
    }

    let detail = if changes.is_empty() {
        format!(
            "preserve-stream B micro {:+.1}% · {}f chunks",
            (initial_rate - 1.0) * 100.0,
            MUSIC_MIX_STRETCH_CHUNK_FRAMES
        )
    } else {
        let last_rate = changes
            .last()
            .map(|(_, rate)| *rate)
            .unwrap_or(initial_rate);
        format!(
            "preserve-stream tempo-feather {:+.1}%→{:+.1}% · {} steps · {}f chunks",
            (initial_rate - 1.0) * 100.0,
            (last_rate - 1.0) * 100.0,
            changes.len(),
            MUSIC_MIX_STRETCH_CHUNK_FRAMES
        )
    };

    Ok(MusicMixStretchResult {
        samples: stretched,
        applied_rate: active_rate,
        preserve_pitch: true,
        detail,
    })
}

pub(crate) fn stretch_mix_transition_preserve_pitch_dynamic_to_frames(
    source: &[f32],
    sample_rate: u32,
    channels: usize,
    initial_transition_source_rate: f64,
    rate_changes: &[(usize, f64)],
    target_output_frames: usize,
) -> Result<MusicMixStretchResult, String> {
    let channels = channels.max(1);
    if source.is_empty() || sample_rate == 0 || source.len() < channels {
        return Err("empty transition buffer".to_owned());
    }

    let target_output_frames = target_output_frames.max(1);
    let target_output_samples = target_output_frames.saturating_mul(channels);
    let source_frames = source.len() / channels;
    let source_seconds = source_frames as f64 / sample_rate.max(1) as f64;
    if source_seconds > MUSIC_MIX_STRETCH_MAX_SECONDS
        || source_frames > MUSIC_MIX_STRETCH_MAX_FRAMES
    {
        return Err(format!(
            "preserve-pitch stretch skipped: realtime budget ({source_seconds:.1}s)"
        ));
    }

    let initial_rate = initial_transition_source_rate
        .clamp(MUSIC_MIX_STRETCH_MIN_RATE, MUSIC_MIX_STRETCH_MAX_RATE);
    let active_rate = rate_changes
        .iter()
        .fold(initial_rate, |rate, (_, next_rate)| {
            if (next_rate - 1.0).abs() > (rate - 1.0).abs() {
                next_rate.clamp(MUSIC_MIX_STRETCH_MIN_RATE, MUSIC_MIX_STRETCH_MAX_RATE)
            } else {
                rate
            }
        });
    if (active_rate - 1.0).abs() < MUSIC_MIX_STRETCH_MIN_GAP {
        let mut samples = fit_interleaved_to_frames(source, channels, target_output_frames);
        trim_denormals_and_non_finite(&mut samples);
        return Ok(MusicMixStretchResult {
            samples,
            applied_rate: 1.0,
            preserve_pitch: false,
            detail: "tempo neutral · reservoir-fit".to_owned(),
        });
    }

    let changes = normalized_rate_changes(rate_changes, source_frames);
    let mut stretched = render_dynamic_keylock(
        source,
        sample_rate,
        channels,
        initial_rate,
        &changes,
        target_output_frames,
    )?;
    stretched.truncate(target_output_samples);

    trim_denormals_and_non_finite(&mut stretched);
    if stretched.len() < channels {
        return Err("preserve-pitch reservoir produced an empty buffer".to_owned());
    }

    let detail = if changes.is_empty() {
        format!(
            "preserve-stream reservoir {:+.1}% · {}f chunks",
            (initial_rate - 1.0) * 100.0,
            MUSIC_MIX_STRETCH_CHUNK_FRAMES
        )
    } else {
        let last_rate = changes
            .last()
            .map(|(_, rate)| *rate)
            .unwrap_or(initial_rate);
        format!(
            "preserve-stream reservoir feather {:+.1}%→{:+.1}% · {} steps · {}f chunks",
            (initial_rate - 1.0) * 100.0,
            (last_rate - 1.0) * 100.0,
            changes.len(),
            MUSIC_MIX_STRETCH_CHUNK_FRAMES
        )
    };

    Ok(MusicMixStretchResult {
        samples: stretched,
        applied_rate: active_rate,
        preserve_pitch: true,
        detail,
    })
}

fn normalized_rate_changes(
    rate_changes: &[(usize, f64)],
    source_frames: usize,
) -> Vec<(usize, f64)> {
    let mut changes: Vec<(usize, f64)> = rate_changes
        .iter()
        .filter_map(|(frame, rate)| {
            (*frame > 0 && *frame < source_frames).then_some((
                *frame,
                rate.clamp(MUSIC_MIX_STRETCH_MIN_RATE, MUSIC_MIX_STRETCH_MAX_RATE),
            ))
        })
        .collect();
    changes.sort_by_key(|(frame, _)| *frame);
    changes.dedup_by(|next, previous| {
        if next.0 == previous.0 {
            previous.1 = next.1;
            true
        } else {
            false
        }
    });
    changes
}

fn dynamic_output_frame_count(
    source_frames: usize,
    initial_rate: f64,
    changes: &[(usize, f64)],
) -> usize {
    let mut output_frames = 0.0_f64;
    let mut previous_source_frame = 0_usize;
    let mut rate = initial_rate;
    for &(source_frame, next_rate) in changes {
        output_frames += source_frame.saturating_sub(previous_source_frame) as f64 / rate;
        previous_source_frame = source_frame;
        rate = next_rate;
    }
    output_frames += source_frames.saturating_sub(previous_source_frame) as f64 / rate;
    output_frames.round().max(1.0) as usize
}

fn render_dynamic_keylock(
    source: &[f32],
    sample_rate: u32,
    channels: usize,
    initial_rate: f64,
    changes: &[(usize, f64)],
    target_output_frames: usize,
) -> Result<Vec<f32>, String> {
    use timestretch::engine::{Engine, EngineConfig, EngineProfile};

    let max_block_frames = MUSIC_MIX_STRETCH_CHUNK_FRAMES.clamp(64, 8192);
    let source_capacity_frames = 32_768_usize.max(max_block_frames.saturating_mul(16));
    let handles = Engine::build(EngineConfig {
        sample_rate,
        channels,
        profile: EngineProfile::Keylock,
        initial_tempo_rate: initial_rate,
        max_block_frames,
        source_capacity_frames,
        pre_analysis: None,
    })
    .map_err(|error| format!("preserve-pitch engine build failed: {error}"))?;
    let (controller, mut processor, mut producer) =
        (handles.controller, handles.processor, handles.source);
    producer.set_track_position(0);

    let mut scheduled_output_frame = 0.0_f64;
    let mut previous_source_frame = 0_usize;
    let mut rate = initial_rate;
    for &(source_frame, next_rate) in changes {
        scheduled_output_frame += source_frame.saturating_sub(previous_source_frame) as f64 / rate;
        controller.set_tempo_rate_at(next_rate, scheduled_output_frame.round() as u64);
        previous_source_frame = source_frame;
        rate = next_rate;
    }

    let latency_frames = processor.pipeline_latency_frames();
    let render_frames = target_output_frames.saturating_add(latency_frames);
    let flush_frames = ((latency_frames as f64 * rate).ceil() as usize)
        .saturating_add(max_block_frames)
        .saturating_add(64);
    let flush = vec![0.0_f32; flush_frames.saturating_mul(channels)];
    let mut source_frame = 0_usize;
    let mut flush_frame = 0_usize;
    let mut rendered_frames = 0_usize;
    let mut chunks_processed = 0_usize;
    let mut block = vec![0.0_f32; max_block_frames.saturating_mul(channels)];
    let mut rendered = Vec::with_capacity(render_frames.saturating_mul(channels));

    while rendered_frames < render_frames {
        while producer.free_frames() > 0 {
            if source_frame < source.len() / channels {
                let offered_frames = producer
                    .free_frames()
                    .min(4_096)
                    .min(source.len() / channels - source_frame);
                let start = source_frame.saturating_mul(channels);
                let end = start.saturating_add(offered_frames.saturating_mul(channels));
                let accepted = producer.push(&source[start..end]);
                if accepted == 0 {
                    break;
                }
                source_frame = source_frame.saturating_add(accepted);
            } else if flush_frame < flush_frames {
                let offered_frames = producer
                    .free_frames()
                    .min(4_096)
                    .min(flush_frames - flush_frame);
                let start = flush_frame.saturating_mul(channels);
                let end = start.saturating_add(offered_frames.saturating_mul(channels));
                let accepted = producer.push(&flush[start..end]);
                if accepted == 0 {
                    break;
                }
                flush_frame = flush_frame.saturating_add(accepted);
            } else {
                break;
            }
        }

        let frames = max_block_frames.min(render_frames - rendered_frames);
        let samples = frames.saturating_mul(channels);
        processor.process(&mut block[..samples]);
        rendered.extend_from_slice(&block[..samples]);
        rendered_frames = rendered_frames.saturating_add(frames);
        chunks_processed = chunks_processed.saturating_add(1);

        if chunks_processed % MUSIC_MIX_STRETCH_CHUNK_SLEEP_EVERY == 0 {
            thread::yield_now();
            #[cfg(debug_assertions)]
            thread::sleep(Duration::from_millis(1));
        }
    }

    let latency_samples = latency_frames.saturating_mul(channels).min(rendered.len());
    rendered.drain(..latency_samples);
    rendered.truncate(target_output_frames.saturating_mul(channels));
    Ok(rendered)
}

fn fit_interleaved_to_frames(source: &[f32], channels: usize, target_frames: usize) -> Vec<f32> {
    let channels = channels.max(1);
    let source_frame_count = source.len() / channels;
    if source_frame_count == 0 || target_frames == 0 {
        return Vec::new();
    }

    let mut output = Vec::with_capacity(target_frames.saturating_mul(channels));
    for output_frame in 0..target_frames {
        let source_position = if target_frames <= 1 || source_frame_count <= 1 {
            0.0
        } else {
            output_frame as f64 * source_frame_count.saturating_sub(1) as f64
                / target_frames.saturating_sub(1) as f64
        };
        let left = source_position.floor().max(0.0) as usize;
        let right = (left + 1).min(source_frame_count.saturating_sub(1));
        let left = left.min(source_frame_count.saturating_sub(1));
        let frac = (source_position - left as f64).clamp(0.0, 1.0) as f32;
        for channel in 0..channels {
            let a = source
                .get(left.saturating_mul(channels) + channel)
                .copied()
                .unwrap_or(0.0);
            let b = source
                .get(right.saturating_mul(channels) + channel)
                .copied()
                .unwrap_or(a);
            output.push(a + (b - a) * frac);
        }
    }
    output
}

fn trim_denormals_and_non_finite(samples: &mut [f32]) {
    for sample in samples {
        if !sample.is_finite() || sample.abs() < 1.0e-20 {
            *sample = 0.0;
        } else {
            *sample = sample.clamp(-1.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_output_frame_count_integrates_each_rate_lane() {
        let frames = dynamic_output_frame_count(1_000, 1.0, &[(500, 2.0)]);
        assert_eq!(frames, 750);
    }

    #[test]
    fn dynamic_keylock_render_fills_the_requested_stereo_frame_count() {
        let sample_rate = 8_000_u32;
        let channels = 2_usize;
        let source_frames = sample_rate as usize;
        let source: Vec<f32> = (0..source_frames)
            .flat_map(|frame| {
                let phase = frame as f32 * 220.0 * std::f32::consts::TAU / sample_rate as f32;
                let sample = phase.sin() * 0.2;
                [sample, sample]
            })
            .collect();
        let target_frames = 8_120_usize;
        let rendered = render_dynamic_keylock(
            &source,
            sample_rate,
            channels,
            0.98,
            &[(4_000, 0.995)],
            target_frames,
        )
        .expect("dynamic keylock render should succeed");

        assert_eq!(rendered.len(), target_frames * channels);
        assert!(rendered.iter().all(|sample| sample.is_finite()));
        assert!(rendered.iter().any(|sample| sample.abs() > 1.0e-4));
    }
}
