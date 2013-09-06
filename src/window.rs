/*
 * FIXME: this file is too big. Some heavy refactoring need to be done here.
 */

use glfw;
use glfw::consts;
use std::ptr;
use std::rt::io::timer::Timer;
use std::rt::rtio::RtioTimer;
use std::num::Zero;
use std::libc;
use std::sys;
use std::cast;
use std::hashmap::HashMap;
use extra::time;
use extra::rc::Rc;
use extra::arc::RWArc;
use gl;
use gl::types::*;
use stb_image::image::*;
use nalgebra::mat::{RMul, ToHomogeneous, FromHomogeneous};
use nalgebra::vec::{Vec2, Vec3, Vec4, AlgebraicVec, VecCast};
use camera::{Camera, ArcBall};
use object::{GeometryIndices, Object, VerticesNormalsTriangles, Deleted};
use lines_manager::LinesManager;
use post_processing::post_processing_effect::PostProcessingEffect;
use resources::shaders_manager::{ShadersManager, ObjectShader, LinesShader};
use resources::textures_manager::{Texture, TexturesManager};
use resources::framebuffers_manager::{FramebuffersManager, Screen, Offscreen};
use builtins::loader;
use event;

mod error;

pub enum Light {
    Absolute(Vec3<GLfloat>),
    StickToCamera
}

/// Structure representing a window and a 3D scene. It is the main interface with the 3d engine.
pub struct Window {
    priv window:               glfw::Window,
    priv max_ms_per_frame:     Option<u64>,
    priv objects:              ~[Object],
    priv camera:               @mut Camera,
    priv light_mode:           Light,
    priv wireframe_mode:       bool,
    priv geometries:           HashMap<~str, GeometryIndices>,
    priv background:           Vec3<GLfloat>,
    priv lines_manager:        LinesManager,
    priv shaders_manager:      ShadersManager,
    priv textures_manager:     TexturesManager,
    priv framebuffers_manager: FramebuffersManager,
    priv post_processing:      Option<@mut PostProcessingEffect>,
    priv process_fbo_texture:  GLuint,
    priv process_fbo_depth:    GLuint,
    priv events:               RWArc<~[event::Event]>,
    priv keyboard_callback:    @fn(&mut Window, &event::KeyboardEvent) -> bool,
    priv mouse_callback:       @fn(&mut Window, &event::MouseEvent) -> bool
}

impl Window {
    /// Sets the current processing effect.
    pub fn set_post_processing_effect(&mut self, effect: Option<@mut PostProcessingEffect>) {
        self.post_processing = effect;
    }

    /// The window width.
    pub fn width(&self) -> f64 {
        let (w, _) = self.window.get_size();

        w as f64
    }

    /// The window height.
    pub fn height(&self) -> f64 {
        let (_, h) = self.window.get_size();

        h as f64
    }

    /// The current camera.
    pub fn camera(&self) -> @mut Camera {
        self.camera
    }

    /// The current camera.
    pub fn set_camera(&mut self, camera: @mut Camera) {
        let (w, h) = self.window.get_size();

        self.camera = camera;
        self.camera.handle_framebuffer_size_change(w as f64, h as f64);
    }

    /// Sets the maximum number of frames per second. Cannot be 0. `None` means there is no limit.
    pub fn set_framerate_limit(&mut self, fps: Option<u64>) {
        self.max_ms_per_frame = do fps.map |f| { assert!(*f != 0); 1000 / *f }
    }

    /// Closes the window.
    pub fn close(&mut self) {
        self.window.set_should_close(true)
    }

    /// Hides the window, without closing it. Use `show` to make it visible again.
    pub fn hide(&mut self) {
        self.window.hide()
    }

    /// Makes the window visible. Use `hide` to hide it.
    pub fn show(&mut self) {
        self.window.show()
    }

    /// Switch on or off wireframe rendering mode. When set to `true`, everything in the scene will
    /// be drawn using wireframes. Wireframe rendering mode cannot be enabled on a per-object basis.
    pub fn set_wireframe_mode(&mut self, mode: bool) {
        self.wireframe_mode = mode;
    }

    /// Sets the background color.
    pub fn set_background_color(&mut self, r: f64, g: GLfloat, b: f64) {
        self.background.x = r as GLfloat;
        self.background.y = g as GLfloat;
        self.background.z = b as GLfloat;
    }

    /// Adds a line to be drawn during the next frame.
    pub fn draw_line(&mut self, a: &Vec3<f64>, b: &Vec3<f64>, color: &Vec3<f64>) {
        self.lines_manager.draw_line(VecCast::from(a.clone()),
                                     VecCast::from(b.clone()),
                                     VecCast::from(color.clone()));
    }

    /// Removes an object from the scene.
    pub fn remove(&mut self, o: Object) {
        match self.objects.iter().rposition(|e| o == *e) {
            Some(i) => {
                // XXX: release textures and buffers if nobody else use them
                self.objects.swap_remove(i);
            },
            None => { }
        }
    }

    /// Adds a cube to the scene. The cube is initially axis-aligned and centered at (0, 0, 0).
    ///
    /// # Arguments
    ///   * `wx` - the cube extent along the z axis
    ///   * `wy` - the cube extent along the y axis
    ///   * `wz` - the cube extent along the z axis
    pub fn add_cube(&mut self, wx: GLfloat, wy: GLfloat, wz: GLfloat) -> Object {
        // FIXME: this weird block indirection are here because of Rust issue #6248
        let res = {
            let tex  = self.textures_manager.get("default").unwrap();
            let geom = self.geometries.find(&~"cube").unwrap();
            Object::new(
                *geom,
                1.0, 1.0, 1.0,
                tex,
                wx, wy, wz, Deleted)
        };
        // FIXME: get the geometry

        self.objects.push(res.clone());

        res
    }

    /// Adds a sphere to the scene. The sphere is initially centered at (0, 0, 0).
    ///
    /// # Arguments
    ///   * `r` - the sphere radius
    pub fn add_sphere(&mut self, r: GLfloat) -> Object {
        // FIXME: this weird block indirection are here because of Rust issue #6248
        let res = {
            let tex  = self.textures_manager.get("default").unwrap();
            let geom = self.geometries.find(&~"sphere").unwrap();
            Object::new(
                *geom,
                1.0, 1.0, 1.0,
                tex,
                r / 0.5, r / 0.5, r / 0.5,
                Deleted)
        };
        // FIXME: get the geometry

        self.objects.push(res.clone());

        res
    }

    /// Adds a cone to the scene. The cone is initially centered at (0, 0, 0) and points toward the
    /// positive `y` axis.
    ///
    /// # Arguments
    ///   * `h` - the cone height
    ///   * `r` - the cone base radius
    pub fn add_cone(&mut self, h: GLfloat, r: GLfloat) -> Object {
        // FIXME: this weird block indirection are here because of Rust issue #6248
        let res = {
            let tex  = self.textures_manager.get("default").unwrap();
            let geom = self.geometries.find(&~"cone").unwrap();
            Object::new(
                *geom,
                1.0, 1.0, 1.0,
                tex,
                r / 0.5, h, r / 0.5,
                Deleted)
        };
        // FIXME: get the geometry

        self.objects.push(res.clone());

        res
    }

    /// Adds a cylinder to the scene. The cylinder is initially centered at (0, 0, 0) and has its
    /// principal axis aligned with the `y` axis.
    ///
    /// # Arguments
    ///   * `h` - the cylinder height
    ///   * `r` - the cylinder base radius
    pub fn add_cylinder(&mut self, h: GLfloat, r: GLfloat) -> Object {
        // FIXME: this weird block indirection are here because of Rust issue #6248
        let res = {
            let tex  = self.textures_manager.get("default").unwrap();
            let geom = self.geometries.find(&~"cylinder").unwrap();
            Object::new(
                *geom,
                1.0, 1.0, 1.0,
                tex,
                r / 0.5, h, r / 0.5,
                Deleted)
        };
        // FIXME: get the geometry

        self.objects.push(res.clone());

        res
    }

    /// Adds a capsule to the scene. The capsule is initially centered at (0, 0, 0) and has its
    /// principal axis aligned with the `y` axis.
    ///
    /// # Arguments
    ///   * `h` - the capsule height
    ///   * `r` - the capsule caps radius
    pub fn add_capsule(&mut self, h: GLfloat, r: GLfloat) -> Object {
        // FIXME: this weird block indirection are here because of Rust issue #6248
        let res = {
            let tex  = self.textures_manager.get("default").unwrap();
            let geom = self.geometries.find(&~"capsule").unwrap();
            Object::new(
                *geom,
                1.0, 1.0, 1.0,
                tex,
                r / 0.5, h, r / 0.5,
                Deleted)
        };
        // FIXME: get the geometry

        self.objects.push(res.clone());

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
    pub fn add_quad(&mut self,
                     w:        f64,
                     h:        f64,
                     wsubdivs: uint,
                     hsubdivs: uint)
                     -> Object {
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
            let tex = self.textures_manager.get("default").unwrap();
            Object::new(
                GeometryIndices::new(0, (triangles.len() * 3) as i32,
                element_buf, normal_buf, vertex_buf, texture_buf),
                1.0, 1.0, 1.0,
                tex,
                1.0, 1.0, 1.0,
                VerticesNormalsTriangles(vertices, normals, triangles))
        };

        self.objects.push(res.clone());

        res
    }

    #[doc(hidden)]
    pub fn add_texture(&mut self, path: &str) -> Rc<Texture> {
        self.textures_manager.add(path)
    }

    /// Converts a 3d point to 2d screen coordinates.
    pub fn project(&self, world_coord: &Vec3<f64>) -> Vec2<f64> {
        let h_world_coord = world_coord.to_homogeneous();
        let h_normalized_coord = self.camera.transformation().rmul(&h_world_coord);

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

        let cam = self.camera.inv_transformation();

        let h_unprojected_begin = cam.rmul(&normalized_begin);
        let h_unprojected_end   = cam.rmul(&normalized_end);

        let unprojected_begin: Vec3<f64> = FromHomogeneous::from(&h_unprojected_begin);
        let unprojected_end: Vec3<f64>   = FromHomogeneous::from(&h_unprojected_end);

        (unprojected_begin, (unprojected_end - unprojected_begin).normalized())
    }

    /// The list of objects on the scene.
    pub fn objects<'r>(&'r self) -> &'r [Object] {
        let res: &'r [Object] = self.objects;

        res
    }

    /// The list of objects on the scene.
    pub fn objects_mut<'r>(&'r mut self) -> &'r mut [Object] {
        let res: &'r mut [Object] = self.objects;

        res
    }

    /// Starts an infinite loop polling events, calling an user-defined callback, and drawing the
    /// scene.
    pub fn render_loop(&mut self, callback: &fn(&mut Window)) {

        let mut timer = Timer::new().unwrap();
        let mut curr  = time::precise_time_ns();

        while !self.window.should_close() {
            // collect events
            glfw::poll_events();
            // redispatch them
            self.redispatch_events();
            // clear the events collector
            self.events.write(|c| c.clear());

            callback(self);

            self.draw(&mut curr, &mut timer)
        }
    }

    fn redispatch_events(&mut self) {
        let events = self.events.clone();
        do events.read |es| {
            for e in es.iter() {
                match *e {
                    event::Keyboard(ref k) => {
                        if (self.keyboard_callback)(self, k) {
                            match *k {
                                event::KeyReleased(key) => {
                                    if key == consts::KEY_ESCAPE {
                                        self.close();
                                        loop
                                    }
                                },
                                _ => { }
                            }

                            self.camera.handle_keyboard(&self.window, k);
                        }
                    },
                    event::Mouse(ref m) => {
                        if (self.mouse_callback)(self, m) {
                            self.camera.handle_mouse(&self.window, m);
                        }
                    },
                    event::FramebufferSize(w, h) => {
                        self.update_viewport(w, h);
                        self.camera.handle_framebuffer_size_change(w, h);
                    }
                }
            }
        }
    }

    /// Sets the user-defined callback called whenever a keyboard event is triggered. It is called
    /// before any specific event handling from the engine (e.g. for the camera).
    ///
    /// # Arguments
    ///   * callback - the user-defined keyboard event handler. If it returns `false`, the event will
    ///   not be further handled by the engine. Handlers overriding some of the default behaviour of
    ///   the engine typically return `false`.
    pub fn set_keyboard_callback(&mut self, callback: @fn(&mut Window, &event::KeyboardEvent) -> bool) {
        self.keyboard_callback = callback;
    }

    /// Sets the user-defined callback called whenever a mouse event is triggered. It is called
    /// before any specific event handling from the engine (e.g. for the camera).
    ///
    /// # Arguments
    ///   * callback - the user-defined mouse event handler. If it returns `false`, the event will
    ///   not be further handled by the engine. Handlers overriding some of the default behaviour of
    ///   the engine typically return `false`.
    pub fn set_mouse_callback(&mut self, callback: @fn(&mut Window, &event::MouseEvent) -> bool) {
        self.mouse_callback = callback;
    }

    /// Sets the light mode. Only one light is supported.
    pub fn set_light(&mut self, pos: Light) {
        match pos {
            Absolute(p)   => self.set_light_pos(&p),
            StickToCamera => {
                let camera_pos = self.camera.eye();
                self.set_light_pos(&VecCast::from(camera_pos))
            }
        }

        self.light_mode = pos;
    }

    fn set_light_pos(&mut self, pos: &Vec3<GLfloat>) {
        self.shaders_manager.select(ObjectShader);
        verify!(gl::Uniform3f(self.shaders_manager.object_context().light, pos.x, pos.y, pos.z));
        // FIXME: select the LinesShader too ?
    }

    // FIXME /// The camera used to render the scene.
    // FIXME pub fn camera(&self) -> &Camera {
    // FIXME     self.camera.clone()
    // FIXME }

    /// Opens a window and hide it. Once the window is created and before any event pooling, a
    /// user-defined callback is called once.
    ///
    /// This method contains an infinite loop and returns when the window is closed.
    ///
    /// # Arguments
    ///   * `title` - the window title
    ///   * `callback` - a callback called once the window has been created
    pub fn spawn_hidden(title: &str, callback: ~fn(&mut Window)) {
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
    pub fn spawn(title: &str, callback: ~fn(&mut Window)) {
        Window::do_spawn(title.to_owned(), false, callback)
    }

    fn do_spawn(title: ~str, hide: bool, callback: ~fn(&mut Window)) {
        glfw::set_error_callback(error_callback);

        do glfw::start {
            let window = glfw::Window::create(800, 600, title, glfw::Windowed).unwrap();

            window.make_context_current();

            verify!(gl::load_with(glfw::get_proc_address));

            init_gl();

            // FIXME: load that iff the user really uses post-processing
            let (process_fbo_texture, process_fbo_depth) = init_post_process_buffers(800, 600);

            let mut textures = TexturesManager::new(); 
            let shaders      = ShadersManager::new();
            let builtins     = loader::load(shaders.object_context(), &mut textures);
            let camera       = @mut ArcBall::new(-Vec3::z(), Zero::zero());

            let mut usr_window = Window {
                max_ms_per_frame:      None,
                window:                window,
                objects:               ~[],
                camera:                camera as @mut Camera,
                light_mode:            Absolute(Vec3::new(0.0, 10.0, 0.0)),
                wireframe_mode:        false,
                geometries:            builtins,
                background:            Vec3::new(0.0, 0.0, 0.0),
                lines_manager:         LinesManager::new(),
                shaders_manager:       shaders,
                post_processing:       None,
                process_fbo_texture:   process_fbo_texture,
                process_fbo_depth:     process_fbo_depth,
                textures_manager:      textures,
                framebuffers_manager:  FramebuffersManager::new(),
                events:                RWArc::new(~[]),
                keyboard_callback:     |_, _| { true },
                mouse_callback:        |_, _| { true }
            };

            // setup callbacks
            let collector = usr_window.events.clone();
            do usr_window.window.set_framebuffer_size_callback |_, w, h| {
                collector.write(|c| c.push(event::FramebufferSize(w as f64, h as f64)))
            }

            let collector = usr_window.events.clone();
            do usr_window.window.set_key_callback |_, key, _, action, _| {
                if action == 1 {
                    collector.write(|c| c.push(event::Keyboard(event::KeyPressed(key))))
                }
                else {
                    collector.write(|c| c.push(event::Keyboard(event::KeyReleased(key))))
                }
            }

            let collector = usr_window.events.clone();
            do usr_window.window.set_mouse_button_callback |_, button, action, mods| {
                if action == 1 {
                    collector.write(|c| c.push(event::Mouse(event::ButtonPressed(button, mods))))
                }
                else {
                    collector.write(|c| c.push(event::Mouse(event::ButtonReleased(button, mods))))
                }
            }

            let collector = usr_window.events.clone();
            do usr_window.window.set_cursor_pos_callback |_, x, y| {
                collector.write(|c| c.push(event::Mouse(event::CursorPos(x, y))))
            }

            let collector = usr_window.events.clone();
            do usr_window.window.set_scroll_callback |_, x, y| {
                collector.write(|c| c.push(event::Mouse(event::Scroll(x, y))))
            }

            let (w, h) = usr_window.window.get_size();
            usr_window.camera.handle_framebuffer_size_change(w as f64, h as f64);

            if hide {
                usr_window.window.hide()
            }

            // usr_window.framebuffer_size_callback(800, 600);
            usr_window.set_light(usr_window.light_mode);

            callback(&mut usr_window);
        }
    }

    fn draw(&mut self, curr: &mut u64, timer: &mut Timer) {
        self.camera.update(&self.window);

        self.shaders_manager.select(LinesShader);
        let view_location2 = self.shaders_manager.lines_context().view;
        self.camera.upload(view_location2);

        self.shaders_manager.select(ObjectShader);
        let view_location1 = self.shaders_manager.object_context().view;
        self.camera.upload(view_location1);

        match self.light_mode {
            StickToCamera => self.set_light(StickToCamera),
            _             => { }
        }

        if self.post_processing.is_some() {
            // if we need post-processing, render to our own frame buffer
            self.framebuffers_manager.select(Offscreen(self.process_fbo_texture, self.process_fbo_depth));
        }
        else {
            self.framebuffers_manager.select(Screen);
        }

        self.render_scene();

        let w = self.width() as f64;
        let h = self.height() as f64;
        let (znear, zfar) = self.camera.clip_planes();

        match self.post_processing {
            Some(ref mut p) => {
                // remove the wireframe mode
                if self.wireframe_mode {
                    verify!(gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL));
                }

                // switch back to the screen framebuffer …
                self.framebuffers_manager.select(Screen);
                // … and execute the post-process
                // FIXME: use the real time value instead of 0.016!
                p.update(0.016, w, h, znear, zfar);
                p.draw(&mut self.shaders_manager, self.process_fbo_texture, self.process_fbo_depth);
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

        // self.transparent_objects.clear();
        // self.opaque_objects.clear();
    }

    fn render_scene(&mut self) {
        // Activate the default texture
        verify!(gl::ActiveTexture(gl::TEXTURE0));
        // Clear the screen to black
        verify!(gl::ClearColor(self.background.x, self.background.y, self.background.z, 1.0));
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
    }


    fn update_viewport(&mut self, w: f64, h: f64) {
        // Update the viewport
        verify!(gl::Viewport(0, 0, w as i32, h as i32));

        // Update the fbo
        verify!(gl::BindTexture(gl::TEXTURE_2D, self.process_fbo_texture));
        unsafe {
            verify!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, w as GLint, h as GLint, 0,
            gl::RGBA, gl::UNSIGNED_BYTE, ptr::null()));
        }
        verify!(gl::BindTexture(gl::TEXTURE_2D, 0));

        verify!(gl::BindTexture(gl::TEXTURE_2D, self.process_fbo_depth));
        unsafe {
            verify!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as GLint, w as GLint, h as GLint, 0,
            gl::DEPTH_COMPONENT, gl::UNSIGNED_BYTE, ptr::null()));
        }
        verify!(gl::BindTexture(gl::TEXTURE_2D, 0));
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

fn init_post_process_buffers(width: uint, height: uint) -> (GLuint, GLuint) {
    /* Create back-buffer, used for post-processing */
    let fbo_texture: GLuint = 0;
    let fbo_depth:   GLuint = 0;

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
    verify!(gl::ActiveTexture(gl::TEXTURE1));
    unsafe { verify!(gl::GenTextures(1, &fbo_depth)); }
    verify!(gl::BindTexture(gl::TEXTURE_2D, fbo_depth));
    verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint));
    verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint));
    verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint));
    verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint));
    unsafe {
        verify!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as GLint, width as GLint, height as GLint,
                0, gl::DEPTH_COMPONENT, gl::UNSIGNED_BYTE, ptr::null()));
    }
    verify!(gl::BindTexture(gl::TEXTURE_2D, 0));

    (fbo_texture, fbo_depth)
}
