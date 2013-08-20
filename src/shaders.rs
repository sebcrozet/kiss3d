pub static OBJECT_VERTEX_SRC: &'static str =
   "#version 120
    attribute vec3 position;
    attribute vec3 normal;
    attribute vec3 color;
    attribute vec2 tex_coord_v;
    varying vec3 ws_normal;
    varying vec3 ws_position;
    varying vec2 tex_coord;
    uniform mat4 projection;
    uniform mat4 view;
    uniform mat4 transform;
    uniform mat3 scale;
    uniform mat3 ntransform;
    void main() {
        mat4 scale4 = mat4(scale);
        vec4 pos4   = transform * scale4 * vec4(position, 1.0);
        tex_coord   = tex_coord_v;
        ws_position = pos4.xyz;
        gl_Position = projection * view * transform * scale4 * vec4(position, 1.0);
        ws_normal   = normalize(ntransform * scale * normal);
    }";

// phong lighting (heavily) inspired
// by http://www.opengl.org/sdk/docs/tutorials/ClockworkCoders/lighting.php
pub static OBJECT_FRAGMENT_SRC: &'static str =
   "#version 120
    uniform vec3      color;
    uniform vec3      light_position;
    uniform sampler2D tex;
    varying vec2      tex_coord;
    varying vec3      ws_normal;
    varying vec3      ws_position;
    void main() {
      vec3 L = normalize(light_position - ws_position);
      vec3 E = normalize(-ws_position);

      //calculate Ambient Term:
      vec4 Iamb = vec4(1.0, 1.0, 1.0, 1.0);

      //calculate Diffuse Term:
      vec4 Idiff1 = vec4(1.0, 1.0, 1.0, 1.0) * max(dot(ws_normal,L), 0.0);
      Idiff1 = clamp(Idiff1, 0.0, 1.0);

      // double sided lighting:
      vec4 Idiff2 = vec4(1.0, 1.0, 1.0, 1.0) * max(dot(-ws_normal,L), 0.0);
      Idiff2 = clamp(Idiff2, 0.0, 1.0);

      vec4 tex_color = texture2D(tex, tex_coord);
      gl_FragColor   = tex_color * (vec4(color, 1.0) + Iamb + (Idiff1 + Idiff2) / 2) / 3;
    }";

pub static LINES_VERTEX_SRC: &'static str =
   "#version 120
    attribute vec3 position;
    attribute vec3 color;
    varying   vec3 Color;
    uniform   mat4   projection;
    uniform   mat4   view;
    void main() {
        gl_Position = projection * view * vec4(position, 1.0);
        Color = color;
    }";

// phong lighting (heavily) inspired
// by http://www.opengl.org/sdk/docs/tutorials/ClockworkCoders/lighting.php
pub static LINES_FRAGMENT_SRC: &'static str =
   "#version 120
    varying vec3 Color;
    void main() {
      gl_FragColor = vec4(Color, 1.0);
    }";
