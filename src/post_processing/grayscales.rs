//! Post-processing effect to draw everything in grey-levels.

use na::Vector2;

use context::Context;
use post_processing::PostProcessingEffect;
use resource::{
    AllocationType, BufferType, Effect, GPUVec, RenderTarget, ShaderAttribute, ShaderUniform,
};

#[path = "../error.rs"]
mod error;

/// Post processing effect which turns everything in gray scales.
pub struct Grayscales {
    shader: Effect,
    fbo_texture: ShaderUniform<i32>,
    v_coord: ShaderAttribute<Vector2<f32>>,
    fbo_vertices: GPUVec<Vector2<f32>>,
}

impl Grayscales {
    /// Creates a new `Grayscales` post processing effect.
    pub fn new() -> Grayscales {
        let fbo_vertices: Vec<Vector2<f32>> = vec![
            Vector2::new(-1.0, -1.0),
            Vector2::new(1.0, -1.0),
            Vector2::new(-1.0, 1.0),
            Vector2::new(1.0, 1.0),
        ];

        let mut fbo_vertices =
            GPUVec::new(fbo_vertices, BufferType::Array, AllocationType::StaticDraw);
        fbo_vertices.load_to_gpu();
        fbo_vertices.unload_from_ram();

        let mut shader = Effect::new_from_str(VERTEX_SHADER, FRAGMENT_SHADER);

        shader.use_program();

        Grayscales {
            fbo_texture: shader.get_uniform("fbo_texture").unwrap(),
            v_coord: shader.get_attrib("v_coord").unwrap(),
            fbo_vertices: fbo_vertices,
            shader: shader,
        }
    }
}

impl PostProcessingEffect for Grayscales {
    fn update(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32) {}

    fn draw(&mut self, target: &RenderTarget) {
        let ctxt = Context::get();
        self.v_coord.enable();

        /*
         * Finalize draw
         */
        self.shader.use_program();
        verify!(ctxt.clear_color(0.0, 0.0, 0.0, 1.0));
        verify!(ctxt.clear(Context::COLOR_BUFFER_BIT | Context::DEPTH_BUFFER_BIT));
        verify!(ctxt.bind_texture(Context::TEXTURE_2D, target.texture_id()));

        self.fbo_texture.upload(&0);
        self.v_coord.bind(&mut self.fbo_vertices);

        verify!(ctxt.draw_arrays(Context::TRIANGLE_STRIP, 0, 4));

        self.v_coord.disable();
    }
}

static VERTEX_SHADER: &'static str = "#version 100
    attribute vec2    v_coord;
    uniform sampler2D fbo_texture;
    varying vec2      f_texcoord;

    void main(void) {
      gl_Position = vec4(v_coord, 0.0, 1.0);
      f_texcoord  = (v_coord + 1.0) / 2.0;
    }";

static FRAGMENT_SHADER: &'static str = "#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif

    uniform sampler2D fbo_texture;
    varying vec2      f_texcoord;

    void main(void) {
      vec2 texcoord = f_texcoord;
      vec4 color    = texture2D(fbo_texture, texcoord);
      float gray    =  0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b;
      gl_FragColor  = vec4(gray, gray, gray, color.a);
    }";
