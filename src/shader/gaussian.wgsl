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
        vec2<f32>(-1.0, -1.0), // bottom left
        vec2<f32>(1.0, -1.0), // bottom right
        vec2<f32>(-1.0, 1.0), // top left
        vec2<f32>(-1.0, 1.0), // top left
        vec2<f32>(1.0, -1.0), // bottom right
        vec2<f32>(1.0, 1.0)  // top right
    );

    let pos = positions[vertex_index];
    var out: VertexOutput;
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = pos * 0.5 + vec2<f32>(0.5, 0.5);
    out.uv.y = 1.0 - out.uv.y; // Flip Y for texture coordinates
    return out;
}

struct FragInput {
    @location(0) uv: vec2<f32>,
};

@fragment
fn gaussian_blur(input: FragInput) -> @location(0) vec4<f32> {
    let uv = input.uv;
    let direction = uniforms.blur_direction;
    let texel_size = vec2<f32>(1.0) / vec2<f32>(textureDimensions(image));
    let radius = 2.0 * uniforms.blur_radius;
    var step = 1.0 + 0.05 * abs(radius);
    var color = vec4<f32>(0.0);
    var total_weight = 0.0;
    for (var x = -radius; x <= radius; x += round(step)) {
        let offset = direction * x * texel_size;
        let sample_uv = clamp(uv + offset, vec2(0.0), uniforms.content_scale);
        let dist = x * x;
        let current_step = round(step);
        let weight = exp(-dist / (2.0 * radius)) * current_step;
        total_weight += weight;
        step = 1.0 + 0.05 * abs(x);
        color += srgb_to_linear(textureSample(image, image_sampler, sample_uv)) * weight;
    }
    return linear_to_srgb(color / total_weight);
}

// ---------- sRGB <-> linear ----------
fn srgb_to_linear(c: vec4<f32>) -> vec4<f32> {
    let cutoff = vec4<f32>(0.04045);
    let low    = c / 12.92;
    let high   = pow((c + vec4<f32>(0.055)) / 1.055, vec4<f32>(2.4));
    return select(high, low, c <= cutoff);
}

fn linear_to_srgb(c: vec4<f32>) -> vec4<f32> {
    let cutoff = vec4<f32>(0.0031308);
    let low    = 12.92 * c;
    let high   = 1.055 * pow(c, vec4<f32>(1.0 / 2.4)) - vec4<f32>(0.055);
    return select(high, low, c <= cutoff);
}