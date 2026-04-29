struct Uniforms {
    tint: vec4<f32>,
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
    _pad: f32,
    _pad2: f32,
    _pad3: f32,
};

@group(1)
@binding(0)
var<uniform> uniforms: Uniforms;

@group(0)
@binding(0)
var texture_atlas: texture_2d<f32>;

@group(0)
@binding(1)
var image: texture_2d<f32>;

@group(0)
@binding(2)
var image_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) atlas_uv: vec2<f32>,
    @location(2) scale: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) atlas_uv: vec2<f32>,
    @location(1) screen_uv: vec2<f32>,
    @location(2) scale: f32,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {

    var out: VertexOutput;
    let pos = input.position;
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.atlas_uv = input.atlas_uv;
    out.screen_uv = vec2<f32>(pos.x * 0.5 + 0.5, 1.0 - (pos.y * 0.5 + 0.5));
    // out.screen_uv *= uniforms.content_scale;
    out.scale = input.scale;
    return out;
}

struct FragInput {
    @location(0) atlas_uv: vec2<f32>,
    @location(1) screen_uv: vec2<f32>,
    @location(2) scale: f32,
};

const TRANSPARENT: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);

@fragment
fn fs_main(input: FragInput) -> @location(0) vec4<f32> {
    // return vec4<f32>(input.atlas_uv, 1.0, 1.0);
    // let alpha = msdf_alpha(input.atlas_uv);
    // return vec4<f32>(input.atlas_uv, 1.0, alpha);
    // let alpha = msdf_alpha(input.uv);
    // return vec4<f32>(1.0, 1.0, 1.0, alpha);
    // return anti_alias(input);
    return mix(TRANSPARENT, physical_sampling(input), uniforms.opacity);
    // let bg = textureSample(image, image_sampler, input.uv);
    let sdf = msdf(input.atlas_uv, input.scale);
    return vec4<f32>(fwidth(sdf) * 10.0, 0.0, 0.0, 1.0);
    // return mix(vec4<f32>(0.0), bg, sdf);
    let gradient = sdf_gradient(input.atlas_uv);
    // return vec4<f32>(msdf_alpha(input.uv));
    // let sdg = sdf_gradient(input.uv);
    let green = clamp(sdf / uniforms.edge_radius + 1.0, 0.0, 1.0);
    return vec4<f32>(gradient.x * 0.5 + 0.5, green, gradient.y * 0.5 + 0.5, 1.0);
}

fn anti_alias(input: FragInput) -> vec4<f32> {
    var i = input;
    let dimensions = vec2<f32>(textureDimensions(image)) * uniforms.content_scale;

    let c1 = physical_sampling_with_opacity(i);
    i.screen_uv = input.screen_uv - vec2<f32>(1.0 / dimensions.x, 0.0);
    let c2 = physical_sampling_with_opacity(i);
    i.screen_uv = input.screen_uv - vec2<f32>(0.0, 1.0 / dimensions.y);
    let c3 = physical_sampling_with_opacity(i);
    i.screen_uv = input.screen_uv + vec2<f32>(1.0 / dimensions.x, 0.0);
    let c4 = physical_sampling_with_opacity(i);
    i.screen_uv = input.screen_uv + vec2<f32>(0.0, 1.0 / dimensions.y);
    let c5 = physical_sampling_with_opacity(i);
    return (c1 + c2 + c3 + c4 + c5) / 5.0;
}

fn physical_sampling_with_opacity(input: FragInput) -> vec4<f32> {
    return mix(TRANSPARENT, physical_sampling(input), uniforms.opacity);
}


// fn msdf(uv: vec2<f32>, scale: f32) -> f32 {
//     let dimensions = vec2<f32>(textureDimensions(texture_atlas));
//     let s = textureSample(texture_atlas, image_sampler, uv).rgb;
//     let d = median(s.r, s.g, s.b);
//     return (0.5 - d) * 1.0 * scale;
// }

// fn median(r: f32, g: f32, b: f32) -> f32 {
//     return max(min(r, g), min(max(r, g), b));
// }

// fn sdf_gradient(uv: vec2<f32>) -> vec2<f32> {
//     let texel = 1.0 / vec2<f32>(textureDimensions(texture_atlas));

//     let c  = textureSample(texture_atlas, image_sampler, uv).rgb;
//     let r  = textureSample(texture_atlas, image_sampler, uv + vec2<f32>(texel.x, 0.0)).rgb;
//     let l  = textureSample(texture_atlas, image_sampler, uv - vec2<f32>(texel.x, 0.0)).rgb;
//     let u  = textureSample(texture_atlas, image_sampler, uv + vec2<f32>(0.0, texel.y)).rgb;
//     let dn = textureSample(texture_atlas, image_sampler, uv - vec2<f32>(0.0, texel.y)).rgb;

//     let ch = median_channel(c);
//     let gx = (r[ch] - l[ch]) * 0.5;
//     let gy = (u[ch] - dn[ch]) * 0.5;
//     let g = vec2<f32>(gx, gy);
//     let len = length(g);
//     return select(vec2<f32>(0.0), g / len, len > 1e-6);
// }

// fn median_channel(c: vec3<f32>) -> u32 {
//     if (c.r >= c.g && c.r <= c.b) || (c.r <= c.g && c.r >= c.b) {
//         return 0u;
//     }
//     if (c.g >= c.r && c.g <= c.b) || (c.g <= c.r && c.g >= c.b) {
//         return 1u;
//     }
//     return 2u;
// }

fn msdf(uv: vec2<f32>, scale: f32) -> f32 {
    let d = msdf_bilinear(uv);
    return (0.5 - d) * scale;
}
fn median(r: f32, g: f32, b: f32) -> f32 {
    return max(min(r, g), min(max(r, g), b));
}
fn texel_median(coord: vec2<i32>) -> f32 {
    let s = textureLoad(texture_atlas, coord, 0).rgb;
    return median(s.r, s.g, s.b);
}
// Bilinear interpolation of per-texel median values.
// Computing median before interpolation avoids the classic MSDF
// medial-axis artifact caused by independently filtering the channels.
fn msdf_bilinear(uv: vec2<f32>) -> f32 {
    let size = vec2<f32>(textureDimensions(texture_atlas));
    let pos = uv * size - 0.5;
    let i = vec2<i32>(floor(pos));
    let f = fract(pos);
    let d00 = texel_median(i);
    let d10 = texel_median(i + vec2(1, 0));
    let d01 = texel_median(i + vec2(0, 1));
    let d11 = texel_median(i + vec2(1, 1));
    return mix(mix(d00, d10, f.x), mix(d01, d11, f.x), f.y);
}
fn sdf_gradient(uv: vec2<f32>) -> vec2<f32> {
    let size = vec2<f32>(textureDimensions(texture_atlas));
    let center = vec2<i32>(round(uv * size - 0.5));
    let r = texel_median(center + vec2(1, 0));
    let l = texel_median(center + vec2(-1, 0));
    let u = texel_median(center + vec2(0, 1));
    let d = texel_median(center + vec2(0, -1));
    let gx = (r - l) * 0.5;
    let gy = (u - d) * 0.5;
    let g = vec2<f32>(gx, gy);
    let len = length(g);
    return select(vec2<f32>(0.0), g / len, len > 1e-6);
}

// fn msdf_distance(uv: vec2<f32>) -> f32 {
//     let s = textureSample(image, image_sampler, uv).rgb;
//     return median(s.r, s.g, s.b);
// }

fn physical_sampling(input: FragInput) -> vec4<f32> {
    let ixy = input.atlas_uv - vec2<f32>(0.5);
    let dimensions = vec2<f32>(textureDimensions(image)) * uniforms.content_scale;
    // return textureSample(image, image_sampler, input.screen_uv);
    let p = ixy * dimensions;

    let p_screen = (input.screen_uv - vec2<f32>(0.5)) * dimensions;

    // let angle = atan2(p.y, p.x);

    let r = clamp_radius(uniforms.corner_radius, dimensions);
    // let sdf_gradient = sdg_rounded_box(p, dimensions / 2.0, r);
    let gradient = sdf_gradient(input.atlas_uv);
    let sdf = msdf(input.atlas_uv, input.scale);

    let h = uniforms.height;
    let n = uniforms.refractive_index;
    let r_edge = clamp_radius(uniforms.edge_radius, dimensions);
    let dx = select(0.0, refract(sdf, r_edge, n, h), sdf > -r_edge && sdf < 0.0);
    let offset = gradient * dx;

    var sample_uv = (p_screen + offset) / dimensions + vec2<f32>(0.5);
    sample_uv *= uniforms.content_scale;

    var color = textureSample(image, image_sampler, sample_uv);
    color = saturate(color);
    color *= uniforms.tint;
    color = edge_highlight(color, sdf, gradient);

    let aa = fwidth(sdf);
    // let outside_factor = smoothstep(-aa, 0.0, sdf);
    let outside_factor = smoothstep(-aa, aa, sdf);
    return mix(color, TRANSPARENT, outside_factor);
}

fn edge_highlight(color: vec4<f32>, sdf: f32, sdf_gradient: vec2<f32>) -> vec4<f32> {
    let aa = fwidth(sdf);

    let highlight_color = apply_glass_exposure(color, vec4<f32>(1.0), 3.0);
    let highlight_width = uniforms.rim_width;
    let sun_direction = normalize(vec2<f32>(1.0, 2.0));
    let f = pow(dot(sdf_gradient, sun_direction), 2.0);
    let t = smoothstep(-highlight_width - aa, -highlight_width + aa, sdf);
    return mix(color, highlight_color, f * t);
}


fn sdg_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> vec3<f32> {
    let dis_gra = sdg_box(p, b - r);
    return vec3<f32>( dis_gra.x - r, dis_gra.y, dis_gra.z );
}

fn sdg_box( p: vec2<f32>, b: vec2<f32> ) -> vec3<f32> {
    let w = abs(p)-b;
    let s = vec2<f32>(
        select(1.0, -1.0, p.x < 0.0),
        select(1.0, -1.0, p.y < 0.0));
    let g = max(w.x,w.y);
    let  q = max(w,vec2<f32>(0.0));
    let l = length(q);


    return vec3<f32>(   
        select(g, l, g > 0.0),
        s * select(
            select(vec2<f32>(0,1), vec2<f32>(1,0), w.x>w.y),
            q/l,
            g>0.0
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