//! A post-processing effect to highlight edges.

use na::Vector2;

use context::Context;
use post_processing::post_processing_effect::PostProcessingEffect;
use resource::{
    AllocationType, BufferType, Effect, GPUVec, RenderTarget, ShaderAttribute, ShaderUniform,
};

#[path = "../error.rs"]
mod error;

/// Post processing effect which turns everything in grayscales.
pub struct SobelEdgeHighlight {
    shiftx: f32,
    shifty: f32,
    zn: f32,
    zf: f32,
    threshold: f32,
    shader: Effect,
    gl_nx: ShaderUniform<f32>,
    gl_ny: ShaderUniform<f32>,
    gl_fbo_depth: ShaderUniform<i32>,
    gl_fbo_texture: ShaderUniform<i32>,
    gl_znear: ShaderUniform<f32>,
    gl_zfar: ShaderUniform<f32>,
    gl_threshold: ShaderUniform<f32>,
    gl_v_coord: ShaderAttribute<Vector2<f32>>,
    gl_fbo_vertices: GPUVec<Vector2<f32>>,
}

impl SobelEdgeHighlight {
    /// Creates a new SobelEdgeHighlight post processing effect.
    pub fn new(threshold: f32) -> SobelEdgeHighlight {
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

        SobelEdgeHighlight {
            shiftx: 0.0,
            shifty: 0.0,
            zn: 0.0,
            zf: 0.0,
            threshold: threshold,
            gl_nx: shader.get_uniform("nx").unwrap(),
            gl_ny: shader.get_uniform("ny").unwrap(),
            gl_fbo_depth: shader.get_uniform("fbo_depth").unwrap(),
            gl_fbo_texture: shader.get_uniform("fbo_texture").unwrap(),
            gl_znear: shader.get_uniform("znear").unwrap(),
            gl_zfar: shader.get_uniform("zfar").unwrap(),
            gl_threshold: shader.get_uniform("threshold").unwrap(),
            gl_v_coord: shader.get_attrib("v_coord").unwrap(),
            gl_fbo_vertices: fbo_vertices,
            shader: shader,
        }
    }
}

impl PostProcessingEffect for SobelEdgeHighlight {
    fn update(&mut self, _: f32, w: f32, h: f32, znear: f32, zfar: f32) {
        self.shiftx = 2.0 / w;
        self.shifty = 2.0 / h;
        self.zn = znear;
        self.zf = zfar;
    }

    fn draw(&mut self, target: &RenderTarget) {
        let ctxt = Context::get();
        self.gl_v_coord.enable();

        /*
         * Finalize draw
         */
        verify!(ctxt.clear(Context::COLOR_BUFFER_BIT | Context::DEPTH_BUFFER_BIT));

        self.shader.use_program();

        self.gl_threshold.upload(&self.threshold);
        self.gl_nx.upload(&self.shiftx);
        self.gl_ny.upload(&self.shifty);
        self.gl_znear.upload(&self.zn);
        self.gl_zfar.upload(&self.zf);

        verify!(ctxt.active_texture(Context::TEXTURE0));
        verify!(ctxt.bind_texture(Context::TEXTURE_2D, target.texture_id()));

        self.gl_fbo_texture.upload(&0);

        verify!(ctxt.active_texture(Context::TEXTURE1));
        verify!(ctxt.bind_texture(Context::TEXTURE_2D, target.depth_id()));

        self.gl_fbo_depth.upload(&1);

        self.gl_v_coord.bind(&mut self.gl_fbo_vertices);

        verify!(ctxt.draw_arrays(Context::TRIANGLE_STRIP, 0, 4));

        self.gl_v_coord.disable();
    }
}

static VERTEX_SHADER: &'static str = "#version 100
    attribute vec2    v_coord;
    uniform sampler2D fbo_depth;
    uniform sampler2D fbo_texture;
    uniform float     nx;
    uniform float     ny;
    uniform float     znear;
    uniform float     zfar;
    uniform float     threshold;
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

    uniform sampler2D fbo_depth;
    uniform sampler2D fbo_texture;
    uniform float     nx;
    uniform float     ny;
    uniform float     znear;
    uniform float     zfar;
    uniform float     threshold;
    varying vec2      f_texcoord;

    float lin_depth(vec2 uv) {
        float nlin_depth = texture2D(fbo_depth, uv).x;

        return znear * zfar / ((nlin_depth * (zfar - znear)) - zfar);
    }

    void main(void) {
        vec2 texcoord  = f_texcoord;

        float KX[9];
        KX[0] = 1.0; KX[1] = 0.0; KX[2] = -1.0;
        KX[3] = 2.0; KX[4] = 0.0; KX[5] = -2.0;
        KX[6] = 1.0; KX[7] = 0.0; KX[8] = -1.0;

        float gx = 0.0;

        for (int i = -1; i < 2; ++i) {
            for (int j = -1; j < 2; ++j) {
                int off = (i + 1) * 3 + j + 1;
                gx += KX[off] * lin_depth(vec2(f_texcoord.x + float(i) * nx, f_texcoord.y + float(j) * ny));
            }
        }

        float KY[9];
        KY[0] = 1.0;  KY[1] = 2.0;  KY[2] = 1.0;
        KY[3] = 0.0;  KY[4] = 0.0;  KY[5] = 0.0;
        KY[6] = -1.0; KY[7] = -2.0; KY[8] = -1.0;

        float gy = 0.0;

        for (int i = -1; i < 2; ++i) {
            for (int j = -1; j < 2; ++j) {
                int off = (i + 1) * 3 + j + 1;
                gy += KY[off] * lin_depth(vec2(f_texcoord.x + float(i) * nx, f_texcoord.y + float(j) * ny));
            }
        }

        float gradient = sqrt(gx * gx + gy * gy);

        float edge;

        if (gradient > threshold) {
            edge = 0.0;
        }
        else {
            edge = 1.0 - gradient / threshold;
        }

        vec4 color = texture2D(fbo_texture, texcoord);

        gl_FragColor = vec4(edge * color.xyz, 1.0);
    }";
