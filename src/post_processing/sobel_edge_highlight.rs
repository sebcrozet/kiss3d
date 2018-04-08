//! A post-processing effect to highlight edges.

use gl::{self, types::*};
use na::Vector2;
use resource::{BufferType, AllocationType, Shader, ShaderUniform, ShaderAttribute, RenderTarget,
               GPUVec};
use post_processing::post_processing_effect::PostProcessingEffect;

#[path = "../error.rs"]
mod error;

/// Post processing effect which turns everything in grayscales.
pub struct SobelEdgeHighlight {
    shiftx:          f32,
    shifty:          f32,
    zn:              f32,
    zf:              f32,
    threshold:       f32,
    shader:          Shader,
    gl_nx:           ShaderUniform<GLfloat>,
    gl_ny:           ShaderUniform<GLfloat>,
    gl_fbo_depth:    ShaderUniform<GLint>,
    gl_fbo_texture:  ShaderUniform<GLint>,
    gl_znear:        ShaderUniform<GLfloat>,
    gl_zfar:         ShaderUniform<GLfloat>,
    gl_threshold:    ShaderUniform<GLfloat>,
    gl_v_coord:      ShaderAttribute<Vector2<f32>>,
    gl_fbo_vertices: GPUVec<Vector2<f32>>
}

impl SobelEdgeHighlight {
    /// Creates a new SobelEdgeHighlight post processing effect.
    pub fn new(threshold: f32) -> SobelEdgeHighlight {
        let fbo_vertices: Vec<Vector2<GLfloat>>  = vec!(
            Vector2::new(-1.0, -1.0),
            Vector2::new(1.0, -1.0),
            Vector2::new(-1.0,  1.0),
            Vector2::new(1.0,  1.0));

        let mut fbo_vertices = GPUVec::new(fbo_vertices, BufferType::Array, AllocationType::StaticDraw);
        fbo_vertices.load_to_gpu();
        fbo_vertices.unload_from_ram();

        let mut shader = Shader::new_from_str(VERTEX_SHADER, FRAGMENT_SHADER);

        shader.use_program();

        SobelEdgeHighlight {
            shiftx:          0.0,
            shifty:          0.0,
            zn:              0.0,
            zf:              0.0,
            threshold,
            gl_nx:           shader.get_uniform("nx").unwrap(),
            gl_ny:           shader.get_uniform("ny").unwrap(),
            gl_fbo_depth:    shader.get_uniform("fbo_depth").unwrap(),
            gl_fbo_texture:  shader.get_uniform("fbo_texture").unwrap(),
            gl_znear:        shader.get_uniform("znear").unwrap(),
            gl_zfar:         shader.get_uniform("zfar").unwrap(),
            gl_threshold:    shader.get_uniform("threshold").unwrap(),
            gl_v_coord:      shader.get_attrib("v_coord").unwrap(),
            gl_fbo_vertices: fbo_vertices,
            shader,
        }
    }
}

impl PostProcessingEffect for SobelEdgeHighlight {
    fn update(&mut self, _: f32, w: f32, h: f32, znear: f32, zfar: f32) {
        self.shiftx = 2.0 / w;
        self.shifty = 2.0 / h;
        self.zn     = znear;
        self.zf     = zfar;
    }

    fn draw(&mut self, target: &RenderTarget) {
        self.gl_v_coord.enable();

        /*
         * Finalize draw
         */
        verify!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));

        self.shader.use_program();

        self.gl_threshold.upload(&self.threshold);
        self.gl_nx.upload(&self.shiftx);
        self.gl_ny.upload(&self.shifty);
        self.gl_znear.upload(&self.zn);
        self.gl_zfar.upload(&self.zf);

        verify!(gl::ActiveTexture(gl::TEXTURE0));
        verify!(gl::BindTexture(gl::TEXTURE_2D, target.texture_id()));

        self.gl_fbo_texture.upload(&0);

        verify!(gl::ActiveTexture(gl::TEXTURE1));
        verify!(gl::BindTexture(gl::TEXTURE_2D, target.depth_id()));

        self.gl_fbo_depth.upload(&1);


        self.gl_v_coord.bind(&mut self.gl_fbo_vertices);

        verify!(gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4));

        self.gl_v_coord.disable();
    }
}

static VERTEX_SHADER: &'static str =
    "#version 120
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

static FRAGMENT_SHADER: &'static str =
    "#version 120
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

        float KX[9] = float[](1, 0, -1,
                              2, 0, -2,
                              1, 0, -1);

        float gx = 0;

        for (int i = -1; i < 2; ++i) {
            for (int j = -1; j < 2; ++j) {
                int off = (i + 1) * 3 + j + 1;
                gx += KX[off] * lin_depth(vec2(f_texcoord.x + i * nx, f_texcoord.y + j * ny));
            }
        }

        float KY[9] = float[](1,  2,  1,
                              0,  0,  0,
                              -1, -2, -1);

        float gy = 0;

        for (int i = -1; i < 2; ++i) {
            for (int j = -1; j < 2; ++j) {
                int off = (i + 1) * 3 + j + 1;
                gy += KY[off] * lin_depth(vec2(f_texcoord.x + i * nx, f_texcoord.y + j * ny));
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

