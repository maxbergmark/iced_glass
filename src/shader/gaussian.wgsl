struct Uniforms {
    blur_radius: f32,
    corner_radius: f32,
    saturation: f32,
    lightness: f32,
    direction: vec2<f32>,
};

struct BlurDirection {
    direction: vec2<f32>,
}


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
    out.uv = pos * 0.5 + vec2<f32>(0.5, 0.5); // Map from [-1,1] to [0,1]
    out.uv.y = 1.0 - out.uv.y; // Flip Y for texture coordinates
    return out;
}

struct FragInput {
    @location(0) uv: vec2<f32>,
};

const PI: f32 = 3.14159265358979323846;
const RADIUS: f32 = 0.10;

@fragment
fn horizontal_pass(input: FragInput) -> @location(0) vec4<f32> {
    let uv = input.uv;
    let direction = uniforms.direction;
    let texel_size = vec2<f32>(1.0) / vec2<f32>(textureDimensions(image));
    let radius = uniforms.blur_radius;
    var step = 1.0 + 0.05 * abs(radius);
    var color = vec4<f32>(0.0);
    var total_weight = 0.0;
    for (var x = -radius; x <= radius; x += step) {
        let offset = direction * x * texel_size;
        let dist = x * x;
        let weight = exp(-dist / (2.0 * radius));
        color += textureSample(image, image_sampler, uv + offset) * weight;
        total_weight += weight;
        step = 1.0 + 0.05 * abs(x);
    }
    return color / total_weight;
}

// @fragment
// fn vertical_pass(input: FragInput) -> @location(0) vec4<f32> {
//     let uv = input.uv;
//     let texel_size = vec2<f32>(1.0) / vec2<f32>(textureDimensions(image));
//     let radius = i32(uniforms.blur_radius);
//     var step = 1 + abs(radius) / 100;
//     var color = vec4<f32>(0.0);
//     var total_weight = 0.0;
//     for (var y = -radius; y <= radius; y += step) {
//         let offset = vec2<f32>(0.0, f32(y)) * texel_size;
//         let dist = f32(y * y);
//         let weight = exp(-dist / (2.0 * f32(radius)));
//         color += textureSample(image, image_sampler, uv + offset) * weight;
//         total_weight += weight;
//         step = 1 + abs(y) / 100;
//     }
//     return color / total_weight;
// }