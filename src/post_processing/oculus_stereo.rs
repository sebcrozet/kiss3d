//! Post processing effect to support the Oculus Rift.

use gl;
use gl::types::*;
use na::Vec2;
use resource::{Shader, ShaderUniform, ShaderAttribute, RenderTarget, GPUVector, ArrayBuffer, StaticDraw};
use post_processing::post_processing_effect::PostProcessingEffect;

#[path = "../error.rs"]
mod error;

/// An post-processing effect to support the oculus rift.
pub struct OculusStereo {
    shader:       Shader,
    fbo_vertices: GPUVector<Vec2<f32>>,
    fbo_texture:  ShaderUniform<GLint>,
    v_coord:      ShaderAttribute<Vec2<GLfloat>>,
    kappa_0:      ShaderUniform<GLfloat>,
    kappa_1:      ShaderUniform<GLfloat>,
    kappa_2:      ShaderUniform<GLfloat>,
    kappa_3:      ShaderUniform<GLfloat>,
    scale:        ShaderUniform<Vec2<GLfloat>>,
    scale_in:     ShaderUniform<Vec2<GLfloat>>,
    w:            f32,
    h:            f32
}

impl OculusStereo {
    /// Creates a new OculusStereo post processing effect.
    pub fn new() -> OculusStereo {
        let fbo_vertices: Vec<Vec2<GLfloat>>  = vec!(
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(-1.0,  1.0),
            Vec2::new(1.0,  1.0));

        let mut fbo_vertices = GPUVector::new(fbo_vertices, ArrayBuffer, StaticDraw);
        fbo_vertices.load_to_gpu();
        fbo_vertices.unload_from_ram();

        let mut shader = Shader::new_from_str(VERTEX_SHADER, FRAGMENT_SHADER);

        shader.use_program();

        OculusStereo {
            fbo_texture:  shader.get_uniform("fbo_texture").unwrap(),
            fbo_vertices: fbo_vertices,
            v_coord:      shader.get_attrib("v_coord").unwrap(),
            kappa_0:      shader.get_uniform("kappa_0").unwrap(),
            kappa_1:      shader.get_uniform("kappa_1").unwrap(),
            kappa_2:      shader.get_uniform("kappa_2").unwrap(),
            kappa_3:      shader.get_uniform("kappa_3").unwrap(),
            scale:        shader.get_uniform("Scale").unwrap(),
            scale_in:     shader.get_uniform("ScaleIn").unwrap(),
            shader:       shader,
            h:            1f32, // will be updated in the first update
            w:            1f32, // ditto
        }
    }
}

impl PostProcessingEffect for OculusStereo {
    fn update(&mut self, _: f32, w: f32, h: f32, _: f32, _: f32) {
        self.w = w;
        self.h = h;
    }

    fn draw(&mut self, target: &RenderTarget) {
        let scale_factor = 0.9f32; // firebox: in Oculus SDK example it's "1.0f/Distortion.Scale"
        let aspect       = (self.w / 2.0f32) / (self.h); // firebox: rift's "half screen aspect ratio"

        self.shader.use_program();

        self.v_coord.enable();

        /*
         * Configure the post-process effect.
         */
        let kappa = [1.0, 1.7, 0.7, 15.0];
        self.kappa_0.upload(&kappa[0]);
        self.kappa_1.upload(&kappa[1]);
        self.kappa_2.upload(&kappa[2]);
        self.kappa_3.upload(&kappa[3]);
        self.scale.upload(&Vec2::new(0.5f32, aspect));
        self.scale_in.upload(&Vec2::new(2.0f32 * scale_factor, 1.0f32 / aspect * scale_factor));

        /*
         * Finalize draw
         */
        verify!(gl::ClearColor(0.0, 0.0, 0.0, 1.0));
        verify!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));

        verify!(gl::BindTexture(gl::TEXTURE_2D, target.texture_id()));

        self.fbo_texture.upload(&0);

        self.v_coord.bind(&mut self.fbo_vertices);

        verify!(gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4));

        self.v_coord.disable();
    }
}

static VERTEX_SHADER: &'static str =
"
#version 120
attribute vec2    v_coord;
uniform sampler2D fbo_texture;
varying vec2      f_texcoord;
 
void main(void) {
  gl_Position = vec4(v_coord, 0.0, 1.0);
  f_texcoord  = (v_coord + 1.0) / 2.0;
}
";

static FRAGMENT_SHADER: &'static str =
"
#version 120
uniform sampler2D fbo_texture;
uniform float kappa_0;
uniform float kappa_1;
uniform float kappa_2;
uniform float kappa_3;
const vec2 LensCenterLeft = vec2(0.25, 0.5);
const vec2 LensCenterRight = vec2(0.75, 0.5);
uniform vec2 Scale;
uniform vec2 ScaleIn;

varying vec2 v_coord;
varying vec2 f_texcoord;

void main()
{
    vec2 theta;
    float rSq;
    vec2 rvector;
    vec2 tc;
    bool left_eye;

    if (f_texcoord.x < 0.5) {
        left_eye = true;
    } else {
        left_eye = false;
    }

    if (left_eye) {
        theta = (f_texcoord - LensCenterLeft) * ScaleIn; 
    } else {
        theta = (f_texcoord - LensCenterRight) * ScaleIn; 
    }
    rSq = theta.x * theta.x + theta.y * theta.y;
    rvector = theta * (kappa_0 + kappa_1 * rSq + kappa_2 * rSq * rSq + kappa_3 * rSq * rSq * rSq);
    if (left_eye) {
        tc = LensCenterLeft + Scale * rvector;
    } else {
        tc = LensCenterRight + Scale * rvector;
    }

    //keep within bounds of texture 
    if ((left_eye && (tc.x < 0.0 || tc.x > 0.5)) ||   
        (!left_eye && (tc.x < 0.5 || tc.x > 1.0)) ||
        tc.y < 0.0 || tc.y > 1.0) {
        discard;
    }

    gl_FragColor = texture2D(fbo_texture, tc); 
}
";
