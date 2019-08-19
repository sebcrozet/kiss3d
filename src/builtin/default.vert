#version 100
attribute vec3 position;
attribute vec2 tex_coord;
attribute vec3 normal;

uniform mat3 ntransform, scale;
uniform mat4 proj, view, transform;
uniform vec3 light_position;

varying vec3 local_light_position;
varying vec2 tex_coord_v;
varying vec3 normalInterp;
varying vec3 vertPos;

void main(){
    gl_Position = proj * view * transform * vec4(scale * position, 1.0);
    vec4 vertPos4 = view * transform * vec4(scale * position, 1.0);
    vertPos = vec3(vertPos4) / vertPos4.w;
    normalInterp = mat3(view) * ntransform * normal;
    tex_coord_v = tex_coord;
    local_light_position = (view * vec4(light_position, 1.0)).xyz;
}
