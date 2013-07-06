pub static VERTEX_SRC: &'static str =
   "#version 130                                                           \n\
    attribute vec3 position;                                                      \n\
    attribute vec3 normal;                                                        \n\
    attribute vec3 color;                                                         \n\
    varying vec3 Color;                                                        \n\
    varying vec3 ws_normal;                                                    \n\
    varying vec3 ws_position;                                                  \n\
    uniform mat4 projection;                                               \n\
    uniform mat4 view;                                                     \n\
    uniform mat4 transform;                                                \n\
    uniform mat3 scale;                                                    \n\
    uniform mat3 ntransform;                                               \n\
    void main() {                                                          \n\
        Color       = color;                                               \n\
        mat4 scale4 = mat4(scale);                                         \n\
        vec4 pos4   = transform * scale4 * vec4(position, 1.0);            \n\
        ws_position = pos4.xyz;                                            \n\
        gl_Position = projection * view * transform * scale4 * vec4(position, 1.0); \n\
        ws_normal   = normalize(ntransform * scale * normal);              \n\
    }";

// phong lighting (heavily) inspired
// by http://www.opengl.org/sdk/docs/tutorials/ClockworkCoders/lighting.php
pub static FRAGMENT_SRC: &'static str =
   "#version 130                     \n\
    uniform vec3 color;              \n\
    uniform vec3 light_position;     \n\
    in  vec3 ws_normal;              \n\
    in  vec3 ws_position;            \n\
    out vec4 outColor;               \n\
    void main() {                    \n\
      vec3 L = normalize(light_position - ws_position);   \n\
      vec3 E = normalize(-ws_position);                   \n\
      vec3 R = normalize(-reflect(L, ws_normal));         \n\
                                                          \n\
      //calculate Ambient Term:                           \n\
      vec4 Iamb = vec4(1.0, 1.0, 1.0, 1.0);               \n\
                                                          \n\
      //calculate Diffuse Term:                           \n\
      vec4 Idiff = vec4(1.0, 1.0, 1.0, 1.0) * max(dot(ws_normal,L), 0.0); \n\
      Idiff = clamp(Idiff, 0.0, 1.0);                                     \n\
                                                                          \n\
      // calculate Specular Term:                                         \n\
      // vec4 Ispec = vec4(0.6, 0.6, 0.6, 1.0)                            \n\
      //              * pow(max(dot(R, E), 0.0), 35.0);                   \n\
      // Ispec = clamp(Ispec, 0.0, 1.0);                                  \n\
                                                                          \n\
      outColor = (vec4(color, 1.0) + Iamb + Idiff) / 3;                   \n\
    }";
