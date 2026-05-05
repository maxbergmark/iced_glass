@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;

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

@fragment
fn fs(input: FragInput) -> @location(0) vec4<f32> {
    // return textureSample(src, src_sampler, input.uv);
    let dims = vec2<f32>(textureDimensions(src));
    let max_idx = vec2<i32>(dims) - vec2(1);
    let coord = input.uv * dims - 0.5;
    let base = vec2<i32>(floor(coord));
    let f = fract(coord);
    let i00 = clamp(base,                  vec2(0), max_idx);
    let i10 = clamp(base + vec2(1, 0),     vec2(0), max_idx);
    let i01 = clamp(base + vec2(0, 1),     vec2(0), max_idx);
    let i11 = clamp(base + vec2(1, 1),     vec2(0), max_idx);
    let c00 = srgb_to_linear(textureLoad(src, i00, 0));
    let c10 = srgb_to_linear(textureLoad(src, i10, 0));
    let c01 = srgb_to_linear(textureLoad(src, i01, 0));
    let c11 = srgb_to_linear(textureLoad(src, i11, 0));
    let top = mix(c00, c10, f.x);
    let bot = mix(c01, c11, f.x);
    return linear_to_srgb(mix(top, bot, f.y));
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