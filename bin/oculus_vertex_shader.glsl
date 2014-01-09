#version 120
attribute vec2    v_coord;
uniform sampler2D fbo_texture;
varying vec2      f_texcoord;
 
void main(void) {
  gl_Position = vec4(v_coord, 0.0, 1.0);
  f_texcoord  = (v_coord + 1.0) / 2.0;
}
