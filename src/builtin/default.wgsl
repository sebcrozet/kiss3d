// Uniforms
struct Uniforms {
    proj: mat4x4<f32>,
    view: mat4x4<f32>,
    transform: mat4x4<f32>,
    ntransform: mat3x3<f32>,
    scale: mat3x3<f32>,
    light_position: vec3<f32>,
    color: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var tex_sampler: sampler;

@group(0) @binding(2)
var tex: texture_2d<f32>;

// Vertex input
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) inst_tra: vec3<f32>,
    @location(4) inst_color: vec4<f32>,
    @location(5) inst_def_0: vec3<f32>,
    @location(6) inst_def_1: vec3<f32>,
    @location(7) inst_def_2: vec3<f32>,
}

// Vertex output / Fragment input
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_light_position: vec3<f32>,
    @location(1) tex_coord_v: vec2<f32>,
    @location(2) normal_interp: vec3<f32>,
    @location(3) vert_pos: vec3<f32>,
    @location(4) vert_color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Build deformation matrix
    let deformation = mat3x3<f32>(
        input.inst_def_0,
        input.inst_def_1,
        input.inst_def_2
    );

    // Calculate position
    let scaled_pos = uniforms.scale * input.position;
    let deformed_pos = deformation * scaled_pos;
    let transformed = uniforms.transform * vec4<f32>(deformed_pos, 1.0);
    let translated = transformed + vec4<f32>(input.inst_tra, 0.0);

    output.position = uniforms.proj * uniforms.view * translated;

    // Calculate view space position
    let vert_pos4 = uniforms.view * translated;
    output.vert_pos = vert_pos4.xyz / vert_pos4.w;

    // Transform normal to view space
    output.normal_interp = mat3x3<f32>(
        uniforms.view[0].xyz,
        uniforms.view[1].xyz,
        uniforms.view[2].xyz
    ) * uniforms.ntransform * input.normal;

    output.tex_coord_v = input.tex_coord;

    // Transform light position to view space
    output.local_light_position = (uniforms.view * vec4<f32>(uniforms.light_position, 1.0)).xyz;

    output.vert_color = input.inst_color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let spec_color = vec3<f32>(0.4, 0.4, 0.4);

    let normal = normalize(input.normal_interp);
    let light_dir = normalize(input.local_light_position - input.vert_pos);

    var lambertian = max(dot(light_dir, normal), 0.0);
    var specular = 0.0;

    if (lambertian > 0.0) {
        let view_dir = normalize(-input.vert_pos);
        let half_dir = normalize(light_dir + view_dir);
        let spec_angle = max(dot(half_dir, normal), 0.0);
        specular = pow(spec_angle, 30.0);
    }

    let base_color = input.vert_color * vec4<f32>(uniforms.color, 1.0);
    let tex_color = textureSample(tex, tex_sampler, input.tex_coord_v);

    let final_color = tex_color * vec4<f32>(
        base_color.xyz / 3.0 +
        lambertian * base_color.xyz / 3.0 +
        specular * spec_color / 3.0,
        base_color.w
    );

    return final_color;
}
