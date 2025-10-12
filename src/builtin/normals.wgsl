// Normals visualization shader

struct Uniforms {
    proj: mat4x4<f32>,
    view: mat4x4<f32>,
    transform: mat4x4<f32>,
    scale: mat3x3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let scaled = uniforms.scale * input.position;
    output.position = uniforms.proj * uniforms.view * uniforms.transform * vec4<f32>(scaled, 1.0);
    output.normal = input.normal;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>((input.normal + 1.0) / 2.0, 1.0);
}
