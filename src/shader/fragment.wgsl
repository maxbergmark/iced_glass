struct Uniforms {
    blur_radius: f32,
    corner_radius: f32,
    saturation: f32,
    lightness: f32,
    blur_direction: vec2<f32>,
    edge_radius: f32,
    height: f32,
    refractive_index: f32,
    rim_width: f32,
    opacity: f32,
    _pad: f32,
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

const TRANSPARENT: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);

@fragment
fn fs_main(input: FragInput) -> @location(0) vec4<f32> {
    return mix(TRANSPARENT, physical_sampling(input), uniforms.opacity);
}

fn physical_sampling(input: FragInput) -> vec4<f32> {
    let ixy = input.uv - vec2<f32>(0.5);
    let dimensions = vec2<f32>(textureDimensions(image));
    let pxy = ixy * dimensions;

    let angle = atan2(pxy.y, pxy.x);
    let sign = sign(pxy);
    let xy = pxy * sign;

    let r = clamp_radius(uniforms.corner_radius, dimensions);
    let sdf = sd_rounded_box(xy, dimensions / 2.0, r);
    let outside_factor = smoothstep(0.0, 1.0, sdf);
    let offset = compute_offset(xy.x, xy.y, dimensions);
    let sample_uv = (xy + offset) * sign / dimensions + vec2<f32>(0.5);

    var color = textureSample(image, image_sampler, sample_uv);
    color = saturate(color);
    color = edge_highlight(color, xy, dimensions, angle);

    return mix(color, TRANSPARENT, outside_factor);
}

fn edge_highlight(color: vec4<f32>, xy: vec2<f32>, dimensions: vec2<f32>, angle: f32) -> vec4<f32> {
    let r = clamp_radius(uniforms.corner_radius, dimensions);
    
    let sdf = sd_rounded_box(xy, dimensions / 2.0, r);
    let highlight_color = apply_glass_exposure(color, vec4<f32>(1.0), 3.0);
    let highlight_width = uniforms.rim_width;
    let highlight_angle = 2.0 * angle - 1.0;
    let f = 0.5 + 0.5 * cos(highlight_angle);
    let t = smoothstep(-highlight_width - 1.0, -highlight_width, sdf);
    return mix(color, highlight_color, f * t);
}

fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

fn clamp_radius(radius: f32, dimensions: vec2<f32>) -> f32 {
    // TODO: why 2.0?
    return min(2.0 * radius, min(dimensions.x, dimensions.y) / 2.0) * 1.0;
}

fn compute_offset(x: f32, y: f32, dimensions: vec2<f32>) -> vec2<f32> {
    let r = clamp_radius(uniforms.edge_radius, dimensions);
    let corner_radius = clamp_radius(uniforms.corner_radius, dimensions);
    let h = uniforms.height;
    let n = uniforms.refractive_index;
    let r_other = corner_radius - r;
    let max_radius = max(r, corner_radius);
    if x > dimensions.x / 2.0 - max_radius && y > dimensions.y / 2.0 - max_radius {
        if r_other > 0.0 {
            let x2 = x - dimensions.x / 2.0 + corner_radius;
            let y2 = y - dimensions.y / 2.0 + corner_radius;
            let d2 = x2 * x2 + y2 * y2;
            if d2 <= r_other * r_other {
                return vec2<f32>(0.0);
            }
            let alpha = atan(y2 / x2);
            let p2 = sqrt(d2) - r_other;
            let z = sqrt(r * r - p2 * p2);
            let theta = atan(p2 / z);
            let gamma = asin(sin(theta) / n);
            let beta = theta - gamma;
            let dx = -(z + h) * tan(beta);
            return vec2<f32>(dx * cos(alpha), dx * sin(alpha));
        } else {
            // Avoid this case if possible, doesn't look great.
            let x2 = x - dimensions.x / 2.0 + corner_radius;
            let y2 = y - dimensions.y / 2.0 + corner_radius;
            if x2 < 0.0 || y2 < 0.0 {
                let dx = refract_edge(x, dimensions.x, r, n, h);
                let dy = refract_edge(y, dimensions.y, r, n, h);
                let t = smoothstep(0.0, 1.0, (abs(dy) - abs(dx) + 0.0) / 2.0 + 0.5);
                return mix(vec2<f32>(dx, 0.0), vec2<f32>(0.0, dy), t);
            } else {
                let d2 = x2 * x2 + y2 * y2;
                let alpha = atan(y2 / x2);
                let p2 = sqrt(d2) - r_other;
                let z = sqrt(r * r - p2 * p2);
                let theta = atan(p2 / z);
                let gamma = asin(sin(theta) / n);
                let beta = theta - gamma;
                let dx = -(z + h) * tan(beta);
                return vec2<f32>(dx * cos(alpha), dx * sin(alpha));
            }

        }
    } else if x > dimensions.x / 2.0 - r {
        let dx = refract_edge(x, dimensions.x, r, n, h);
        return vec2<f32>(dx, 0.0);
    } else if y > dimensions.y / 2.0 - r {
        let dy = refract_edge(y, dimensions.y, r, n, h);
        return vec2<f32>(0.0, dy);
    } else {
        return vec2<f32>(0.0);
    }
}

fn refract_edge(x: f32, dim: f32, r: f32, n: f32, h: f32) -> f32 {
    let x2 = x - dim / 2.0 + r;
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
    var low    = c / 12.92;
    var high   = pow((c + vec4<f32>(0.055)) / 1.055, vec4<f32>(2.4));
    low.a = c.a;
    high.a = c.a;
    return select(high, low, c <= cutoff);
}

fn linear_to_srgb(c: vec4<f32>) -> vec4<f32> {
    let cutoff = vec4<f32>(0.0031308);
    var low    = 12.92 * c;
    var high   = 1.055 * pow(c, vec4<f32>(1.0 / 2.4)) - vec4<f32>(0.055);
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
    srgb_in: vec4<f32>,
    tint: vec4<f32>,
    ev_stops: f32,
) -> vec4<f32> {
    let lin      = srgb_to_linear(srgb_in);
    let filtered = lin * tint * exp2(ev_stops);
    return linear_to_srgb(clamp(filtered, vec4<f32>(0.0), vec4<f32>(1.0)));
}