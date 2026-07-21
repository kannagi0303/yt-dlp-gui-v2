use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, Ui, mutex::Mutex};
use eframe::egui_glow;
use eframe::glow::{self, HasContext as _};

use crate::app::state::{MusicPlayerAuraDisplay, MusicPlayerAuraTrackField};
use crate::app::widgets::url_input::accent_blue_for_ui;

use super::main_tab_music_aura_dynamics::MusicPlayerAuraDynamics;

const AURA_TARGET_FRAME_INTERVAL: Duration = Duration::from_micros(33_333);

const AURA_VERTEX_SHADER: &str = r#"#version 140
out vec2 v_uv;

const vec2 POSITIONS[3] = vec2[](
    vec2(-1.0, -1.0),
    vec2( 3.0, -1.0),
    vec2(-1.0,  3.0)
);

void main() {
    vec2 position = POSITIONS[gl_VertexID];
    v_uv = position * 0.5 + 0.5;
    gl_Position = vec4(position, 0.0, 1.0);
}
"#;

const AURA_FRAGMENT_SHADER: &str = r#"#version 140
in vec2 v_uv;
out vec4 out_color;

uniform vec3 u_size_radius;
uniform vec3 u_accent;
uniform vec4 u_environment;
uniform vec4 u_primary_a;
uniform vec4 u_primary_b;
uniform vec4 u_primary_c;
uniform vec4 u_primary_spectrum_low;
uniform vec4 u_primary_spectrum_high;
uniform vec4 u_primary_pulse_1;
uniform vec4 u_primary_pulse_2;
uniform vec4 u_primary_pulse_3;
uniform vec4 u_primary_pulse_4;
uniform vec4 u_secondary_a;
uniform vec4 u_secondary_b;
uniform vec4 u_secondary_c;
uniform vec4 u_secondary_spectrum_low;
uniform vec4 u_secondary_spectrum_high;
uniform vec4 u_secondary_pulse_1;
uniform vec4 u_secondary_pulse_2;
uniform vec4 u_secondary_pulse_3;
uniform vec4 u_secondary_pulse_4;

const float PI = 3.14159265358979323846;
const float TAU = 6.28318530717958647692;

float rounded_rect_sdf(vec2 point, vec2 half_size, float radius) {
    vec2 inner = half_size - vec2(radius);
    vec2 q = abs(point) - inner;
    return length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

vec3 hsv_to_rgb(vec3 hsv) {
    vec3 p = abs(fract(hsv.xxx + vec3(0.0, 2.0 / 3.0, 1.0 / 3.0)) * 6.0 - 3.0);
    return hsv.z * mix(vec3(1.0), clamp(p - 1.0, 0.0, 1.0), hsv.y);
}

vec2 boundary_wavefront(
    vec4 pulse,
    vec2 point,
    vec2 half_size,
    float x_bias
) {
    float strength = clamp(pulse.z, 0.0, 1.0);
    if (strength <= 0.001) {
        return vec2(0.0);
    }

    float age = clamp(pulse.y, 0.0, 1.0);
    float air = clamp(pulse.w, 0.0, 1.0);
    float source_angle = TAU * pulse.x;
    vec2 source_axis = vec2(cos(source_angle), sin(source_angle));
    vec2 source_extent = max(half_size - vec2(4.0), vec2(1.0));
    float source_scale = min(
        source_extent.x / max(abs(source_axis.x), 0.001),
        source_extent.y / max(abs(source_axis.y), 0.001)
    );
    vec2 source = source_axis * source_scale;
    float slow_lateral_drift = sin(source_angle * 1.73) * 0.035;
    source.x = clamp(
        source.x + half_size.x * (x_bias + slow_lateral_drift),
        -source_extent.x,
        source_extent.x
    );
    vec2 tangent = vec2(-source_axis.y, source_axis.x);
    vec2 inward = -source_axis;
    vec2 delta = point - source;
    float along_distance = abs(dot(delta, tangent));
    float inward_distance = max(dot(delta, inward), 0.0);
    float radius = mix(5.0, mix(94.0, 56.0, air), age);
    float width = mix(13.5, 6.5, air) * (0.82 + strength * 0.28);
    float distortion =
        1.0
        + sin(
            along_distance * 0.041 + source_angle * 1.7 + age * TAU
        ) * 0.10
        + sin(
            inward_distance * 0.057 - source_angle + age * TAU * 0.61
        ) * 0.045;
    float inward_scale = mix(0.72, 1.12, air);
    float metric = sqrt(
        pow(along_distance * distortion, 2.0)
        + pow(inward_distance * inward_scale, 2.0)
    );
    // A persistent lobe cycles only while its visibility is zero at the wrap.
    // Its identity, position and energy continue from the previous frame.
    float cycle_envelope = pow(sin(PI * age), mix(1.15, 1.55, air));
    float crest = exp(-pow((metric - radius) / width, 2.0))
        * strength * cycle_envelope;
    float wake =
        (1.0 - smoothstep(radius - width * 1.7, radius + width * 0.30, metric))
        * exp(-inward_distance / mix(35.0, 19.0, air))
        * strength
        * cycle_envelope;
    return vec2(wake, crest);
}

vec2 track_wave_field(
    vec4 pulse_1,
    vec4 pulse_2,
    vec4 pulse_3,
    vec4 pulse_4,
    vec2 point,
    vec2 half_size
) {
    vec2 field =
        boundary_wavefront(pulse_1, point, half_size, -0.18)
        + boundary_wavefront(pulse_2, point, half_size, 0.14)
        + boundary_wavefront(pulse_3, point, half_size, -0.08)
        + boundary_wavefront(pulse_4, point, half_size, 0.22);
    return clamp(field, vec2(0.0), vec2(1.35));
}

float ambient_fluid_field(
    vec4 a,
    vec4 b,
    vec4 c,
    vec4 low,
    vec4 high,
    vec2 normalized_point,
    float direction
) {
    float energy = clamp(a.w, 0.0, 1.0);
    float momentum = clamp(b.x, -1.0, 1.0);
    float low_current = clamp((low.x + low.y) * 0.5, 0.0, 1.0);
    float air_current = clamp((high.z + high.w) * 0.5, 0.0, 1.0);
    float phase = TAU * (a.x + c.y * 0.09);
    vec2 center_1 = vec2(
        -0.72 * direction + sin(phase) * 0.08,
        -0.58 + cos(phase * 0.73) * 0.10
    );
    vec2 center_2 = vec2(
        0.64 * direction + cos(phase * 0.61) * 0.11,
        0.64 + sin(phase * 0.87) * 0.08
    );
    vec2 delta_1 = (normalized_point - center_1) / vec2(0.78, 0.56);
    vec2 delta_2 = (normalized_point - center_2) / vec2(0.66, 0.62);
    float blob_1 = exp(-dot(delta_1, delta_1) * 1.65);
    float blob_2 = exp(-dot(delta_2, delta_2) * 1.85);
    float fold = 0.5 + 0.5 * sin(TAU * (
        normalized_point.x * 0.34
        - normalized_point.y * 0.27
        + direction * sin(phase) * 0.07
        + momentum * 0.04
    ));
    return clamp(
        (blob_1 + blob_2) * (0.055 + energy * 0.095)
        + blob_1 * low_current * 0.075
        + blob_2 * air_current * 0.060
        + fold * clamp(b.z, 0.0, 1.0) * 0.035,
        0.0,
        0.30
    );
}

vec3 analytic_track_color(vec4 a, vec4 b, vec4 c, float u, float dark_mode) {
    float hue = fract(
        c.y
        + u * 0.34
        + sin(TAU * (2.0 * u + a.x)) * (0.030 + b.z * 0.025)
        + b.x * 0.015
    );
    float saturation = clamp(0.76 + c.z * 0.16 + c.x * 0.05, 0.76, 0.98);
    float value = mix(0.88, 1.0, dark_mode);
    vec3 harmonic = hsv_to_rgb(vec3(hue, saturation, value));
    return mix(u_accent, harmonic, 0.74 + c.z * 0.18);
}

void main() {
    vec2 size = max(u_size_radius.xy, vec2(1.0));
    vec2 half_size = size * 0.5;
    float radius = clamp(u_size_radius.z, 1.0, min(half_size.x, half_size.y));
    vec2 point = (v_uv - 0.5) * size;
    float signed_distance = rounded_rect_sdf(point, half_size, radius);
    float inside = 1.0 - smoothstep(-0.35, 0.85, signed_distance);
    if (inside <= 0.001) {
        out_color = vec4(0.0);
        return;
    }

    float edge_distance = max(-signed_distance, 0.0);
    vec2 normalized_point = point / max(half_size, vec2(1.0));
    // Color drifts continuously across the material. It must not reuse a
    // nearest-edge perimeter coordinate because that coordinate has diagonal
    // medial-axis seams when extended through the panel interior.
    float color_coordinate = fract(
        0.50 + normalized_point.x * 0.18 + normalized_point.y * 0.13
    );
    float mix_progress = clamp(u_environment.x, 0.0, 1.0);
    float weight_a = cos(mix_progress * PI * 0.5);
    float weight_b = sin(mix_progress * PI * 0.5);
    vec2 wave_a = track_wave_field(
        u_primary_pulse_1,
        u_primary_pulse_2,
        u_primary_pulse_3,
        u_primary_pulse_4,
        point,
        half_size
    );
    vec2 wave_b = track_wave_field(
        u_secondary_pulse_1,
        u_secondary_pulse_2,
        u_secondary_pulse_3,
        u_secondary_pulse_4,
        point,
        half_size
    );
    vec2 wave_amplitude_a = weight_a * wave_a;
    vec2 wave_amplitude_b = weight_b * wave_b;
    float wave_wake = sqrt(
        wave_amplitude_a.x * wave_amplitude_a.x
        + wave_amplitude_b.x * wave_amplitude_b.x
    );
    float wave_crest = sqrt(
        wave_amplitude_a.y * wave_amplitude_a.y
        + wave_amplitude_b.y * wave_amplitude_b.y
    );
    float ambient_a = ambient_fluid_field(
        u_primary_a,
        u_primary_b,
        u_primary_c,
        u_primary_spectrum_low,
        u_primary_spectrum_high,
        normalized_point,
        1.0
    );
    float ambient_b = ambient_fluid_field(
        u_secondary_a,
        u_secondary_b,
        u_secondary_c,
        u_secondary_spectrum_low,
        u_secondary_spectrum_high,
        normalized_point,
        -1.0
    );
    float ambient_field = sqrt(
        pow(weight_a * ambient_a, 2.0) + pow(weight_b * ambient_b, 2.0)
    );

    vec3 color_a = analytic_track_color(
        u_primary_a, u_primary_b, u_primary_c, color_coordinate, u_environment.z
    );
    vec3 color_b = analytic_track_color(
        u_secondary_a, u_secondary_b, u_secondary_c, color_coordinate, u_environment.z
    );
    float color_weight_a = weight_a * (0.12 + ambient_a + wave_a.x + wave_a.y);
    float color_weight_b = weight_b * (0.12 + ambient_b + wave_b.x + wave_b.y);
    vec3 color = (
        color_a * color_weight_a + color_b * color_weight_b
    ) / max(color_weight_a + color_weight_b, 0.001);

    float top_light = clamp(0.5 - normalized_point.y * 0.32, 0.0, 1.0);
    float neutral_outer_cut = exp(-pow((edge_distance - 0.60) / 0.50, 2.0));
    float neutral_inner_cut = exp(-pow((edge_distance - 1.85) / 0.78, 2.0));
    float neutral_alpha = inside * (
        neutral_outer_cut * 0.040
        + neutral_inner_cut * (0.052 + top_light * 0.042)
    );

    // The neutral frame owns the first pixels. Beyond that moat, wave age and
    // anisotropic distance decide the shape; there is no fixed rectangular
    // inner cutoff.
    float material_gate = smoothstep(1.65, 3.05, edge_distance);
    float motion_presence = mix(0.66, 1.0, u_environment.y);
    float fluid_alpha = inside * material_gate * motion_presence * (
        ambient_field * 0.72 + wave_wake * 0.34
    );
    float crest_alpha = inside * material_gate * motion_presence * wave_crest * 0.46;

    float alpha = clamp(
        neutral_alpha + fluid_alpha + crest_alpha,
        0.0,
        0.86
    );
    vec3 neutral_color = vec3(mix(0.52, 0.76, top_light));
    vec3 crest_color = mix(color, vec3(1.0), 0.34);
    vec3 premultiplied = (
        neutral_color * neutral_alpha
        + color * fluid_alpha
        + crest_color * crest_alpha
    );

    out_color = vec4(premultiplied, alpha);
}
"#;

#[derive(Clone)]
pub(super) struct MusicPlayerAuraRenderer {
    renderer: Arc<Mutex<MusicPlayerAuraGlowRenderer>>,
}

impl MusicPlayerAuraRenderer {
    pub(super) fn new(gl: &glow::Context) -> Result<Self, String> {
        Ok(Self {
            renderer: Arc::new(Mutex::new(MusicPlayerAuraGlowRenderer::new(gl)?)),
        })
    }

    pub(super) fn destroy(&self, gl: &glow::Context) {
        self.renderer.lock().destroy(gl);
    }
}

pub(super) fn render_music_player_aura_at(
    ui: &Ui,
    rect: egui::Rect,
    renderer: Option<&MusicPlayerAuraRenderer>,
    display: MusicPlayerAuraDisplay,
    corner_radius: f32,
) {
    let (Some(renderer), Some(_)) = (renderer, display.primary) else {
        return;
    };
    if rect.width() <= 2.0 || rect.height() <= 2.0 {
        return;
    }

    let accent = color_to_linearish_rgb(accent_blue_for_ui(ui));
    let dark_mode = ui.visuals().dark_mode;
    let renderer = renderer.clone();
    let callback = egui_glow::CallbackFn::new(move |info, painter| {
        renderer.renderer.lock().paint(
            painter.gl().as_ref(),
            info,
            corner_radius,
            accent,
            dark_mode,
            display,
        );
    });
    ui.painter().add(egui::PaintCallback {
        rect,
        callback: Arc::new(callback),
    });

    if display.animating {
        ui.ctx().request_repaint_after(AURA_TARGET_FRAME_INTERVAL);
    }
}

fn color_to_linearish_rgb(color: Color32) -> [f32; 3] {
    [
        f32::from(color.r()) / 255.0,
        f32::from(color.g()) / 255.0,
        f32::from(color.b()) / 255.0,
    ]
}

struct MusicPlayerAuraGlowRenderer {
    program: Option<glow::Program>,
    vertex_array: Option<glow::VertexArray>,
    uniforms: MusicPlayerAuraUniforms,
    dynamics: MusicPlayerAuraDynamics,
    last_dynamics_update: Instant,
    last_track_identity: (Option<u64>, Option<u64>),
    cached_display: Option<MusicPlayerAuraDisplay>,
}

impl MusicPlayerAuraGlowRenderer {
    fn new(gl: &glow::Context) -> Result<Self, String> {
        let program = create_aura_program(gl)?;
        let vertex_array = match unsafe { gl.create_vertex_array() } {
            Ok(vertex_array) => vertex_array,
            Err(error) => {
                unsafe {
                    gl.delete_program(program);
                }
                return Err(format!("cannot create aura vertex array: {error}"));
            }
        };
        let uniforms = MusicPlayerAuraUniforms::new(gl, program);
        Ok(Self {
            program: Some(program),
            vertex_array: Some(vertex_array),
            uniforms,
            dynamics: MusicPlayerAuraDynamics::new(),
            last_dynamics_update: Instant::now(),
            last_track_identity: (None, None),
            cached_display: None,
        })
    }

    fn paint(
        &mut self,
        gl: &glow::Context,
        info: egui::PaintCallbackInfo,
        corner_radius: f32,
        accent: [f32; 3],
        dark_mode: bool,
        display: MusicPlayerAuraDisplay,
    ) {
        let (Some(program), Some(vertex_array)) = (self.program, self.vertex_array) else {
            return;
        };
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_dynamics_update);
        let identity = (display.primary_item_id, display.secondary_item_id);
        if aura_dynamics_should_advance(
            elapsed,
            self.cached_display.is_none(),
            identity != self.last_track_identity,
        ) {
            self.cached_display = Some(self.dynamics.advance(display, elapsed.as_secs_f32()));
            self.last_dynamics_update = now;
            self.last_track_identity = identity;
        }
        let display = self.cached_display.unwrap_or(display);
        let Some(primary) = display.primary else {
            return;
        };
        let secondary = display.secondary.unwrap_or(primary);
        let mix_progress = if display.secondary.is_some() {
            display.mix_progress.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let environment = [
            mix_progress,
            f32::from(display.animating),
            f32::from(dark_mode),
            0.0,
        ];
        let viewport = info.viewport_in_pixels();
        let size_radius = [
            viewport.width_px.max(1) as f32,
            viewport.height_px.max(1) as f32,
            (corner_radius * info.pixels_per_point).max(1.0),
        ];
        let primary = PackedAuraTrackField::from(primary);
        let secondary = PackedAuraTrackField::from(secondary);

        unsafe {
            gl.use_program(Some(program));
            gl.bind_vertex_array(Some(vertex_array));
            gl.uniform_3_f32(
                self.uniforms.size_radius.as_ref(),
                size_radius[0],
                size_radius[1],
                size_radius[2],
            );
            gl.uniform_3_f32(
                self.uniforms.accent.as_ref(),
                accent[0],
                accent[1],
                accent[2],
            );
            uniform_vec4(gl, self.uniforms.environment.as_ref(), environment);
            uniform_vec4(gl, self.uniforms.primary_a.as_ref(), primary.a);
            uniform_vec4(gl, self.uniforms.primary_b.as_ref(), primary.b);
            uniform_vec4(gl, self.uniforms.primary_c.as_ref(), primary.c);
            uniform_vec4(
                gl,
                self.uniforms.primary_spectrum_low.as_ref(),
                primary.spectrum_low,
            );
            uniform_vec4(
                gl,
                self.uniforms.primary_spectrum_high.as_ref(),
                primary.spectrum_high,
            );
            uniform_vec4(
                gl,
                self.uniforms.primary_pulses[0].as_ref(),
                primary.pulses[0],
            );
            uniform_vec4(
                gl,
                self.uniforms.primary_pulses[1].as_ref(),
                primary.pulses[1],
            );
            uniform_vec4(
                gl,
                self.uniforms.primary_pulses[2].as_ref(),
                primary.pulses[2],
            );
            uniform_vec4(
                gl,
                self.uniforms.primary_pulses[3].as_ref(),
                primary.pulses[3],
            );
            uniform_vec4(gl, self.uniforms.secondary_a.as_ref(), secondary.a);
            uniform_vec4(gl, self.uniforms.secondary_b.as_ref(), secondary.b);
            uniform_vec4(gl, self.uniforms.secondary_c.as_ref(), secondary.c);
            uniform_vec4(
                gl,
                self.uniforms.secondary_spectrum_low.as_ref(),
                secondary.spectrum_low,
            );
            uniform_vec4(
                gl,
                self.uniforms.secondary_spectrum_high.as_ref(),
                secondary.spectrum_high,
            );
            uniform_vec4(
                gl,
                self.uniforms.secondary_pulses[0].as_ref(),
                secondary.pulses[0],
            );
            uniform_vec4(
                gl,
                self.uniforms.secondary_pulses[1].as_ref(),
                secondary.pulses[1],
            );
            uniform_vec4(
                gl,
                self.uniforms.secondary_pulses[2].as_ref(),
                secondary.pulses[2],
            );
            uniform_vec4(
                gl,
                self.uniforms.secondary_pulses[3].as_ref(),
                secondary.pulses[3],
            );
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }

    fn destroy(&mut self, gl: &glow::Context) {
        unsafe {
            if let Some(program) = self.program.take() {
                gl.delete_program(program);
            }
            if let Some(vertex_array) = self.vertex_array.take() {
                gl.delete_vertex_array(vertex_array);
            }
        }
    }
}

fn uniform_vec4(gl: &glow::Context, location: Option<&glow::UniformLocation>, values: [f32; 4]) {
    unsafe {
        gl.uniform_4_f32(location, values[0], values[1], values[2], values[3]);
    }
}

fn aura_dynamics_should_advance(
    elapsed: Duration,
    cache_empty: bool,
    track_identity_changed: bool,
) -> bool {
    cache_empty || track_identity_changed || elapsed >= AURA_TARGET_FRAME_INTERVAL
}

struct PackedAuraTrackField {
    a: [f32; 4],
    b: [f32; 4],
    c: [f32; 4],
    spectrum_low: [f32; 4],
    spectrum_high: [f32; 4],
    pulses: [[f32; 4]; 4],
}

impl From<MusicPlayerAuraTrackField> for PackedAuraTrackField {
    fn from(field: MusicPlayerAuraTrackField) -> Self {
        Self {
            a: [
                field.bar_phase,
                field.beat_phase,
                field.downbeat_strength,
                field.energy,
            ],
            b: [
                field.energy_momentum,
                field.boundary,
                field.novelty,
                field.recurrence,
            ],
            c: [
                field.chorusness,
                field.chroma_hue,
                field.chroma_coherence,
                0.0,
            ],
            spectrum_low: [
                field.spectrum_bands[0],
                field.spectrum_bands[1],
                field.spectrum_bands[2],
                field.spectrum_bands[3],
            ],
            spectrum_high: [
                field.spectrum_bands[4],
                field.spectrum_bands[5],
                field.spectrum_bands[6],
                field.spectrum_bands[7],
            ],
            pulses: field
                .pulses
                .map(|pulse| [pulse.origin, pulse.age, pulse.strength, pulse.air]),
        }
    }
}

struct MusicPlayerAuraUniforms {
    size_radius: Option<glow::UniformLocation>,
    accent: Option<glow::UniformLocation>,
    environment: Option<glow::UniformLocation>,
    primary_a: Option<glow::UniformLocation>,
    primary_b: Option<glow::UniformLocation>,
    primary_c: Option<glow::UniformLocation>,
    primary_spectrum_low: Option<glow::UniformLocation>,
    primary_spectrum_high: Option<glow::UniformLocation>,
    primary_pulses: [Option<glow::UniformLocation>; 4],
    secondary_a: Option<glow::UniformLocation>,
    secondary_b: Option<glow::UniformLocation>,
    secondary_c: Option<glow::UniformLocation>,
    secondary_spectrum_low: Option<glow::UniformLocation>,
    secondary_spectrum_high: Option<glow::UniformLocation>,
    secondary_pulses: [Option<glow::UniformLocation>; 4],
}

impl MusicPlayerAuraUniforms {
    fn new(gl: &glow::Context, program: glow::Program) -> Self {
        unsafe {
            Self {
                size_radius: gl.get_uniform_location(program, "u_size_radius"),
                accent: gl.get_uniform_location(program, "u_accent"),
                environment: gl.get_uniform_location(program, "u_environment"),
                primary_a: gl.get_uniform_location(program, "u_primary_a"),
                primary_b: gl.get_uniform_location(program, "u_primary_b"),
                primary_c: gl.get_uniform_location(program, "u_primary_c"),
                primary_spectrum_low: gl.get_uniform_location(program, "u_primary_spectrum_low"),
                primary_spectrum_high: gl.get_uniform_location(program, "u_primary_spectrum_high"),
                primary_pulses: [
                    gl.get_uniform_location(program, "u_primary_pulse_1"),
                    gl.get_uniform_location(program, "u_primary_pulse_2"),
                    gl.get_uniform_location(program, "u_primary_pulse_3"),
                    gl.get_uniform_location(program, "u_primary_pulse_4"),
                ],
                secondary_a: gl.get_uniform_location(program, "u_secondary_a"),
                secondary_b: gl.get_uniform_location(program, "u_secondary_b"),
                secondary_c: gl.get_uniform_location(program, "u_secondary_c"),
                secondary_spectrum_low: gl
                    .get_uniform_location(program, "u_secondary_spectrum_low"),
                secondary_spectrum_high: gl
                    .get_uniform_location(program, "u_secondary_spectrum_high"),
                secondary_pulses: [
                    gl.get_uniform_location(program, "u_secondary_pulse_1"),
                    gl.get_uniform_location(program, "u_secondary_pulse_2"),
                    gl.get_uniform_location(program, "u_secondary_pulse_3"),
                    gl.get_uniform_location(program, "u_secondary_pulse_4"),
                ],
            }
        }
    }
}

fn create_aura_program(gl: &glow::Context) -> Result<glow::Program, String> {
    unsafe {
        let program = gl
            .create_program()
            .map_err(|error| format!("cannot create aura shader program: {error}"))?;
        let vertex = match compile_aura_shader(gl, glow::VERTEX_SHADER, AURA_VERTEX_SHADER) {
            Ok(shader) => shader,
            Err(error) => {
                gl.delete_program(program);
                return Err(error);
            }
        };
        let fragment = match compile_aura_shader(gl, glow::FRAGMENT_SHADER, AURA_FRAGMENT_SHADER) {
            Ok(shader) => shader,
            Err(error) => {
                gl.delete_shader(vertex);
                gl.delete_program(program);
                return Err(error);
            }
        };

        gl.attach_shader(program, vertex);
        gl.attach_shader(program, fragment);
        gl.link_program(program);
        let linked = gl.get_program_link_status(program);
        let link_log = gl.get_program_info_log(program);
        gl.detach_shader(program, vertex);
        gl.detach_shader(program, fragment);
        gl.delete_shader(vertex);
        gl.delete_shader(fragment);

        if linked {
            Ok(program)
        } else {
            gl.delete_program(program);
            Err(format!("cannot link aura shader program: {link_log}"))
        }
    }
}

fn compile_aura_shader(
    gl: &glow::Context,
    shader_type: u32,
    source: &str,
) -> Result<glow::Shader, String> {
    unsafe {
        let shader = gl
            .create_shader(shader_type)
            .map_err(|error| format!("cannot create aura shader: {error}"))?;
        gl.shader_source(shader, source);
        gl.compile_shader(shader);
        if gl.get_shader_compile_status(shader) {
            Ok(shader)
        } else {
            let compile_log = gl.get_shader_info_log(shader);
            gl.delete_shader(shader);
            Err(format!("cannot compile aura shader: {compile_log}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_track_field_keeps_analysis_channels_in_explicit_uniform_groups() {
        let packed = PackedAuraTrackField::from(MusicPlayerAuraTrackField {
            bar_phase: 0.1,
            beat_phase: 0.2,
            downbeat_strength: 0.3,
            energy: 0.4,
            energy_momentum: -0.5,
            boundary: 0.6,
            novelty: 0.7,
            recurrence: 0.8,
            chorusness: 0.9,
            chroma_hue: 0.25,
            chroma_coherence: 0.75,
            section_color_unit: 0.5,
            section_color_strength: 0.0,
            spectrum_bands: [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
            spectrum_peaks: [0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1],
            pulses: [
                crate::app::state::MusicPlayerAuraPulse {
                    origin: 0.1,
                    age: 0.2,
                    strength: 0.3,
                    air: 0.0,
                },
                crate::app::state::MusicPlayerAuraPulse {
                    origin: 0.4,
                    age: 0.5,
                    strength: 0.6,
                    air: 0.0,
                },
                crate::app::state::MusicPlayerAuraPulse {
                    origin: 0.7,
                    age: 0.8,
                    strength: 0.9,
                    air: 1.0,
                },
                crate::app::state::MusicPlayerAuraPulse::default(),
            ],
        });

        assert_eq!(packed.a, [0.1, 0.2, 0.3, 0.4]);
        assert_eq!(packed.b, [-0.5, 0.6, 0.7, 0.8]);
        assert_eq!(packed.c, [0.9, 0.25, 0.75, 0.0]);
        assert_eq!(packed.spectrum_low, [0.1, 0.2, 0.3, 0.4]);
        assert_eq!(packed.spectrum_high, [0.5, 0.6, 0.7, 0.8]);
        assert_eq!(packed.pulses[0], [0.1, 0.2, 0.3, 0.0]);
        assert_eq!(packed.pulses[2], [0.7, 0.8, 0.9, 1.0]);
    }

    #[test]
    fn shader_keeps_distributed_boundary_waves_searchable() {
        assert!(AURA_FRAGMENT_SHADER.contains("rounded_rect_sdf"));
        assert!(AURA_FRAGMENT_SHADER.contains("boundary_wavefront"));
        assert!(AURA_FRAGMENT_SHADER.contains("track_wave_field"));
        assert!(AURA_FRAGMENT_SHADER.contains("source_axis"));
        assert!(AURA_FRAGMENT_SHADER.contains("x_bias"));
        assert!(AURA_FRAGMENT_SHADER.contains("slow_lateral_drift"));
        assert!(AURA_FRAGMENT_SHADER.contains("along_distance"));
        assert!(AURA_FRAGMENT_SHADER.contains("inward_distance"));
        assert!(AURA_FRAGMENT_SHADER.contains("cycle_envelope"));
        assert!(AURA_FRAGMENT_SHADER.contains("previous frame"));
        assert!(AURA_FRAGMENT_SHADER.contains("color_coordinate"));
        assert!(AURA_FRAGMENT_SHADER.contains("ambient_fluid_field"));
        assert!(AURA_FRAGMENT_SHADER.contains("(low.x + low.y)"));
        assert!(AURA_FRAGMENT_SHADER.contains("(high.z + high.w)"));
        assert!(AURA_FRAGMENT_SHADER.contains("material_gate"));
        assert!(AURA_FRAGMENT_SHADER.contains("wave_wake"));
        assert!(AURA_FRAGMENT_SHADER.contains("wave_crest"));
        assert!(AURA_FRAGMENT_SHADER.contains("neutral_outer_cut"));
        assert!(AURA_FRAGMENT_SHADER.contains("neutral_inner_cut"));
        assert!(!AURA_FRAGMENT_SHADER.contains("CAUSTIC_EDGE_DEPTH"));
        assert!(!AURA_FRAGMENT_SHADER.contains("broad_caustic_pulse"));
        assert!(!AURA_FRAGMENT_SHADER.contains("precision_rgb_rail"));
        assert!(!AURA_FRAGMENT_SHADER.contains("peak_cap"));
        assert!(!AURA_FRAGMENT_SHADER.contains("atan(point.y, point.x)"));
        assert!(!AURA_FRAGMENT_SHADER.contains("rounded_perimeter_u"));
        assert!(!AURA_FRAGMENT_SHADER.contains("circular_distance"));
        assert!(!AURA_FRAGMENT_SHADER.contains("analysis_pearl"));
        assert!(!AURA_FRAGMENT_SHADER.contains("u_primary_peak_low"));
        assert!(AURA_FRAGMENT_SHADER.contains("0.76, 0.98"));
        assert!(AURA_FRAGMENT_SHADER.contains("cos(mix_progress * PI * 0.5)"));
        assert!(AURA_FRAGMENT_SHADER.contains("sin(mix_progress * PI * 0.5)"));
    }

    #[test]
    fn aura_dynamics_are_capped_at_thirty_fps_between_track_changes() {
        assert!(!aura_dynamics_should_advance(
            Duration::from_millis(16),
            false,
            false,
        ));
        assert!(aura_dynamics_should_advance(
            AURA_TARGET_FRAME_INTERVAL,
            false,
            false,
        ));
    }

    #[test]
    fn aura_dynamics_refresh_immediately_for_new_track_identity() {
        assert!(aura_dynamics_should_advance(Duration::ZERO, false, true,));
        assert!(aura_dynamics_should_advance(Duration::ZERO, true, false,));
    }
}
