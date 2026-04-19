struct Uniforms {
    blur_radius: f32,
    corner_radius: f32,
    saturation: f32,
    lightness: f32,
};

@group(1)
@binding(0)
var<uniform> uniforms: Uniforms;

@group(0)
@binding(0)
var image: texture_2d<f32>;

@group(0)
@binding(1)
var image_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0)
    );

    let pos = positions[vertex_index];
    var out: VertexOutput;
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = pos * 0.5 + vec2<f32>(0.5, 0.5);
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

struct FragInput {
    @location(0) uv: vec2<f32>,
};

const PI: f32 = 3.14159265358979323846;
const BLUE_REFRACTION_INDEX: f32 = 1.535;
const GREEN_REFRACTION_INDEX: f32 = 1.526;
const RED_REFRACTION_INDEX: f32 = 1.521;
const RADIUS: f32 = 0.10;

// Tunables for the rim highlight. Convert to uniforms for live mouse tracking.
// mouse_offset is in % of widget size, matching the React component.
const MOUSE_OFFSET: vec2<f32> = vec2<f32>(0.0, 0.0);
const BORDER_WIDTH_PX: f32 = 3.0;

fn smoothStep(a: f32, b: f32, t_in: f32) -> f32 {
    let t = max(0.0, min(1.0, (t_in - a) / (b - a)));
    return t * t * (3.0 - 2.0 * t);
}

fn roundedRectSDF(x: f32, y: f32, width: f32, height: f32, radius: f32) -> f32 {
    let qx = abs(x) - width + radius;
    let qy = abs(y) - height + radius;
    let q = vec2<f32>(max(qx, 0.0), max(qy, 0.0));
    return min(max(qx, qy), 0.0) + length(q) - radius;
}

fn screen_blend(base: vec3<f32>, over: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(1.0) - (vec3<f32>(1.0) - base) * (vec3<f32>(1.0) - over);
}

fn overlay_blend(base: vec3<f32>, over: vec3<f32>) -> vec3<f32> {
    let lo = 2.0 * base * over;
    let hi = vec3<f32>(1.0) - 2.0 * (vec3<f32>(1.0) - base) * (vec3<f32>(1.0) - over);
    return select(hi, lo, base < vec3<f32>(0.5));
}

// Evaluate the 4-stop diagonal rim highlight gradient.
// `t` is the position along the rotated axis, normalized to 0..1.
fn rim_gradient(t: f32, base_alpha_1: f32, base_alpha_2: f32) -> f32 {
    let stop1 = clamp(0.33 + MOUSE_OFFSET.y * 0.003, 0.10, 0.90);
    let stop2 = clamp(0.66 + MOUSE_OFFSET.y * 0.004, 0.10, 0.90);
    let a1 = base_alpha_1 + abs(MOUSE_OFFSET.x) * 0.00008;
    let a2 = base_alpha_2 + abs(MOUSE_OFFSET.x) * 0.00012;

    if t < stop1 {
        return mix(0.0, a1, smoothStep(0.0, stop1, t));
    } else if t < stop2 {
        return mix(a1, a2, (t - stop1) / (stop2 - stop1));
    }
    return mix(a2, 0.0, smoothStep(stop2, 1.0, t));
}

@fragment
fn fs_main(input: FragInput) -> @location(0) vec4<f32> {
    // UV-space offsets from center (kept for texture sampling).
    let ix = input.uv.x - 0.5;
    let iy = input.uv.y - 0.5;
    let dimensions = vec2<f32>(textureDimensions(image));
    let min_dim = min(dimensions.x, dimensions.y);
    // Aspect-corrected position. One unit in `p` = one unit of the shorter side.
    // For a 800x400 widget: aspect = (2, 1), half = (1, 0.5), and p ranges over
    // [-1..1] on x and [-0.5..0.5] on y.
    let aspect = dimensions / min_dim;
    let half   = 0.5 * aspect;
    let p      = vec2<f32>(ix, iy) * aspect;
    // One physical pixel, expressed in the aspect-corrected space.
    let px = 1.0 / min_dim;
    // --- Shape mask (now isotropic) ---
    let corner_radius = min(uniforms.corner_radius, min(dimensions.x, dimensions.y) / 2.0) * px;
    let shape_sdf = roundedRectSDF(p.x, p.y, half.x, half.y, corner_radius);
    if shape_sdf > 0.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    // --- Background sampling with edge displacement ---
    // The displacement "inner rect" used to be (0.3, 0.2, 0.6) inside a (0.5, 0.5) box.
    // Re-express those insets relative to `half` so the effect matches at any aspect.
    let inner = half - vec2<f32>(0.2, 0.3);
    let distanceToEdge = roundedRectSDF(p.x, p.y, inner.x, inner.y, 0.6);
    let displacement = smoothStep(0.8, 0.0, distanceToEdge - 0.15);
    let scaled = smoothStep(0.0, 1.0, displacement);
    // Sampling happens in UV space, so use `ix`/`iy` (not `p`).
    let sample_uv = vec2<f32>(ix * scaled + 0.5, iy * scaled + 0.5);
    var color = textureSample(image, image_sampler, sample_uv);
    color = saturate(color);
    // --- Ring mask / rim highlight ---
    let border_width = BORDER_WIDTH_PX * px;
    let ring_outer = 1.0 - smoothStep(-0.5 * px, 0.5 * px, shape_sdf);
    let ring_inner = 1.0 - smoothStep(-border_width - 0.5 * px, -border_width + 0.5 * px, shape_sdf);
    let ring = clamp(ring_outer - ring_inner, 0.0, 1.0);
    // Diagonal projection uses the aspect-corrected position so 135° means 135°
    // visually, regardless of widget shape.
    let angle_rad = (135.0 + MOUSE_OFFSET.x * 1.2) * PI / 180.0;
    let dir = vec2<f32>(cos(angle_rad), sin(angle_rad));
    let diag_len = length(half);                 // was hard-coded sqrt(0.5) for 0.5x0.5
    let t_proj = dot(p, dir) / diag_len * 0.5 + 0.5;
    let rim = vec3<f32>(1.0);
    let screen_amount = rim_gradient(t_proj, 0.12, 0.40) * ring * 0.2;
    color = vec4<f32>(
        mix(color.rgb, screen_blend(color.rgb, rim * screen_amount), ring),
        color.a,
    );
    let overlay_amount = rim_gradient(t_proj, 0.32, 0.60) * ring;
    color = vec4<f32>(
        mix(color.rgb, overlay_blend(color.rgb, rim), overlay_amount),
        color.a,
    );
    // --- Inset highlights ---
    let edge_line = exp(-abs(shape_sdf + 0.5 * px) / (0.5 * px)) * 0.5;
    color = vec4<f32>(color.rgb + rim * edge_line * ring_outer, color.a);
    // Top glow: compare against `p.y` against the top of the shape (`-half.y`),
    // so the glow fades in at the same visual distance regardless of aspect.
    let near_edge = 1.0 - smoothStep(0.0, border_width * 3.0, -shape_sdf);
    let near_top = smoothStep(-0.7 * half.y, -half.y, p.y);
    color = vec4<f32>(color.rgb + rim * near_edge * near_top * 0.25, color.a);
    return color;
}

fn saturate(rgba: vec4<f32>) -> vec4<f32> {
    let exposed = apply_glass_exposure(rgba.rgb, vec3<f32>(1.0, 1.0, 1.0), uniforms.lightness);
    let hsv = rgb_to_hsv(exposed);
    return vec4<f32>(hsv_to_rgb(vec3<f32>(hsv.x, hsv.y * uniforms.saturation, hsv.z)), rgba.a);
}

const EPSILON: f32 = 1e-10;
fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
    let p = select(
        vec4<f32>(rgb.gb, 0.0, -1.0 / 3.0),
        vec4<f32>(rgb.bg, -1.0, 2.0 / 3.0),
        rgb.g < rgb.b,
    );
    let q = select(
        vec4<f32>(rgb.r, p.yzx),
        vec4<f32>(p.xyw, rgb.r),
        rgb.r < p.x,
    );
    let c = q.x - min(q.w, q.y);
    let h = abs((q.w - q.y) / (6.0 * c + EPSILON) + q.z);
    let hcv = vec3<f32>(h, c, q.x);
    let s = hcv.y / (hcv.z + EPSILON);
    return vec3<f32>(hcv.x, s, hcv.z);
}

fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let h = hsv.x;
    let r = abs(h * 6.0 - 3.0) - 1.0;
    let g = 2.0 - abs(h * 6.0 - 2.0);
    let b = 2.0 - abs(h * 6.0 - 4.0);
    let rgb = clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0));
    return ((rgb - vec3<f32>(1.0)) * hsv.y + vec3<f32>(1.0)) * hsv.z;
}

// ---------- sRGB <-> linear ----------
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let cutoff = vec3<f32>(0.04045);
    let low    = c / 12.92;
    let high   = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(high, low, c <= cutoff);
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let cutoff = vec3<f32>(0.0031308);
    let low    = 12.92 * c;
    let high   = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    return select(high, low, c <= cutoff);
}

// ---------- "Dark glass" exposure ----------
// Attenuate an sRGB color as if it passed through a piece of glass with the
// given per-channel transmission (`tint`, each channel in [0, 1]) and
// exposure offset (`ev_stops`, negative = darker). All physics happens in
// linear-light space.
fn apply_glass_exposure(
    srgb_in: vec3<f32>,
    tint: vec3<f32>,
    ev_stops: f32,
) -> vec3<f32> {
    let lin      = srgb_to_linear(srgb_in);
    let filtered = lin * tint * exp2(ev_stops);
    return linear_to_srgb(clamp(filtered, vec3<f32>(0.0), vec3<f32>(1.0)));
}