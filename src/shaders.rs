pub static VERTEX_SRC: &'static str =
   "#version 120                                                           \n\
    attribute vec3 position;                                               \n\
    attribute vec3 normal;                                                 \n\
    attribute vec3 color;                                                  \n\
    attribute vec2 tex_coord_v;                                            \n\
    varying vec3 Color;                                                    \n\
    varying vec3 ws_normal;                                                \n\
    varying vec3 ws_position;                                              \n\
    varying vec2 tex_coord;                                                \n\
    uniform mat4 projection;                                               \n\
    uniform mat4 view;                                                     \n\
    uniform mat4 transform;                                                \n\
    uniform mat3 scale;                                                    \n\
    uniform mat3 ntransform;                                               \n\
    void main() {                                                          \n\
        Color       = color;                                               \n\
        mat4 scale4 = mat4(scale);                                         \n\
        vec4 pos4   = transform * scale4 * vec4(position, 1.0);            \n\
        tex_coord   = tex_coord_v;                                         \n\
        ws_position = pos4.xyz;                                            \n\
        gl_Position = projection * view * transform * scale4 * vec4(position, 1.0); \n\
        ws_normal   = normalize(ntransform * scale * normal);                       \n\
    }";

// phong lighting (heavily) inspired
// by http://www.opengl.org/sdk/docs/tutorials/ClockworkCoders/lighting.php
pub static FRAGMENT_SRC: &'static str =
   "#version 120                     \n\
    uniform vec3      color;         \n\
    uniform vec3      light_position;\n\
    uniform sampler2D tex;           \n\
    varying vec2      tex_coord;     \n\
    varying vec3      ws_normal;     \n\
    varying vec3      ws_position;   \n\
    varying vec4      outColor;      \n\
    void main() {                    \n\
      vec3 L = normalize(light_position - ws_position);   \n\
      vec3 E = normalize(-ws_position);                   \n\
                                                          \n\
      //calculate Ambient Term:                           \n\
      vec4 Iamb = vec4(1.0, 1.0, 1.0, 1.0);               \n\
                                                          \n\
      //calculate Diffuse Term:                           \n\
      vec4 Idiff1 = vec4(1.0, 1.0, 1.0, 1.0) * max(dot(ws_normal,L), 0.0);  \n\
      Idiff1 = clamp(Idiff1, 0.0, 1.0);                                     \n\
                                                                            \n\
      // double sided lighting:                                             \n\
      vec4 Idiff2 = vec4(1.0, 1.0, 1.0, 1.0) * max(dot(-ws_normal,L), 0.0); \n\
      Idiff2 = clamp(Idiff2, 0.0, 1.0);                                     \n\
                                                                            \n\
      vec4 tex_color = texture2D(tex, tex_coord);                           \n\
      gl_FragColor   = tex_color * (vec4(color, 1.0) + Iamb + (Idiff1 + Idiff2) / 2) / 3; \n\
    }";
