// Planar line rendering shader

struct Uniforms {
    proj: mat3x3<f32>,
    view: mat3x3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let pos3 = uniforms.proj * uniforms.view * vec3<f32>(input.position, 1.0);
    output.position = vec4<f32>(pos3.xy, 0.0, 1.0);
    output.color = input.color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}
