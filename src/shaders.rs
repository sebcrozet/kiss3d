pub static vertex_src: &'static str =
   "#version 150                                                    \n\
    in vec3 position;                                               \n\
    in vec3 normal;                                                 \n\
    in vec3 color;                                                  \n\
    out vec3 Color;                                                 \n\
    out vec3 ws_normal;                                             \n\
    out vec3 ws_position;                                           \n\
    uniform mat4 projection;                                        \n\
    uniform mat4 transform;                                         \n\
    uniform mat3 ntransform;                                        \n\
    void main() {                                                   \n\
        Color       = color;                                        \n\
        vec4 pos4   = transform * vec4(position, 1.0);              \n\
        ws_position = pos4.xyz;                                     \n\
        gl_Position = projection * transform * vec4(position, 1.0); \n\
        ws_normal   = normalize(ntransform * normal);               \n\
    }";

// phong lighting (heavily) inspired
// by http://www.opengl.org/sdk/docs/tutorials/ClockworkCoders/lighting.php
pub static fragment_src: &'static str =
   "#version 150                     \n\
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
      vec4 Iamb = vec4(0.1, 0.1, 0.1, 1.0);               \n\
                                                          \n\
      //calculate Diffuse Term:                           \n\
      vec4 Idiff = vec4(0.5, 0.5, 0.5, 1.0) * max(dot(ws_normal,L), 0.0); \n\
      Idiff = clamp(Idiff, 0.0, 1.0);                                     \n\
                                                                          \n\
      // calculate Specular Term:                                         \n\
      vec4 Ispec = vec4(0.3, 0.3, 0.3, 1.0)                               \n\
                   * pow(max(dot(R, E), 0.0), 0.3 * 10.0);                \n\
      Ispec = clamp(Ispec, 0.0, 1.0);                                     \n\
                                                                          \n\
      outColor = vec4(color, 1.0) + Iamb + Idiff + Ispec;                 \n\
    }";
