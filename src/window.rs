/*
 * FIXME: this file is too big. Some heavy refactoring need to be done here.
 */

use glfw;
use std::ptr;
use std::rt::io::timer::Timer;
use std::rt::rtio::RtioTimer;
use std::num::{Zero, One};
use std::libc;
use std::sys;
use std::cast;
use std::hashmap::HashMap;
use extra::time;
use gl;
use gl::types::*;
use stb_image::image::*;
use nalgebra::traits::homogeneous::{ToHomogeneous, FromHomogeneous};
use nalgebra::traits::inv::Inv;
use nalgebra::traits::vec_cast::VecCast;
use nalgebra::traits::mat_cast::MatCast;
use nalgebra::traits::translation::Translation;
use nalgebra::traits::transpose::Transpose;
use nalgebra::traits::rlmul::RMul;
use nalgebra::traits::vector::AlgebraicVec;
use nalgebra::mat::Mat4;
use nalgebra::vec::{Vec2, Vec3, Vec4};
use camera::{Camera, ArcBall};
use object::{GeometryIndices, Object, VerticesNormalsTriangles, Deleted};
use lines_manager::LinesManager;
use shaders_manager::{ShadersManager, ObjectShader, LinesShader};
use post_processing::post_processing_effect::PostProcessingEffect;
use builtins::loader;
use event;
use arc_ball;

mod error;

pub enum Light {
    Absolute(Vec3<GLfloat>),
    StickToCamera
}

/// Structure representing a window and a 3D scene. It is the main interface with the 3d engine.
pub struct Window {
    priv window:                @mut glfw::Window,
    priv max_ms_per_frame:      Option<u64>,
    priv objects:               ~[@mut Object],
    priv camera:                Camera,
    priv znear:                 f64,
    priv zfar:                  f64,
    priv light_mode:            Light,
    priv wireframe_mode:        bool,
    priv textures:              HashMap<~str, GLuint>,
    priv geometries:            HashMap<~str, GeometryIndices>,
    priv background:            Vec3<GLfloat>,
    priv projection:            Mat4<f64>,
    priv inv_projection:        Mat4<f64>,
    priv lines_manager:         @mut LinesManager, // FIXME: @mut should not be used here
    priv shaders_manager:       ShadersManager,
    priv usr_loop_callback:     @fn(),
    priv usr_keyboard_callback: @fn(&event::KeyboardEvent) -> bool,
    priv usr_mouse_callback:    @fn(&event::MouseEvent) -> bool,
    priv post_processing:       Option<@mut PostProcessingEffect>,
    priv process_fbo:           GLuint,
    priv process_fbo_texture:   GLuint,
    priv process_rbo_depth:     GLuint
}

impl Window {
    /// Sets the current processing effect.
    pub fn set_post_processing_effect(&mut self, effect: Option<@mut PostProcessingEffect>) {
        self.post_processing = effect;
    }

    /// Sets the maximum number of frames per second. Cannot be 0. `None` means there is no limit.
    pub fn set_framerate_limit(&mut self, fps: Option<u64>) {
        self.max_ms_per_frame = do fps.map |f| { assert!(*f != 0); 1000 / *f }
    }

    /// The `znear` value used by the perspective projection.
    pub fn znear(&self) -> f64 {
        self.znear
    }

    /// The `zfar` value used by the perspective projection.
    pub fn zfar(&self) -> f64 {
        self.zfar
    }

    /// The width of the window.
    pub fn width(&self) -> f64 {
        let (w, _) = self.window.get_size();

        w as f64
    }

    /// The height of the window.
    pub fn height(&self) -> f64 {
        let (_, h) = self.window.get_size();

        h as f64
    }

    /// Closes the window.
    pub fn close(@mut self) {
        self.window.set_should_close(true)
    }

    /// Hides the window, without closing it. Use `show` to make it visible again.
    pub fn hide(@mut self) {
        self.window.hide()
    }

    /// Makes the window visible. Use `hide` to hide it.
    pub fn show(@mut self) {
        self.window.show()
    }

    /// Switch on or off wireframe rendering mode. When set to `true`, everything in the scene will
    /// be drawn using wireframes. Wireframe rendering mode cannot be enabled on a per-object basis.
    pub fn set_wireframe_mode(@mut self, mode: bool) {
        self.wireframe_mode = mode;
    }

    /// Sets the background color.
    pub fn set_background_color(@mut self, r: f64, g: GLfloat, b: f64) {
        self.background.x = r as GLfloat;
        self.background.y = g as GLfloat;
        self.background.z = b as GLfloat;
    }

    /// Adds a line to be drawn during the next frame.
    pub fn draw_line(@mut self, a: &Vec3<f64>, b: &Vec3<f64>, color: &Vec3<f64>) {
        self.lines_manager.draw_line(VecCast::from(a.clone()),
                                     VecCast::from(b.clone()),
                                     VecCast::from(color.clone()));
    }

    /// Adds a cube to the scene. The cube is initially axis-aligned and centered at (0, 0, 0).
    ///
    /// # Arguments
    ///   * `wx` - the cube extent along the z axis
    ///   * `wy` - the cube extent along the y axis
    ///   * `wz` - the cube extent along the z axis
    pub fn add_cube(@mut self, wx: GLfloat, wy: GLfloat, wz: GLfloat) -> @mut Object {
        // FIXME: this weird block indirection are here because of Rust issue #6248
        let res = {
            let geom = self.geometries.find(&~"cube").unwrap();
            let tex  = self.textures.find(&~"default").unwrap();
            @mut Object::new(
                self,
                *geom,
                1.0, 1.0, 1.0,
                *tex,
                wx, wy, wz, Deleted)
        };
        // FIXME: get the geometry

        self.objects.push(res);

        res
    }

    /// Adds a sphere to the scene. The sphere is initially centered at (0, 0, 0).
    ///
    /// # Arguments
    ///   * `r` - the sphere radius
    pub fn add_sphere(@mut self, r: GLfloat) -> @mut Object {
        // FIXME: this weird block indirection are here because of Rust issue #6248
        let res = {
            let geom = self.geometries.find(&~"sphere").unwrap();
            let tex  = self.textures.find(&~"default").unwrap();
            @mut Object::new(
                self,
                *geom,
                1.0, 1.0, 1.0,
                *tex,
                r / 0.5, r / 0.5, r / 0.5,
                Deleted)
        };
        // FIXME: get the geometry

        self.objects.push(res);

        res
    }

    /// Adds a cone to the scene. The cone is initially centered at (0, 0, 0) and points toward the
    /// positive `y` axis.
    ///
    /// # Arguments
    ///   * `h` - the cone height
    ///   * `r` - the cone base radius
    pub fn add_cone(@mut self, h: GLfloat, r: GLfloat) -> @mut Object {
        // FIXME: this weird block indirection are here because of Rust issue #6248
        let res = {
            let geom = self.geometries.find(&~"cone").unwrap();
            let tex  = self.textures.find(&~"default").unwrap();
            @mut Object::new(
                self,
                *geom,
                1.0, 1.0, 1.0,
                *tex,
                r / 0.5, h, r / 0.5,
                Deleted)
        };
        // FIXME: get the geometry

        self.objects.push(res);

        res
    }

    /// Adds a cylinder to the scene. The cylinder is initially centered at (0, 0, 0) and has its
    /// principal axis aligned with the `y` axis.
    ///
    /// # Arguments
    ///   * `h` - the cylinder height
    ///   * `r` - the cylinder base radius
    pub fn add_cylinder(@mut self, h: GLfloat, r: GLfloat) -> @mut Object {
        // FIXME: this weird block indirection are here because of Rust issue #6248
        let res = {
            let geom = self.geometries.find(&~"cylinder").unwrap();
            let tex  = self.textures.find(&~"default").unwrap();
            @mut Object::new(
                self,
                *geom,
                1.0, 1.0, 1.0,
                *tex,
                r / 0.5, h, r / 0.5,
                Deleted)
        };
        // FIXME: get the geometry

        self.objects.push(res);

        res
    }

    /// Adds a capsule to the scene. The capsule is initially centered at (0, 0, 0) and has its
    /// principal axis aligned with the `y` axis.
    ///
    /// # Arguments
    ///   * `h` - the capsule height
    ///   * `r` - the capsule caps radius
    pub fn add_capsule(@mut self, h: GLfloat, r: GLfloat) -> @mut Object {
        // FIXME: this weird block indirection are here because of Rust issue #6248
        let res = {
            let geom = self.geometries.find(&~"capsule").unwrap();
            let tex  = self.textures.find(&~"default").unwrap();
            @mut Object::new(
                self,
                *geom,
                1.0, 1.0, 1.0,
                *tex,
                r / 0.5, h, r / 0.5,
                Deleted)
        };
        // FIXME: get the geometry

        self.objects.push(res);

        res
    }

    /// Adds a double-sided quad to the scene. The cylinder is initially centered at (0, 0, 0). The
    /// quad itself is composed of a user-defined number of triangles regularly spaced on a grid.
    /// This is the main way to draw height maps.
    ///
    /// # Arguments
    ///   * `w` - the quad width
    ///   * `h` - the quad height
    ///   * `wsubdivs` - number of horizontal subdivisions. This correspond to the number of squares
    ///   which will be placed horizontally on each line. Must not be `0`
    ///   * `hsubdivs` - number of vertical subdivisions. This correspond to the number of squares
    ///   which will be placed vertically on each line. Must not be `0`
    pub fn add_quad(@mut self,
                     w:        f64,
                     h:        f64,
                     wsubdivs: uint,
                     hsubdivs: uint)
                     -> @mut Object {
        assert!(wsubdivs > 0 && hsubdivs > 0,
        "The number of subdivisions cannot be zero");

        let wstep    = (w as GLfloat) / (wsubdivs as GLfloat);
        let hstep    = (h as GLfloat) / (hsubdivs as GLfloat);
        let wtexstep = 1.0 / (wsubdivs as GLfloat);
        let htexstep = 1.0 / (hsubdivs as GLfloat);
        let cw       = w as GLfloat / 2.0;
        let ch       = h as GLfloat / 2.0;

        let mut vertices   = ~[];
        let mut normals    = ~[];
        let mut triangles  = ~[];
        let mut tex_coords = ~[];

        // create the vertices
        for i in range(0u, hsubdivs + 1) {
            for j in range(0u, wsubdivs + 1) {
                vertices.push(Vec3::new(j as GLfloat * wstep - cw, i as GLfloat * hstep - ch, 0.0));
                tex_coords.push(Vec2::new(1.0 - j as GLfloat * wtexstep, 1.0 - i as GLfloat * htexstep))
            }
        }

        // create the normals
        do ((hsubdivs + 1) * (wsubdivs + 1)).times {
            { normals.push(Vec3::new(1.0 as GLfloat, 0.0, 0.0)) }
        }

        // create triangles
        fn dl_triangle(i: u32, j: u32, ws: u32) -> (u32, u32, u32) {
            ((i + 1) * ws + j, i * ws + j, (i + 1) * ws + j + 1)
        }

        fn ur_triangle(i: u32, j: u32, ws: u32) -> (u32, u32, u32) {
            (i * ws + j, i * ws + (j + 1), (i + 1) * ws + j + 1)
        }

        fn inv_wind((a, b, c): (u32, u32, u32)) -> (u32, u32, u32) {
            (b, a, c)
        }

        for i in range(0u, hsubdivs) {
            for j in range(0u, wsubdivs) {
                // build two triangles...
                triangles.push(dl_triangle(i as GLuint,
                j as GLuint,
                (wsubdivs + 1) as GLuint));
                triangles.push(ur_triangle(i as GLuint,
                j as GLuint,
                (wsubdivs + 1) as GLuint));
            }
        }

        // create gpu buffers
        let vertex_buf:   GLuint = 0;
        let element_buf:  GLuint = 0;
        let normal_buf:   GLuint = 0;
        let texture_buf:  GLuint = 0;

        unsafe {
            // FIXME: use gl::GenBuffers(3, ...) ?
            verify!(gl::GenBuffers(1, &vertex_buf));
            verify!(gl::GenBuffers(1, &element_buf));
            verify!(gl::GenBuffers(1, &normal_buf));
            verify!(gl::GenBuffers(1, &texture_buf));
        }

        // copy vertices
        unsafe {
            verify!(gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buf));
            verify!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * 3 * sys::size_of::<GLfloat>()) as GLsizeiptr,
                cast::transmute(&vertices[0]),
                gl::DYNAMIC_DRAW
            ));
        }

        // copy elements
        unsafe {
            verify!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buf));
            verify!(gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (triangles.len() * 3 * sys::size_of::<GLuint>()) as GLsizeiptr,
                cast::transmute(&triangles[0]),
                gl::STATIC_DRAW
            ));
        }

        // copy normals
        unsafe {
            verify!(gl::BindBuffer(gl::ARRAY_BUFFER, normal_buf));
            verify!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (normals.len() * 3 * sys::size_of::<GLfloat>()) as GLsizeiptr,
                cast::transmute(&normals[0]),
                gl::DYNAMIC_DRAW
            ));
        }

        // copy texture coordinates
        unsafe {
            verify!(gl::BindBuffer(gl::ARRAY_BUFFER, texture_buf));
            verify!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (tex_coords.len() * 2 * sys::size_of::<GLfloat>()) as GLsizeiptr,
                cast::transmute(&tex_coords[0]),
                gl::STATIC_DRAW
            ));
        }

        // FIXME: this weird block indirection are here because of Rust issue #6248
        let res = {
            let tex = self.textures.find(&~"default").unwrap();
            @mut Object::new(
                self,
                GeometryIndices::new(0, (triangles.len() * 3) as i32,
                element_buf, normal_buf, vertex_buf, texture_buf),
                1.0, 1.0, 1.0,
                *tex,
                1.0, 1.0, 1.0,
                VerticesNormalsTriangles(vertices, normals, triangles)
                )
        };

        self.objects.push(res);

        res
    }

    #[doc(hidden)]
    pub fn add_texture(@mut self, path: ~str) -> GLuint {
        let tex: Option<GLuint> = self.textures.find(&path).map(|e| **e);

        match tex {
            Some(id) => id,
            None => {
                let texture: GLuint = 0;

                unsafe {
                    verify!(gl::GenTextures(1, &texture));

                    match load_with_depth(path.clone(), 3, false) {
                        ImageU8(image) => {
                            verify!(gl::ActiveTexture(gl::TEXTURE0));
                            verify!(gl::BindTexture(gl::TEXTURE_2D, texture));

                            verify!(gl::TexImage2D(
                                gl::TEXTURE_2D, 0,
                                gl::RGB as GLint,
                                image.width as GLsizei,
                                image.height as GLsizei,
                                0, gl::RGB, gl::UNSIGNED_BYTE,
                                cast::transmute(&image.data[0])
                                ));

                            verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint));
                            verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint));
                            verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint));
                            verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint));
                        }
                        _ => {
                            fail!("Failed to load texture " + path);
                        }
                    }
                }

                self.textures.insert(path.clone(), texture);

                texture
            }
        }
    }

    /// Converts a 3d point to 2d screen coordinates.
    pub fn project(&self, world_coord: &Vec3<f64>) -> Vec2<f64> {
        let camera_coord   = self.camera.transformation().inverse().unwrap().rmul(world_coord);
        let h_camera_coord = camera_coord.to_homogeneous();

        let h_normalized_coord = self.projection.rmul(&h_camera_coord);

        let normalized_coord: Vec3<f64> = FromHomogeneous::from(&h_normalized_coord);

        let (w, h) = self.window.get_size();

        Vec2::new(
            (1.0 + normalized_coord.x) * (w as f64) / 2.0,
            (1.0 + normalized_coord.y) * (h as f64) / 2.0)
    }

    /// Converts a point in 2d screen coordinates to a ray (a 3d position and a direction).
    pub fn unproject(&self, window_coord: &Vec2<f64>) -> (Vec3<f64>, Vec3<f64>) {
        let (w, h) = self.window.get_size();

        let normalized_coord = Vec2::new(2.0 * window_coord.x / (w as f64) - 1.0,
                                         2.0 * -window_coord.y / (h as f64) + 1.0);

        let normalized_begin = Vec4::new(normalized_coord.x, normalized_coord.y, -1.0, 1.0);
        let normalized_end   = Vec4::new(normalized_coord.x, normalized_coord.y, 1.0, 1.0);

        let h_begin = self.inv_projection.rmul(&normalized_begin);
        let h_end   = self.inv_projection.rmul(&normalized_end);

        let begin = FromHomogeneous::from(&h_begin);
        let end   = FromHomogeneous::from(&h_end);

        let cam = self.camera.transformation();

        let unprojected_begin = cam.rmul(&begin);
        let unprojected_end   = cam.rmul(&end);

        (unprojected_begin, (unprojected_end - unprojected_begin).normalized())
    }

    /// The list of objects on the scene.
    pub fn objects<'r>(&'r self) -> &'r ~[@mut Object] {
        &self.objects
    }

    fn exec_callback(@mut self) {
        (self.usr_loop_callback)()
    }

    /// Sets the user-defined callback called at each event-pooling iteration of the engine.
    pub fn set_loop_callback(@mut self, callback: @fn()) {
        self.usr_loop_callback = callback
    }

    /// Sets the user-defined callback called whenever a keyboard event is triggered. It is called
    /// before any specific event handling from the engine (e.g. for the camera).
    ///
    /// # Arguments
    ///   * callback - the user-defined keyboard event handler. If it returns `false`, the event will
    ///   not be further handled by the engine. Handlers overriding some of the default behaviour of
    ///   the engine typically return `false`.
    pub fn set_keyboard_callback(@mut self, callback: @fn(&event::KeyboardEvent) -> bool) {
        self.usr_keyboard_callback = callback
    }

    /// Sets the user-defined callback called whenever a mouse event is triggered. It is called
    /// before any specific event handling from the engine (e.g. for the camera).
    ///
    /// # Arguments
    ///   * callback - the user-defined mouse event handler. If it returns `false`, the event will
    ///   not be further handled by the engine. Handlers overriding some of the default behaviour of
    ///   the engine typically return `false`.
    pub fn set_mouse_callback(@mut self, callback: @fn(&event::MouseEvent) -> bool) {
        self.usr_mouse_callback = callback
    }

    /// Sets the light mode. Only one light is supported.
    pub fn set_light(@mut self, pos: Light) {
        match pos {
            Absolute(p)   => self.set_light_pos(&p),
            StickToCamera => {
                let camera_pos = self.camera.transformation().translation();
                self.set_light_pos(&VecCast::from(camera_pos))
            }
        }

        self.light_mode = pos;
    }

    fn set_light_pos(@mut self, pos: &Vec3<GLfloat>) {
        self.shaders_manager.select(ObjectShader);
        verify!(gl::Uniform3f(self.shaders_manager.object_context().light, pos.x, pos.y, pos.z));
        // FIXME: select the LinesShader too ?
    }

    /// The camera used to render the scene. Only one camera is supported.
    pub fn camera<'r>(&'r mut self) -> &'r mut Camera {
        &'r mut self.camera
    }

    /// Opens a window and hide it. Once the window is created and before any event pooling, a
    /// user-defined callback is called once.
    ///
    /// This method contains an infinite loop and returns when the window is closed.
    ///
    /// # Arguments
    ///   * `title` - the window title
    ///   * `callback` - a callback called once the window has been created
    pub fn spawn_hidden(title: &str, callback: ~fn(@mut Window)) {
        Window::do_spawn(title.to_owned(), true, callback)
    }

    /// Opens a window. Once the window is created and before any event pooling, a user-defined
    /// callback is called once.
    ///
    /// This method contains an infinite loop and returns when the window is closed.
    ///
    /// # Arguments
    ///   * `title` - the window title
    ///   * `callback` - a callback called once the window has been created
    pub fn spawn(title: &str, callback: ~fn(@mut Window)) {
        Window::do_spawn(title.to_owned(), false, callback)
    }

    fn do_spawn(title: ~str, hide: bool, callback: ~fn(@mut Window)) {
        glfw::set_error_callback(error_callback);

        do glfw::start {
            let window = @mut glfw::Window::create(800, 600, title, glfw::Windowed).unwrap();

            window.make_context_current();

            verify!(gl::load_with(glfw::get_proc_address));

            init_gl();

            // FIXME: load that iff the user really uses post-processing
            let (process_fbo_texture, process_rbo_depth, process_fbo) = init_post_process_fbo(800, 600);

            let shaders      = ShadersManager::new();
            let mut textures = HashMap::new();
            let builtins     = loader::load(shaders.object_context(), &mut textures);

            let usr_window = @mut Window {
                max_ms_per_frame:      None,
                window:                window,
                objects:               ~[],
                camera:                Camera::new(ArcBall(arc_ball::ArcBall::new(-Vec3::z(), Zero::zero()))),
                znear:                 0.1,
                zfar:                  1024.0,
                light_mode:            Absolute(Vec3::new(0.0, 10.0, 0.0)),
                wireframe_mode:        false,
                textures:              textures,
                geometries:            builtins,
                background:            Vec3::new(0.0, 0.0, 0.0),
                inv_projection:        One::one(),
                projection:            One::one(),
                lines_manager:         @mut LinesManager::new(),
                shaders_manager:       shaders,
                post_processing:       None,
                process_fbo:           process_fbo,
                process_fbo_texture:   process_fbo_texture,
                process_rbo_depth:     process_rbo_depth,
                usr_loop_callback:     || {},
                usr_keyboard_callback: |_| { true },
                usr_mouse_callback:    |_| { true },
            };

            callback(usr_window);

            usr_window.set_light(usr_window.light_mode);

            // setup callbacks
            window.set_key_callback(|_, a, b, c, d| usr_window.key_callback(a, b, c, d));
            window.set_mouse_button_callback(|_, b, a, m| usr_window.mouse_button_callback(b, a, m));
            window.set_scroll_callback(|_, xoff, yoff| usr_window.scroll_callback(xoff, yoff));
            window.set_cursor_pos_callback(|_, xpos, ypos| usr_window.cursor_pos_callback(xpos, ypos));
            window.set_size_callback(|_, w, h| usr_window.size_callback(w, h));
            window.set_size(800, 600);
            usr_window.size_callback(800, 600);

            if hide {
                window.hide()
            }

            let mut timer = Timer::new().unwrap();
            let mut curr  = time::precise_time_ns();

            while !window.should_close() {
                usr_window.draw(&mut curr, &mut timer)
            }
        }
    }

    fn draw(@mut self, curr: &mut u64, timer: &mut Timer) {
        // Poll events
        glfw::poll_events();

        self.exec_callback();
        self.camera.update(self.window);

        if self.camera.needs_rendering() {
            self.shaders_manager.select(LinesShader);
            let view_location2 = self.shaders_manager.lines_context().view;
            self.camera.upload(view_location2);

            self.shaders_manager.select(ObjectShader);
            let view_location1 = self.shaders_manager.object_context().view;
            self.camera.upload(view_location1);
        }

        match self.light_mode {
            StickToCamera => self.set_light(StickToCamera),
            _             => { }
        }

        if self.post_processing.is_some() {
            // if we need post-processing, render to our own frame buffer
            verify!(gl::BindFramebuffer(gl::FRAMEBUFFER, self.process_fbo));
        }

        // Clear the screen to black
        verify!(gl::ClearColor(
            self.background.x,
            self.background.y,
            self.background.z,
            1.0));
        verify!(gl::Clear(gl::COLOR_BUFFER_BIT));
        verify!(gl::Clear(gl::DEPTH_BUFFER_BIT));

        if self.lines_manager.needs_rendering() {
            self.shaders_manager.select(LinesShader);
            self.lines_manager.upload(self.shaders_manager.lines_context());
            self.shaders_manager.select(ObjectShader);
        }

        if self.wireframe_mode {
            verify!(gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE));
        }
        else {
            verify!(gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL));
        }

        for o in self.objects.iter() {
            o.upload(self.shaders_manager.object_context())
        }

        match self.post_processing {
            Some(p) => {
                // remove the wireframe mode
                if self.wireframe_mode {
                    verify!(gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL));
                }

                // switch back to the screen framebuffer …
                verify!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));
                // … and execute the post-process
                p.update(0.016); // FIXME: use the real value
                p.draw(&mut self.shaders_manager, self.process_fbo_texture);
            },
            None => { }
        }

        // We are done: swap buffers
        self.window.swap_buffers();

        // Limit the fps if needed.
        match self.max_ms_per_frame {
            None     => { },
            Some(ms) => {
                let elapsed = (time::precise_time_ns() - *curr) / 1000000;
                if elapsed < ms {
                    timer.sleep(ms - elapsed);
                }
            }
        }

        *curr = time::precise_time_ns();
    }

    fn key_callback(@mut self,
                    key:    libc::c_int,
                    _:      libc::c_int,
                    action: libc::c_int,
                    _:      libc::c_int) {
        let event = if action == glfw::PRESS {
            event::KeyPressed(key)
        }
        else { // if action == glfw::RELEASE
            event::KeyReleased(key)
        };

        if !(self.usr_keyboard_callback)(&event) {
            return
        }

        if action == glfw::PRESS && key == glfw::KEY_ESCAPE {
            self.window.set_should_close(true);
        }

        if action == glfw::PRESS && key == glfw::KEY_SPACE {
            self.set_wireframe_mode(!self.wireframe_mode);
        }

        self.camera.handle_keyboard(&event);
    }

    fn cursor_pos_callback(@mut self, xpos: float, ypos: float) {
        let event = event::CursorPos(xpos, ypos);

        if (self.usr_mouse_callback)(&event) {
            self.camera.handle_mouse(&event)
        }
    }

    fn scroll_callback(@mut self, xoff: float, yoff: float) {
        let event = event::Scroll(xoff, yoff);

        if (self.usr_mouse_callback)(&event) {
            self.camera.handle_mouse(&event)
        }
    }

    fn mouse_button_callback(@mut self,
                             button: libc::c_int,
                             action: libc::c_int,
                             mods:   libc::c_int) {
        let event = if action == 1 {
            event::ButtonPressed(button, mods)
        }
        else {
            event::ButtonReleased(button, mods)
        };

        if !(self.usr_mouse_callback)(&event) {
            return
        }


        self.camera.handle_mouse(&event)
    }

    fn size_callback(@mut self, w: int, h: int) {
        // Update the viewport
        verify!(gl::Viewport(0, 0, w as i32, h as i32));

        // Update the fbo
        verify!(gl::BindTexture(gl::TEXTURE_2D, self.process_fbo));
        unsafe {
            verify!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, w as GLint, h as GLint, 0,
                    gl::RGBA, gl::UNSIGNED_BYTE, ptr::null()));
        }
        verify!(gl::BindTexture(gl::TEXTURE_2D, 0));

        verify!(gl::BindRenderbuffer(gl::RENDERBUFFER, self.process_rbo_depth));
        verify!(gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT16, w as GLint, h as GLint));
        verify!(gl::BindRenderbuffer(gl::RENDERBUFFER, 0));

        // Update the projection
        self.projection = self.projection();
        self.inv_projection = self.projection.inverse().unwrap();
        let projection: Mat4<GLfloat> = MatCast::from(self.projection.transposed());

        unsafe {
            self.shaders_manager.select(LinesShader);

            verify!(gl::UniformMatrix4fv(
                self.shaders_manager.lines_context().proj,
                1,
                gl::FALSE as u8,
                cast::transmute(&projection)));

            self.shaders_manager.select(ObjectShader);

            verify!(gl::UniformMatrix4fv(
                self.shaders_manager.object_context().proj,
                1,
                gl::FALSE as u8,
                cast::transmute(&projection)));
        }
    }

    /// The projection matrix used by the window.
    pub fn projection(&self) -> Mat4<f64> {
        let (w, h) = self.window.get_size();
        let fov    = (45.0 as f64).to_radians();
        let aspect = w as f64 / (h as f64);

        let sy = 1.0 / (fov * 0.5).tan();
        let sx = -sy / aspect;
        let sz = -(self.zfar + self.znear) / (self.znear - self.zfar);
        let tz = 2.0 * self.zfar * self.znear / (self.znear - self.zfar);

        Mat4::new(
            sx , 0.0, 0.0, 0.0,
            0.0, sy , 0.0, 0.0,
            0.0, 0.0, sz , tz,
            0.0, 0.0, 1.0, 0.0
            )
    }
}

fn error_callback(_: libc::c_int, description: ~str) {
    println(fmt!("Kiss3d Error: %s", description));
}

fn init_gl() {
    /*
     * Misc configurations
     */
    verify!(gl::FrontFace(gl::CCW));
    verify!(gl::Enable(gl::DEPTH_TEST));
    verify!(gl::DepthFunc(gl::LEQUAL));
}

fn init_post_process_fbo(width: uint, height: uint) -> (GLuint, GLuint, GLuint) {
    /* Create back-buffer, used for post-processing */
    let fbo_texture: GLuint = 0;
    let rbo_depth:   GLuint = 0;
    let fbo:         GLuint = 0;

    /* Texture */
    verify!(gl::ActiveTexture(gl::TEXTURE0));
    unsafe { verify!(gl::GenTextures(1, &fbo_texture)); }
    verify!(gl::BindTexture(gl::TEXTURE_2D, fbo_texture));
    verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint));
    verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint));
    verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint));
    verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint));
    unsafe {
        verify!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, width as GLint, height as GLint,
                       0, gl::RGBA, gl::UNSIGNED_BYTE, ptr::null()));
    }
    verify!(gl::BindTexture(gl::TEXTURE_2D, 0));

    /* Depth buffer */
    unsafe { verify!(gl::GenRenderbuffers(1, &rbo_depth)); }
    verify!(gl::BindRenderbuffer(gl::RENDERBUFFER, rbo_depth));
    verify!(gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT16, width as GLint, height as GLint));
    verify!(gl::BindRenderbuffer(gl::RENDERBUFFER, 0));

    /* Framebuffer to link everything together */
    unsafe { gl::GenFramebuffers(1, &fbo); }
    verify!(gl::BindFramebuffer(gl::FRAMEBUFFER, fbo));
    verify!(gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, fbo_texture, 0));
    verify!(gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, rbo_depth));
    verify!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));

    (fbo_texture, rbo_depth, fbo)
}
