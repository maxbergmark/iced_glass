struct Uniforms {
    tint: vec4<f32>,
    scrim: vec4<f32>,
    fill_color: vec4<f32>,
    blur_direction: vec2<f32>,
    content_scale: vec2<f32>,
    blur_radius: f32,
    corner_radius: f32,
    saturation: f32,
    lightness: f32,
    edge_radius: f32,
    height: f32,
    refractive_index: f32,
    rim_width: f32,
    opacity: f32,
    edge_type: i32,
    chromatic_aberration: f32,
    rim_angle: f32,
    num_children: u32,
    blending_factor: f32,
    fill_level: f32,
    _pad2: f32,
};

struct Child {
    center: vec2<f32>,
    half_size: vec2<f32>,
};

@group(1)
@binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(1)
var<storage, read> children: array<Child>;

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

const TRANSPARENT: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 0.0);
const GLASS_EDGE: i32 = 0;
const SMOOTH_EDGE: i32 = 1;

@fragment
fn fs_main(input: FragInput) -> @location(0) vec4<f32> {
    if uniforms.edge_type == GLASS_EDGE {
        var color = physical_sampling(input);
        color.a *= uniforms.opacity;
        return linear_to_srgb(color);
    } else {
        var color = soft_edge_sampling(input);
        color.a *= uniforms.opacity;
        return linear_to_srgb(color);
    }
}

fn soft_edge_sampling(input: FragInput) -> vec4<f32> {
    let ixy = input.uv - vec2<f32>(0.5);
    let dimensions = vec2<f32>(textureDimensions(image));
    let p = ixy * dimensions;

    let r = clamp_radius(uniforms.corner_radius, dimensions);
    // let sdf_gradient = sdg_rounded_box(p, dimensions / 2.0, r);
    let sdf_gradient = sdf(p, dimensions, r);
    let gradient = sdf_gradient.yz;
    let sdf = sdf_gradient.x;

    let aa = fwidth(sdf);
    let outside_factor = smoothstep(-aa, aa, sdf);
    var color = textureSample(image, image_sampler, input.uv);
    color = blend_fill_color(color, input.uv.x);
    color = saturate(color);
    let scrim = vec4<f32>(uniforms.scrim.rgb, 1.0);
    color = mix(color, scrim, uniforms.scrim.a);
    color *= uniforms.tint;

    let edge_factor = smoothstep(-uniforms.edge_radius, 0.0, sdf);
    color = srgb_to_linear(color);
    color.a *= (1.0 - edge_factor) * (1.0 - outside_factor);
    return color;
}

// returns linear color
fn physical_sampling(input: FragInput) -> vec4<f32> {
    let ixy = input.uv - vec2<f32>(0.5);
    let dimensions = vec2<f32>(textureDimensions(image));
    let p = ixy * dimensions;

    let r = clamp_radius(uniforms.corner_radius, dimensions);
    let sdf_gradient = sdf(p, dimensions, r);
    let gradient = sdf_gradient.yz;
    let sdf = sdf_gradient.x;

    let h = uniforms.height;
    let n_r = max(uniforms.refractive_index - uniforms.chromatic_aberration, 1.0);
    let n_g = max(uniforms.refractive_index, 1.0);
    let n_b = max(uniforms.refractive_index + uniforms.chromatic_aberration, 1.0);
    let r_edge = clamp_radius(uniforms.edge_radius, dimensions);

    var red = sample_color_channel(p, sdf, gradient, r_edge, n_r, h, dimensions, input.uv.x);
    var green = sample_color_channel(p, sdf, gradient, r_edge, n_g, h, dimensions, input.uv.x);
    var blue = sample_color_channel(p, sdf, gradient, r_edge, n_b, h, dimensions, input.uv.x);
    var color = vec4<f32>(red.r, green.g, blue.b, green.a);

    return color;
}

fn sample_color_channel(p: vec2<f32>, sdf: f32, gradient: vec2<f32>, r_edge: f32, n: f32, h: f32, dimensions: vec2<f32>, x: f32) -> vec4<f32> {
    let dx = select(0.0, refract(sdf, r_edge, n, h), sdf > -r_edge);
    let offset = gradient * dx;

    let sample_uv = (p + offset) / dimensions + vec2<f32>(0.5);

    var color = textureSample(image, image_sampler, sample_uv);
    color = blend_fill_color(color, x);

    color = srgb_to_linear(color);
    color = saturate(color);
    let scrim = vec4<f32>(uniforms.scrim.rgb, 1.0);
    color = mix(color, scrim, uniforms.scrim.a);
    color *= uniforms.tint;
    color = edge_highlight(color, sdf, gradient);

    let aa = fwidth(sdf);
    let outside_factor = smoothstep(-aa, aa, sdf);
    color.a *= (1.0 - outside_factor);
    return color;
}

fn blend_fill_color(color: vec4<f32>, x: f32) -> vec4<f32> {
    let fill_color = vec4<f32>(uniforms.fill_color.rgb, 1.0);
    let fill_level = uniforms.fill_level;
    let aa = fwidth(x);
    let a = 1.0 - smoothstep(fill_level - aa, fill_level + aa, x);
    return mix(color, fill_color, uniforms.fill_color.a * a);
}

fn sdf(p: vec2<f32>, dimensions: vec2<f32>, r: f32) -> vec3<f32> {
    if uniforms.num_children > 0 {
        return rounded_group_sdf(p, r);
    } else {
        return sdg_rounded_box(p, dimensions / 2.0, r);
    }
}

fn rounded_group_sdf(p: vec2<f32>, r: f32) -> vec3<f32> {
    var best_sdf = 1e20;
    var best_grad = vec2<f32>(0.0);
    let k = uniforms.blending_factor;
    let n = uniforms.num_children;
    for (var i: u32 = 0u; i < n; i = i + 1u) {
        let c = children[i];
        let local_p = p - c.center;
        let r_i = min(r, min(c.half_size.x, c.half_size.y));
        let sdg = sdg_box(local_p, c.half_size - r_i);
        let d = sdg.x - r_i; // <-- per-child inflate 
        let s = smin(best_sdf, d, k);
        best_sdf = s.x;
        best_grad = mix(best_grad, sdg.yz, s.y);
    }
    return vec3<f32>(best_sdf, best_grad);
}

// Returns (smoothed_sdf, t) where t in [0,1] is the blend factor for `b`.
// k > 0. With k -> 0 this reduces to min(a,b) with t = step(b, a).
fn smin(a: f32, b: f32, k: f32) -> vec2<f32> {
    let h = max(k - abs(a - b), 0.0) / max(k, 1e-6);
    let m = h * h * 0.5;
    let s = m * k * 0.5;
    if a < b {
        return vec2<f32>(a - s, m);
    } else {
        return vec2<f32>(b - s, 1.0 - m);
    }
}

fn edge_highlight(color: vec4<f32>, sdf: f32, sdf_gradient: vec2<f32>) -> vec4<f32> {
    let aa = fwidth(sdf);

    let highlight_color = apply_glass_exposure(color, vec4<f32>(1.0), 3.0);
    let highlight_width = uniforms.rim_width;
    let sun_direction = normalize(vec2<f32>(cos(uniforms.rim_angle), sin(uniforms.rim_angle)));
    let f = pow(dot(sdf_gradient, sun_direction), 2.0);
    let t = smoothstep(-highlight_width - aa, -highlight_width + aa, sdf);
    return mix(color, highlight_color, f * t);
}

fn sdg_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> vec3<f32> {
    let dis_gra = sdg_box(p, b - r);
    return vec3<f32>(dis_gra.x - r, dis_gra.y, dis_gra.z);
}

fn sdg_box(p: vec2<f32>, b: vec2<f32>) -> vec3<f32> {
    let w = abs(p) - b;
    let s = vec2<f32>(
        select(1.0, -1.0, p.x < 0.0),
        select(1.0, -1.0, p.y < 0.0)
    );
    let g = max(w.x, w.y);
    let q = max(w, vec2<f32>(0.0));
    let l = length(q);

    return vec3<f32>(
        select(g, l, g > 0.0),
        s * select(
            select(vec2<f32>(0, 1), vec2<f32>(1, 0), w.x > w.y),
            q / l,
            g > 0.0
        ),
    );
}

fn clamp_radius(radius: f32, dimensions: vec2<f32>) -> f32 {
    return min(radius, min(dimensions.x, dimensions.y) / 2.0);
}

fn refract(x: f32, r: f32, n: f32, h: f32) -> f32 {
    let x2 = x + r;
    let z = sqrt(r * r - x2 * x2);
    let theta = atan(x2 / z);
    let gamma = asin(sin(theta) / n);
    let beta = theta - gamma;
    let dx = -(z + h) * tan(beta);
    return dx;
}

fn saturate(rgba: vec4<f32>) -> vec4<f32> {
    let exposed = apply_glass_exposure(rgba, vec4<f32>(1.0), uniforms.lightness);
    let hsv = rgb_to_hsv(exposed);
    return hsv_to_rgb(vec4<f32>(hsv.x, hsv.y * uniforms.saturation, hsv.z, rgba.a));
}

// ---------- HSV <-> RGB ----------
const EPSILON: f32 = 1e-10;
fn rgb_to_hsv(rgb: vec4<f32>) -> vec4<f32> {
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
    return vec4<f32>(hcv.x, s, hcv.z, rgb.a);
}

fn hsv_to_rgb(hsv: vec4<f32>) -> vec4<f32> {
    let h = hsv.x;
    let r = abs(h * 6.0 - 3.0) - 1.0;
    let g = 2.0 - abs(h * 6.0 - 2.0);
    let b = 2.0 - abs(h * 6.0 - 4.0);
    var rgb = clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0));
    rgb = ((rgb - vec3<f32>(1.0)) * hsv.y + vec3<f32>(1.0)) * hsv.z;
    return vec4<f32>(rgb, hsv.a);
}

// ---------- sRGB <-> linear ----------
fn srgb_to_linear(c: vec4<f32>) -> vec4<f32> {
    let cutoff = vec4<f32>(0.04045);
    var low = c / 12.92;
    var high = pow((c + vec4<f32>(0.055)) / 1.055, vec4<f32>(2.4));
    low.a = c.a;
    high.a = c.a;
    return select(high, low, c <= cutoff);
}

fn linear_to_srgb(c: vec4<f32>) -> vec4<f32> {
    let cutoff = vec4<f32>(0.0031308);
    var low = 12.92 * c;
    var high = 1.055 * pow(c, vec4<f32>(1.0 / 2.4)) - vec4<f32>(0.055);
    low.a = c.a;
    high.a = c.a;
    return select(high, low, c <= cutoff);
}

// ---------- "Dark glass" exposure ----------
// Attenuate an sRGB color as if it passed through a piece of glass with the
// given per-channel transmission (`tint`, each channel in [0, 1]) and
// exposure offset (`ev_stops`, negative = darker). All physics happens in
// linear-light space.
fn apply_glass_exposure(
    lin: vec4<f32>,
    tint: vec4<f32>,
    ev_stops: f32,
) -> vec4<f32> {
    // let lin = srgb_to_linear(srgb_in);
    let filtered = lin * tint * exp2(ev_stops);
    return clamp(filtered, vec4<f32>(0.0), vec4<f32>(1.0));
}