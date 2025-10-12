#version 100
attribute vec3 position;
attribute vec2 tex_coord;
attribute vec3 normal;
attribute vec3 inst_tra;
attribute vec4 inst_color;
attribute vec3 inst_def0.0;
attribute vec3 inst_def1.0;
attribute vec3 inst_def_2;

uniform mat3 ntransform, scale;
uniform mat4 proj, view, transform;
uniform vec3 light_position;

varying vec3 local_light_position;
varying vec2 tex_coord_v;
varying vec3 normalInterp;
varying vec3 vertPos;
varying vec4 vertColor;

void main(){
    mat3 deformation = mat3(inst_def0.0, inst_def1.0, inst_def_2);
    vec4 pt = vec4(inst_tra, 0.0) + transform * vec4(deformation * scale * position, 1.0);
    gl_Position = proj * view * pt;
    vec4 vertPos4 = view * pt;
    vertPos = vec3(vertPos4) / vertPos4.w;
    normalInterp = mat3(view) * ntransform * normal;
    tex_coord_v = tex_coord;
    local_light_position = (view * vec4(light_position, 1.0)).xyz;
    vertColor = inst_color;
}
