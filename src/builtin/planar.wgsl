// Planar (2D) rendering shader

struct Uniforms {
    proj: mat3x3<f32>,
    view: mat3x3<f32>,
    model: mat3x3<f32>,
    scale: mat2x2<f32>,
    color: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var tex_sampler: sampler;

@group(0) @binding(2)
var tex: texture_2d<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) inst_tra: vec2<f32>,
    @location(3) inst_color: vec4<f32>,
    @location(4) inst_deformation: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) vert_color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Build deformation matrix from vec4
    let def = mat2x2<f32>(
        vec2<f32>(input.inst_deformation[0], input.inst_deformation[1]),
        vec2<f32>(input.inst_deformation[2], input.inst_deformation[3])
    );

    let scaled = def * uniforms.scale * input.position;
    let transformed = uniforms.model * vec3<f32>(scaled, 1.0);
    let translated = vec3<f32>(input.inst_tra, 0.0) + transformed;
    var projected = uniforms.proj * uniforms.view * translated;
    projected.z = 0.0;

    output.position = vec4<f32>(projected, 1.0);
    output.tex_coord = input.tex_coord;
    output.vert_color = input.inst_color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(tex, tex_sampler, input.tex_coord);
    return tex_color * (vec4<f32>(uniforms.color, 1.0) * input.vert_color);
}

