//! The kiss3d window.
/*
 * FIXME: this file is too big. Some heavy refactoring need to be done here.
 */

use glfw;
use std::libc;
use std::io::timer::Timer;
use std::num::Zero;
use std::hashmap::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use extra::time;
use extra::arc::RWArc;
use gl;
use gl::types::*;
use stb_image::image::*;
use nalgebra::na::{Vec2, Vec3, Vec4};
use nalgebra::na;
use camera::{Camera, ArcBall};
use object::Object;
use lines_manager::LinesManager;
use post_processing::post_processing_effect::PostProcessingEffect;
use resources::shaders_manager::{ShadersManager, ObjectShader, LinesShader};
use resources::textures_manager::Texture;
use resources::textures_manager;
use resources::framebuffers_manager::{FramebuffersManager, RenderTarget};
use builtins::loader;
use event;
use mesh::{Mesh, StorageLocation};
use obj;
use mtl::MtlMaterial;

mod error;

/// The light configuration.
pub enum Light {
    /// A light with an absolute world position.
    Absolute(Vec3<GLfloat>),
    /// A light superimposed with the camera position.
    StickToCamera
}

static DEFAULT_WIDTH:  u32 = 800u32;
static DEFAULT_HEIGHT: u32 = 600u32;

/// Structure representing a window and a 3D scene. It is the main interface with the 3d engine.
pub struct Window<'a> {
    priv window:                     glfw::Window,
    priv max_ms_per_frame:           Option<u64>,
    priv objects:                    ~[Object],
    priv camera:                     &'a mut Camera,
    priv light_mode:                 Light,
    priv wireframe_mode:             bool,
    priv geometries:                 HashMap<~str, Rc<RefCell<Mesh>>>,
    priv background:                 Vec3<GLfloat>,
    priv lines_manager:              LinesManager,
    priv shaders_manager:            ShadersManager,
    priv framebuffers_manager:       FramebuffersManager,
    priv post_processing:            Option<&'a mut PostProcessingEffect>,
    priv post_process_render_target: RenderTarget,
    priv events:                     RWArc<~[event::Event]>
}

impl<'a> Window<'a> {
    /// Access the glfw window.
    pub fn glfw_window<'r>(&'r self) -> &'r glfw::Window {
        &self.window
    }

    /// Sets the current processing effect.
    pub fn set_post_processing_effect(&mut self, effect: Option<&'a mut PostProcessingEffect>) {
        self.post_processing = effect;
    }

    /// The window width.
    pub fn width(&self) -> f32 {
        let (w, _) = self.window.get_size();

        w as f32
    }

    /// The window height.
    pub fn height(&self) -> f32 {
        let (_, h) = self.window.get_size();

        h as f32
    }

    /// The current camera.
    pub fn camera<'b>(&'b self) -> &'b &'a mut Camera {
        &'b self.camera
    }

    /// The current camera.
    pub fn set_camera(&mut self, camera: &'a mut Camera) {
        let (w, h) = self.window.get_size();

        self.camera = camera;
        self.camera.handle_event(&self.window, &event::FramebufferSize(w as f32, h as f32));
    }

    /// Sets the maximum number of frames per second. Cannot be 0. `None` means there is no limit.
    pub fn set_framerate_limit(&mut self, fps: Option<u64>) {
        self.max_ms_per_frame = fps.map(|f| { assert!(f != 0); 1000 / f })
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
    pub fn set_background_color(&mut self, r: f32, g: GLfloat, b: f32) {
        self.background.x = r;
        self.background.y = g;
        self.background.z = b;
    }

    /// Adds a line to be drawn during the next frame.
    pub fn draw_line(&mut self, a: &Vec3<f32>, b: &Vec3<f32>, color: &Vec3<f32>) {
        self.lines_manager.draw_line(a.clone(), b.clone(), color.clone());
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

    /// Loads a mesh from an obj file located at `path` and registers its geometry as
    /// `geometry_name`.
    pub fn load_obj(&mut self, path: &Path, mtl_dir: &Path, geometry_name: &str) -> ~[(~str, Rc<RefCell<Mesh>>, Option<MtlMaterial>)] {
        let ms = obj::parse_file(path, mtl_dir, geometry_name).expect("Unable to parse the obj file: " + path.as_str().unwrap());

        let mut res = ~[];

        for (n, m, mat) in ms.move_iter() {
            let m = Rc::new(RefCell::new(m));
            self.geometries.insert(geometry_name.to_owned(), m.clone());

            res.push((n, m, mat));
        }

        res
    }

    /// Gets the geometry named `geometry_name` if it has been already registered.
    pub fn get_mesh(&mut self, geometry_name: &str) -> Option<Rc<RefCell<Mesh>>> {
        self.geometries.find(&geometry_name.to_owned()).map(|m| m.clone())
    }

    /// Registers the geometry `mesh` with the name `geometry_name`.
    pub fn register_mesh(&mut self, geometry_name: &str, mesh: Mesh) {
        self.geometries.insert(geometry_name.to_owned(), Rc::new(RefCell::new(mesh)));
    }

    /// Adds an obj model to the scene.
    ///
    /// # Arguments
    ///     * `path`  - relative path to the obj file.
    ///     * `scale` - uniform scale to apply to the model.
    pub fn add_obj(&mut self, path: &Path, mtl_dir: &Path, scale: GLfloat) -> ~[Object] {
        let tex  = textures_manager::get(|tm| tm.get_default());
        let objs = self.load_obj(path, mtl_dir, path.as_str().unwrap());
        println!("Parsing complete.");

        let mut res = ~[];

        for (_, mesh, mtl) in objs.move_iter() {
            let mut object = Object::new(
                mesh,
                1.0, 1.0, 1.0,
                tex.clone(),
                scale, scale, scale
                );

            match mtl {
                None      => { },
                Some(mtl) => {
                    object.set_color(mtl.diffuse.x, mtl.diffuse.y, mtl.diffuse.z);

                    mtl.diffuse_texture.as_ref().map(|t| {
                        let mut tpath = mtl_dir.clone();
                        tpath.push(t.as_slice());
                        object.set_texture(&tpath, tpath.as_str().unwrap())
                    });

                    mtl.ambiant_texture.map(|t| {
                        let mut tpath = mtl_dir.clone();
                        tpath.push(t.as_slice());
                        object.set_texture(&tpath, tpath.as_str().unwrap())
                    });
                }
            }

            res.push(object.clone());
            self.objects.push(object);
        }

        res
    }

    /// Adds an unnamed mesh to the scene.
    pub fn add_mesh(&mut self, mesh: Mesh, scale: GLfloat) -> Object {
        let tex  = textures_manager::get(|tm| tm.get_default());

        let res = Object::new(
                    Rc::new(RefCell::new(mesh)),
                    1.0, 1.0, 1.0,
                    tex,
                    scale, scale, scale);

        self.objects.push(res.clone());

        res
    }

    /// Creates and adds a new object using the geometry registered as `geometry_name`.
    pub fn add(&mut self, geometry_name: &str, scale: GLfloat) -> Option<Object> {
        self.geometries.find(&geometry_name.to_owned()).map(|m| {
            let res = Object::new(
                        m.clone(),
                        1.0, 1.0, 1.0,
                        textures_manager::get(|tm| tm.get_default()),
                        scale, scale, scale);
            self.objects.push(res.clone());

            res
        })
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
            let tex  = textures_manager::get(|tm| tm.get_default());
            let geom = self.geometries.find(&~"cube").unwrap();
            Object::new(
                geom.clone(),
                1.0, 1.0, 1.0,
                tex,
                wx, wy, wz)
        };

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
            let tex  = textures_manager::get(|tm| tm.get_default());
            let geom = self.geometries.find(&~"sphere").unwrap();
            Object::new(
                geom.clone(),
                1.0, 1.0, 1.0,
                tex,
                r / 0.5, r / 0.5, r / 0.5)
        };

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
            let tex  = textures_manager::get(|tm| tm.get_default());
            let geom = self.geometries.find(&~"cone").unwrap();
            Object::new(
                geom.clone(),
                1.0, 1.0, 1.0,
                tex,
                r / 0.5, h, r / 0.5)
        };

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
            let tex  = textures_manager::get(|tm| tm.get_default());
            let geom = self.geometries.find(&~"cylinder").unwrap();
            Object::new(
                geom.clone(),
                1.0, 1.0, 1.0,
                tex,
                r / 0.5, h, r / 0.5)
        };

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
            let tex  = textures_manager::get(|tm| tm.get_default());
            let geom = self.geometries.find(&~"capsule").unwrap();
            Object::new(
                geom.clone(),
                1.0, 1.0, 1.0,
                tex,
                r / 0.5, h, r / 0.5)
        };

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
    ///   * `shared` - whether or not the mesh datas can be shared. Shared datas are slower to
    ///   update.
    pub fn add_quad(&mut self,
                     w:        f32,
                     h:        f32,
                     wsubdivs: uint,
                     hsubdivs: uint,
                     shared:   bool)
                     -> Object {
        assert!(wsubdivs > 0 && hsubdivs > 0, "The number of subdivisions cannot be zero");

        let wstep    = w / (wsubdivs as GLfloat);
        let hstep    = h / (hsubdivs as GLfloat);
        let wtexstep = 1.0 / (wsubdivs as GLfloat);
        let htexstep = 1.0 / (hsubdivs as GLfloat);
        let cw       = w / 2.0;
        let ch       = h / 2.0;

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
        ((hsubdivs + 1) * (wsubdivs + 1)).times(|| {
            { normals.push(Vec3::new(1.0, 0.0, 0.0)) }
        });

        // create triangles
        fn dl_triangle(i: u32, j: u32, ws: u32) -> Vec3<GLuint> {
            Vec3::new((i + 1) * ws + j, i * ws + j, (i + 1) * ws + j + 1)
        }

        fn ur_triangle(i: u32, j: u32, ws: u32) -> Vec3<GLuint> {
            Vec3::new(i * ws + j, i * ws + (j + 1), (i + 1) * ws + j + 1)
        }

        for i in range(0u, hsubdivs) {
            for j in range(0u, wsubdivs) {
                // build two triangles...
                triangles.push(dl_triangle(i as GLuint, j as GLuint, (wsubdivs + 1) as GLuint));
                triangles.push(ur_triangle(i as GLuint, j as GLuint, (wsubdivs + 1) as GLuint));
            }
        }

        let mesh = Mesh::new(StorageLocation::new(vertices, shared),
                             StorageLocation::new(triangles, shared),
                             Some(StorageLocation::new(normals, shared)),
                             Some(StorageLocation::new(tex_coords, shared)),
                             true);

        // FIXME: this weird block indirection are here because of Rust issue #6248
        let res = {
            let tex  = textures_manager::get(|tm| tm.get_default());
            Object::new(
                Rc::new(RefCell::new(mesh)),
                1.0, 1.0, 1.0,
                tex,
                1.0, 1.0, 1.0)
        };

        self.objects.push(res.clone());

        res
    }

    #[doc(hidden)]
    pub fn add_texture(&mut self, path: &Path, name: &str) -> Rc<Texture> {
        textures_manager::get(|tm| tm.add(path, name))
    }

    /// Converts a 3d point to 2d screen coordinates.
    pub fn project(&self, world_coord: &Vec3<f32>) -> Vec2<f32> {
        let h_world_coord = na::to_homogeneous(world_coord);
        let h_normalized_coord = self.camera.transformation() * h_world_coord;

        let normalized_coord: Vec3<f32> = na::from_homogeneous(&h_normalized_coord);

        let (w, h) = self.window.get_size();

        Vec2::new(
            (1.0 + normalized_coord.x) * (w as f32) / 2.0,
            (1.0 + normalized_coord.y) * (h as f32) / 2.0)
    }

    /// Converts a point in 2d screen coordinates to a ray (a 3d position and a direction).
    pub fn unproject(&self, window_coord: &Vec2<f32>) -> (Vec3<f32>, Vec3<f32>) {
        let (w, h) = self.window.get_size();

        let normalized_coord = Vec2::new(
            2.0 * window_coord.x  / (w as f32) - 1.0,
            2.0 * -window_coord.y / (h as f32) + 1.0);

        let normalized_begin = Vec4::new(normalized_coord.x, normalized_coord.y, -1.0, 1.0);
        let normalized_end   = Vec4::new(normalized_coord.x, normalized_coord.y, 1.0, 1.0);

        let cam = self.camera.inv_transformation();

        let h_unprojected_begin = cam * normalized_begin;
        let h_unprojected_end   = cam * normalized_end;

        let unprojected_begin: Vec3<f32> = na::from_homogeneous(&h_unprojected_begin);
        let unprojected_end: Vec3<f32>   = na::from_homogeneous(&h_unprojected_end);

        (unprojected_begin, na::normalize(&(unprojected_end - unprojected_begin)))
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

    /// Poll events and pass them to a user-defined function. If the function returns `true`, the
    /// default engine event handler (camera, framebuffer size, etc.) is executed, if it returns
    /// `false`, the default engine event handler is not executed. Return `false` if you want to
    /// override the default engine behaviour.
    #[inline(always)]
    pub fn poll_events(&mut self, events_handler: |&mut Window, &event::Event| -> bool) {
        // redispatch them
        let events = self.events.clone();
        events.read(|es| {
            for e in es.iter() {
                if events_handler(self, e) {
                    match *e {
                        event::KeyReleased(key) => {
                            if key == glfw::KeyEscape {
                                self.close();
                                continue
                            }
                        },
                        event::FramebufferSize(w, h) => {
                            self.update_viewport(w, h);
                        },
                        _ => { }
                    }

                    self.camera.handle_event(&self.window, e);
                }
            }
        });

        // clear the events collector
        self.events.write(|c| c.clear());
    }

    /// Starts an infinite loop polling events, calling an user-defined callback, and drawing the
    /// scene.
    pub fn render_loop(&mut self, callback: |&mut Window| -> ()) {
        let mut timer = Timer::new().unwrap();
        let mut curr  = time::precise_time_ns();

        while !self.window.should_close() {
            // collect events
            glfw::poll_events();

            callback(self);

            self.poll_events(|_, _| true);

            self.draw(&mut curr, &mut timer)
        }
    }

    /// Sets the light mode. Only one light is supported.
    pub fn set_light(&mut self, pos: Light) {
        match pos {
            Absolute(p)   => self.set_light_pos(&p),
            StickToCamera => {
                let camera_pos = self.camera.eye();
                self.set_light_pos(&camera_pos)
            }
        }

        self.light_mode = pos;
    }

    fn set_light_pos(&mut self, pos: &Vec3<GLfloat>) {
        self.shaders_manager.select(ObjectShader);
        verify!(gl::Uniform3f(self.shaders_manager.object_context().light, pos.x, pos.y, pos.z));
        // FIXME: select the LinesShader too ?
    }

    // FIXME /// The camera used to render the scene.
    // FIXME pub fn camera(&self) -> &Camera {
    // FIXME     self.camera.clone()
    // FIXME }

    /// Opens a window and hide it. Once the window is created and before any event pooling, a
    /// user-defined callback is called once.
    ///
    /// This method contains an infinite loop and returns when the window is closed.
    ///
    /// # Arguments
    ///   * `title` - the window title
    ///   * `callback` - a callback called once the window has been created
    pub fn spawn_hidden(title: &str, callback: proc(&mut Window)) {
        Window::do_spawn(title.to_owned(), true, DEFAULT_WIDTH, DEFAULT_HEIGHT, callback)
    }

    /// Opens a window. Once the window is created and before any event pooling, a user-defined
    /// callback is called once.
    ///
    /// This method contains an infinite loop and returns when the window is closed.
    ///
    /// # Arguments
    ///   * `title` - the window title
    ///   * `callback` - a callback called once the window has been created
    pub fn spawn(title: &str, callback: proc(&mut Window)) {
        Window::do_spawn(title.to_owned(), false, DEFAULT_WIDTH, DEFAULT_HEIGHT, callback)
    }

    /// spawn with window size
    pub fn spawn_size(title: &str, width: u32, height: u32, callback: proc(&mut Window)) {
        Window::do_spawn(title.to_owned(), false, width, height, callback)
    }

    fn do_spawn(title: ~str, hide: bool, width: u32, height: u32, callback: proc(&mut Window)) {
        glfw::set_error_callback(~ErrorCallback);

        do glfw::start {
            let window = glfw::Window::create(width, height, title, glfw::Windowed)
                         .expect("Unable to open a glfw window.");

            window.make_context_current();

            verify!(gl::load_with(glfw::get_proc_address));
            init_gl();

            // FIXME: load that iff the user really uses post-processing
            let mut shaders  = ShadersManager::new();
            shaders.select(ObjectShader);
            let builtins     = loader::load(shaders.object_context());
            let mut camera   = ArcBall::new(-Vec3::z(), Zero::zero());

            let mut usr_window = Window {
                max_ms_per_frame:      None,
                window:                window,
                objects:               ~[],
                camera:                &mut camera as &mut Camera,
                light_mode:            Absolute(Vec3::new(0.0, 10.0, 0.0)),
                wireframe_mode:        false,
                geometries:            builtins,
                background:            Vec3::new(0.0, 0.0, 0.0),
                lines_manager:         LinesManager::new(),
                shaders_manager:       shaders,
                post_processing:       None,
                post_process_render_target: FramebuffersManager::new_render_target(width as uint, height as uint),
                framebuffers_manager:  FramebuffersManager::new(),
                events:                RWArc::new(~[])
            };

            // setup callbacks
            let collector = usr_window.events.clone();
            usr_window.window.set_framebuffer_size_callback(~FramebufferSizeCallback::new(collector));

            let collector = usr_window.events.clone();
            usr_window.window.set_key_callback(~KeyCallback::new(collector));

            let collector = usr_window.events.clone();
            usr_window.window.set_mouse_button_callback(~MouseButtonCallback::new(collector));

            let collector = usr_window.events.clone();
            usr_window.window.set_cursor_pos_callback(~CursorPosCallback::new(collector));

            let collector = usr_window.events.clone();
            usr_window.window.set_scroll_callback(~ScrollCallback::new(collector));

            let (w, h) = usr_window.window.get_size();
            usr_window.camera.handle_event(
                &usr_window.window,
                &event::FramebufferSize(w as f32, h as f32));

            if hide {
                usr_window.window.hide()
            }

            // usr_window.framebuffer_size_callback(DEFAULT_WIDTH, DEFAULT_HEIGHT);
            usr_window.set_light(usr_window.light_mode);

            callback(&mut usr_window);
        }
    }

    fn draw(&mut self, curr: &mut u64, timer: &mut Timer) {
        self.camera.update(&self.window);

        match self.light_mode {
            StickToCamera => self.set_light(StickToCamera),
            _             => { }
        }

        if self.post_processing.is_some() {
            // if we need post-processing, render to our own frame buffer
            self.framebuffers_manager.select(&self.post_process_render_target);
        }
        else {
            self.framebuffers_manager.select(&FramebuffersManager::screen());
        }

        // TODO: change to pass_iter when I learn the lingo
        for pass in range(0u, self.camera.num_passes()) {
            self.camera.start_pass(pass, &self.window);

            self.shaders_manager.select(LinesShader);
            let view_location2 = self.shaders_manager.lines_context().view;
            self.camera.upload(pass, view_location2);

            self.shaders_manager.select(ObjectShader);
            let view_location1 = self.shaders_manager.object_context().view;
            self.camera.upload(pass, view_location1);

            self.render_scene();
        }
        self.camera.render_complete(&self.window);

        let w = self.width();
        let h = self.height();
        let (znear, zfar) = self.camera.clip_planes();

        match self.post_processing {
            Some(ref mut p) => {
                // remove the wireframe mode
                if self.wireframe_mode {
                    verify!(gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL));
                }

                // switch back to the screen framebuffer …
                self.framebuffers_manager.select(&FramebuffersManager::screen());
                // … and execute the post-process
                // FIXME: use the real time value instead of 0.016!
                p.update(0.016, w, h, znear, zfar);
                p.draw(&mut self.shaders_manager, &self.post_process_render_target);
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


    fn update_viewport(&mut self, w: f32, h: f32) {
        // Update the viewport
        verify!(gl::Scissor(0 as i32, 0 as i32, w as i32, h as i32));
        FramebuffersManager::screen().resize(w, h);
        self.post_process_render_target.resize(w, h);
    }
}

fn init_gl() {
    /*
     * Misc configurations
     */
    verify!(gl::FrontFace(gl::CCW));
    verify!(gl::Enable(gl::DEPTH_TEST));
    verify!(gl::Enable(gl::SCISSOR_TEST));
    verify!(gl::DepthFunc(gl::LEQUAL));
}

//
// Cursor Pos Callback
//
struct ScrollCallback {
    collector: RWArc<~[event::Event]>
}

impl ScrollCallback {
    pub fn new(collector: RWArc<~[event::Event]>) -> ScrollCallback {
        ScrollCallback {
            collector: collector
        }
    }
}

impl glfw::ScrollCallback for ScrollCallback {
    fn call(&self, _: &glfw::Window, x: f64, y: f64) {
        self.collector.write(|c| c.push(event::Scroll(x as f32, y as f32)))
    }
}

//
// Cursor Pos Callback
//
struct CursorPosCallback {
    collector: RWArc<~[event::Event]>
}

impl CursorPosCallback {
    pub fn new(collector: RWArc<~[event::Event]>) -> CursorPosCallback {
        CursorPosCallback {
            collector: collector
        }
    }
}

impl glfw::CursorPosCallback for CursorPosCallback {
    fn call(&self, _: &glfw::Window, x: f64, y: f64) {
        self.collector.write(|c| c.push(event::CursorPos(x as f32, y as f32)))
    }
}

//
// Mouse Button Callback
//
struct MouseButtonCallback {
    collector: RWArc<~[event::Event]>
}

impl MouseButtonCallback {
    pub fn new(collector: RWArc<~[event::Event]>) -> MouseButtonCallback {
        MouseButtonCallback {
            collector: collector
        }
    }
}

impl glfw::MouseButtonCallback for MouseButtonCallback {
    fn call(&self,
            _:      &glfw::Window,
            button: glfw::MouseButton,
            action: glfw::Action,
            mods:   glfw::Modifiers) {
        if action == glfw::Press {
            self.collector.write(|c| c.push(event::ButtonPressed(button, mods)))
        }
        else {
            self.collector.write(|c| c.push(event::ButtonReleased(button, mods)))
        }
    }
}

//
// Key callback
//
struct KeyCallback {
    collector: RWArc<~[event::Event]>
}

impl KeyCallback {
    pub fn new(collector: RWArc<~[event::Event]>) -> KeyCallback {
        KeyCallback {
            collector: collector
        }
    }
}

impl glfw::KeyCallback for KeyCallback {
    fn call(&self,
            _:      &glfw::Window,
            key:    glfw::Key,
            _:      libc::c_int,
            action: glfw::Action,
            _:      glfw::Modifiers) {
        if action == glfw::Press {
            self.collector.write(|c| c.push(event::KeyPressed(key)))
        }
        else {
            self.collector.write(|c| c.push(event::KeyReleased(key)))
        }
    }
}

//
// Framebuffer callback
//
struct FramebufferSizeCallback {
    collector: RWArc<~[event::Event]>
}

impl FramebufferSizeCallback {
    pub fn new(collector: RWArc<~[event::Event]>) -> FramebufferSizeCallback {
        FramebufferSizeCallback {
            collector: collector
        }
    }
}

impl glfw::FramebufferSizeCallback for FramebufferSizeCallback {
    fn call(&self, _: &glfw::Window, w: i32, h: i32) {
        self.collector.write(|c| c.push(event::FramebufferSize(w as f32, h as f32)))
    }
}

//
// Error callback
//
struct ErrorCallback;
impl glfw::ErrorCallback for ErrorCallback {
    fn call(&self, _: glfw::Error, description: ~str) {
        println!("Kiss3d Error: {}", description);
    }
}
