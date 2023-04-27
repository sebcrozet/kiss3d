//! The kiss3d window.
/*
 * FIXME: this file is too big. Some heavy refactoring need to be done here.
 */
use std::cell::RefCell;
use std::iter::repeat;
use std::path::Path;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use na::{Point2, Point3, Vector2, Vector3};

use crate::camera::{ArcBall, Camera};
use crate::context::Context;
use crate::event::{Action, EventManager, Key, WindowEvent};
use crate::light::Light;
use crate::planar_camera::{FixedView, PlanarCamera};
use crate::planar_line_renderer::PlanarLineRenderer;
use crate::post_processing::PostProcessingEffect;
#[cfg(feature = "conrod")]
use crate::renderer::ConrodRenderer;
use crate::renderer::{LineRenderer, PointRenderer, Renderer};
use crate::resource::{
    FramebufferManager, Mesh, PlanarMesh, RenderTarget, Texture, TextureManager,
};
use crate::scene::{PlanarSceneNode, SceneNode};
use crate::text::{Font, TextRenderer};
use crate::verify;
use crate::window::canvas::CanvasSetup;
use crate::window::{Canvas, State};
use image::imageops;
use image::{GenericImage, Pixel};
use image::{ImageBuffer, Rgb};
use ncollide3d::procedural::TriMesh;

#[cfg(feature = "conrod")]
use std::collections::HashMap;

use super::window_cache::WindowCache;

static DEFAULT_WIDTH: u32 = 800u32;
static DEFAULT_HEIGHT: u32 = 600u32;

#[cfg(feature = "conrod")]
struct ConrodContext {
    renderer: ConrodRenderer,
    textures: conrod::image::Map<(Rc<Texture>, (u32, u32))>,
    texture_ids: HashMap<String, conrod::image::Id>,
}

#[cfg(feature = "conrod")]
impl ConrodContext {
    fn new(width: f64, height: f64) -> Self {
        Self {
            renderer: ConrodRenderer::new(width, height),
            textures: conrod::image::Map::new(),
            texture_ids: HashMap::new(),
        }
    }
}

/// Structure representing a window and a 3D scene.
///
/// This is the main interface with the 3d engine.
pub struct Window {
    events: Rc<Receiver<WindowEvent>>,
    unhandled_events: Rc<RefCell<Vec<WindowEvent>>>,
    min_dur_per_frame: Option<Duration>,
    scene: SceneNode,
    scene2: PlanarSceneNode,
    light_mode: Light, // FIXME: move that to the scene graph
    background: Vector3<f32>,
    line_renderer: LineRenderer,
    planar_line_renderer: PlanarLineRenderer,
    point_renderer: PointRenderer,
    text_renderer: TextRenderer,
    framebuffer_manager: FramebufferManager,
    post_process_render_target: RenderTarget,
    #[cfg(not(target_arch = "wasm32"))]
    curr_time: std::time::Instant,
    planar_camera: Rc<RefCell<FixedView>>,
    camera: Rc<RefCell<ArcBall>>,
    should_close: bool,
    #[cfg(feature = "conrod")]
    conrod_context: ConrodContext,
    canvas: Canvas,
}

impl Drop for Window {
    fn drop(&mut self) {
        WindowCache::clear();
    }
}

impl Window {
    /// Indicates whether this window should be closed.
    #[inline]
    pub fn should_close(&self) -> bool {
        self.should_close
    }

    /// The window width.
    #[inline]
    pub fn width(&self) -> u32 {
        self.canvas.size().0
    }

    /// The window height.
    #[inline]
    pub fn height(&self) -> u32 {
        self.canvas.size().1
    }

    /// The size of the window.
    #[inline]
    pub fn size(&self) -> Vector2<u32> {
        let (w, h) = self.canvas.size();
        Vector2::new(w, h)
    }

    /// Sets the maximum number of frames per second. Cannot be 0. `None` means there is no limit.
    #[inline]
    pub fn set_framerate_limit(&mut self, fps: Option<u64>) {
        self.min_dur_per_frame = fps.map(|f| {
            assert!(f != 0);
            Duration::from_millis(1000 / f)
        })
    }

    /// Set window title
    pub fn set_title(&mut self, title: &str) {
        self.canvas.set_title(title)
    }

    /// Set the window icon. On wasm this does nothing.
    ///
    /// ```rust,should_panic
    /// # extern crate kiss3d;
    /// # extern crate image;
    /// # use kiss3d::window::Window;
    ///
    /// # fn main() -> Result<(), image::ImageError> {
    /// #    let mut window = Window::new("");
    /// window.set_icon(image::open("foo.ico")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_icon(&mut self, icon: impl GenericImage<Pixel = impl Pixel<Subpixel = u8>>) {
        self.canvas.set_icon(icon)
    }

    /// Set the cursor grabbing behaviour.
    ///
    /// If cursor grabbing is on, the cursor is prevented from leaving the window.
    /// Does nothing on web platforms.
    pub fn set_cursor_grab(&self, grab: bool) {
        self.canvas.set_cursor_grab(grab);
    }

    #[inline]
    /// Set the cursor position.
    pub fn set_cursor_position(&self, x: f64, y: f64) {
        self.canvas.set_cursor_position(x, y);
    }

    #[inline]
    /// Toggle the cursor visibility.
    pub fn hide_cursor(&self, hide: bool) {
        self.canvas.hide_cursor(hide);
    }

    /// Closes the window.
    #[inline]
    pub fn close(&mut self) {
        self.should_close = true;
    }

    /// Hides the window, without closing it. Use `show` to make it visible again.
    #[inline]
    pub fn hide(&mut self) {
        self.canvas.hide()
    }

    /// Makes the window visible. Use `hide` to hide it.
    #[inline]
    pub fn show(&mut self) {
        self.canvas.show()
    }

    /// Sets the background color.
    #[inline]
    pub fn set_background_color(&mut self, r: f32, g: f32, b: f32) {
        self.background.x = r;
        self.background.y = g;
        self.background.z = b;
    }

    /// Set the size of all points that will be rendered.
    ///
    /// Unfortunately, not all point sizes are supported by all graphics drivers.
    #[inline]
    pub fn set_point_size(&mut self, pt_size: f32) {
        self.point_renderer.set_point_size(pt_size);
    }

    /// Set the width of all lines that will be rendered.
    #[inline]
    pub fn set_line_width(&mut self, line_width: f32) {
        self.line_renderer.set_line_width(line_width);
        self.planar_line_renderer.set_line_width(line_width);
    }

    /// Adds a 3D line to be drawn during the next render.
    ///
    /// The line is being drawn only during the next frame after this call.
    /// Therefore, this call must be executed at as many frames as you want it to remain visible.
    #[inline]
    pub fn draw_line(&mut self, a: &Point3<f32>, b: &Point3<f32>, color: &Point3<f32>) {
        self.line_renderer.draw_line(*a, *b, *color);
    }

    /// Draws a 2D line to be drawn during the next render.
    ///
    /// The line is being drawn only during the next frame after this call.
    /// Therefore, this call must be executed at as many frames as you want it to remain visible.
    #[inline]
    pub fn draw_planar_line(&mut self, a: &Point2<f32>, b: &Point2<f32>, color: &Point3<f32>) {
        self.planar_line_renderer.draw_line(*a, *b, *color);
    }

    /// Adds a point to be drawn during the next frame.
    #[inline]
    pub fn draw_point(&mut self, pt: &Point3<f32>, color: &Point3<f32>) {
        self.point_renderer.draw_point(*pt, *color);
    }

    /// Adds a string to be drawn during the next frame.
    #[inline]
    pub fn draw_text(
        &mut self,
        text: &str,
        pos: &Point2<f32>,
        scale: f32,
        font: &Rc<Font>,
        color: &Point3<f32>,
    ) {
        self.text_renderer.draw_text(text, pos, scale, font, color);
    }

    /// Removes an object from the scene.
    #[deprecated(note = "Use `remove_node` instead.")]
    pub fn remove(&mut self, sn: &mut SceneNode) {
        self.remove_node(sn)
    }

    /// Removes an object from the scene.
    pub fn remove_node(&mut self, sn: &mut SceneNode) {
        sn.unlink()
    }

    /// Removes a 2D object from the scene.
    pub fn remove_planar_node(&mut self, sn: &mut PlanarSceneNode) {
        sn.unlink()
    }

    /// Adds a group to the scene.
    ///
    /// A group is a node not containing any object.
    pub fn add_group(&mut self) -> SceneNode {
        self.scene.add_group()
    }

    /// Adds a 2D group to the scene.
    ///
    /// A group is a node not containing any object.
    pub fn add_planar_group(&mut self) -> PlanarSceneNode {
        self.scene2.add_group()
    }

    /// Adds an obj model to the scene.
    ///
    /// # Arguments
    /// * `path`  - relative path to the obj file.
    /// * `scale` - scale to apply to the model.
    pub fn add_obj(&mut self, path: &Path, mtl_dir: &Path, scale: Vector3<f32>) -> SceneNode {
        self.scene.add_obj(path, mtl_dir, scale)
    }

    /// Adds an unnamed mesh to the scene.
    pub fn add_mesh(&mut self, mesh: Rc<RefCell<Mesh>>, scale: Vector3<f32>) -> SceneNode {
        self.scene.add_mesh(mesh, scale)
    }

    /// Adds an unnamed planar mesh to the scene.
    pub fn add_planar_mesh(
        &mut self,
        mesh: Rc<RefCell<PlanarMesh>>,
        scale: Vector2<f32>,
    ) -> PlanarSceneNode {
        self.scene2.add_mesh(mesh, scale)
    }

    /// Creates and adds a new object using the geometry generated by a given procedural generator.
    /// Creates and adds a new object using a mesh descriptor.
    pub fn add_trimesh(&mut self, descr: TriMesh<f32>, scale: Vector3<f32>) -> SceneNode {
        self.scene.add_trimesh(descr, scale)
    }

    /// Creates and adds a new object using the geometry registered as `geometry_name`.
    pub fn add_geom_with_name(
        &mut self,
        geometry_name: &str,
        scale: Vector3<f32>,
    ) -> Option<SceneNode> {
        self.scene.add_geom_with_name(geometry_name, scale)
    }

    /// Adds a cube to the scene. The cube is initially axis-aligned and centered at (0, 0, 0).
    ///
    /// # Arguments
    /// * `wx` - the cube extent along the x axis
    /// * `wy` - the cube extent along the y axis
    /// * `wz` - the cube extent along the z axis
    pub fn add_cube(&mut self, wx: f32, wy: f32, wz: f32) -> SceneNode {
        self.scene.add_cube(wx, wy, wz)
    }

    /// Adds a sphere to the scene. The sphere is initially centered at (0, 0, 0).
    ///
    /// # Arguments
    /// * `r` - the sphere radius
    pub fn add_sphere(&mut self, r: f32) -> SceneNode {
        self.scene.add_sphere(r)
    }

    /// Adds a cone to the scene. The cone is initially centered at (0, 0, 0) and points toward the
    /// positive `y` axis.
    ///
    /// # Arguments
    /// * `h` - the cone height
    /// * `r` - the cone base radius
    pub fn add_cone(&mut self, r: f32, h: f32) -> SceneNode {
        self.scene.add_cone(r, h)
    }

    /// Adds a cylinder to the scene. The cylinder is initially centered at (0, 0, 0) and has its
    /// principal axis aligned with the `y` axis.
    ///
    /// # Arguments
    /// * `h` - the cylinder height
    /// * `r` - the cylinder base radius
    pub fn add_cylinder(&mut self, r: f32, h: f32) -> SceneNode {
        self.scene.add_cylinder(r, h)
    }

    /// Adds a capsule to the scene. The capsule is initially centered at (0, 0, 0) and has its
    /// principal axis aligned with the `y` axis.
    ///
    /// # Arguments
    /// * `r` - the capsule caps radius
    /// * `h` - the capsule height
    pub fn add_capsule(&mut self, r: f32, h: f32) -> SceneNode {
        self.scene.add_capsule(r, h)
    }

    /// Adds a 2D capsule to the scene. The capsule is initially centered at (0, 0) and has its
    /// principal axis aligned with the `y` axis.
    ///
    /// # Arguments
    /// * `r` - the capsule caps radius
    /// * `h` - the capsule height
    pub fn add_planar_capsule(&mut self, r: f32, h: f32) -> PlanarSceneNode {
        self.scene2.add_capsule(r, h)
    }

    /// Adds a double-sided quad to the scene. The quad is initially centered at (0, 0, 0). The
    /// quad itself is composed of a user-defined number of triangles regularly spaced on a grid.
    /// This is the main way to draw height maps.
    ///
    /// # Arguments
    /// * `w` - the quad width.
    /// * `h` - the quad height.
    /// * `wsubdivs` - number of horizontal subdivisions. This correspond to the number of squares
    /// which will be placed horizontally on each line. Must not be `0`.
    /// * `hsubdivs` - number of vertical subdivisions. This correspond to the number of squares
    /// which will be placed vertically on each line. Must not be `0`.
    /// update.
    pub fn add_quad(&mut self, w: f32, h: f32, usubdivs: usize, vsubdivs: usize) -> SceneNode {
        self.scene.add_quad(w, h, usubdivs, vsubdivs)
    }

    /// Adds a double-sided quad with the specified vertices.
    pub fn add_quad_with_vertices(
        &mut self,
        vertices: &[Point3<f32>],
        nhpoints: usize,
        nvpoints: usize,
    ) -> SceneNode {
        self.scene
            .add_quad_with_vertices(vertices, nhpoints, nvpoints)
    }

    /// Load a texture from a file and return a reference to it.
    pub fn add_texture(&mut self, path: &Path, name: &str) -> Rc<Texture> {
        TextureManager::get_global_manager(|tm| tm.add(path, name))
    }

    /// Adds a rectangle to the scene. The rectangle is initially axis-aligned and centered at (0, 0, 0).
    ///
    /// # Arguments
    /// * `wx` - the cube extent along the x axis
    /// * `wy` - the cube extent along the y axis
    pub fn add_rectangle(&mut self, wx: f32, wy: f32) -> PlanarSceneNode {
        self.scene2.add_rectangle(wx, wy)
    }

    /// Adds a circle to the scene. The circle is initially centered at (0, 0, 0).
    ///
    /// # Arguments
    /// * `r` - the circle radius
    pub fn add_circle(&mut self, r: f32) -> PlanarSceneNode {
        self.scene2.add_circle(r)
    }

    /// Adds a convex polygon to the scene.
    ///
    /// # Arguments
    /// * `r` - the circle radius
    pub fn add_convex_polygon(
        &mut self,
        polygon: Vec<Point2<f32>>,
        scale: Vector2<f32>,
    ) -> PlanarSceneNode {
        self.scene2.add_convex_polygon(polygon, scale)
    }

    /// Returns whether this window is closed or not.
    pub fn is_closed(&self) -> bool {
        false // FIXME
    }

    /// The scale factor of this screen.
    pub fn scale_factor(&self) -> f64 {
        self.canvas.scale_factor()
    }

    /// Sets the light mode. Only one light is supported.
    pub fn set_light(&mut self, pos: Light) {
        self.light_mode = pos;
    }

    /// Retrieve a mutable reference to the UI based on Conrod.
    #[cfg(feature = "conrod")]
    pub fn conrod_ui_mut(&mut self) -> &mut conrod::Ui {
        self.conrod_context.renderer.ui_mut()
    }

    /// Attributes a conrod ID to the given texture and returns it if it exists.
    #[cfg(feature = "conrod")]
    pub fn conrod_texture_id(&mut self, name: &str) -> Option<conrod::image::Id> {
        let tex = TextureManager::get_global_manager(|tm| tm.get_with_size(name))?;
        let textures = &mut self.conrod_context.textures;
        Some(
            *self
                .conrod_context
                .texture_ids
                .entry(name.to_string())
                .or_insert_with(|| textures.insert(tex)),
        )
    }

    /// Retrieve a reference to the UI based on Conrod.
    #[cfg(feature = "conrod")]
    pub fn conrod_ui(&self) -> &conrod::Ui {
        self.conrod_context.renderer.ui()
    }

    /// Returns `true` if the mouse is currently interacting with a Conrod widget.
    #[cfg(feature = "conrod")]
    pub fn is_conrod_ui_capturing_mouse(&self) -> bool {
        let ui = self.conrod_ui();
        let state = &ui.global_input().current;
        let window_id = Some(ui.window);

        state.widget_capturing_mouse.is_some() && state.widget_capturing_mouse != window_id
    }

    /// Returns `true` if the keyboard is currently interacting with a Conrod widget.
    #[cfg(feature = "conrod")]
    pub fn is_conrod_ui_capturing_keyboard(&self) -> bool {
        let ui = self.conrod_ui();
        let state = &ui.global_input().current;
        let window_id = Some(ui.window);

        state.widget_capturing_keyboard.is_some() && state.widget_capturing_keyboard != window_id
    }

    /// Opens a window, hide it then calls a user-defined procedure.
    ///
    /// # Arguments
    /// * `title` - the window title
    pub fn new_hidden(title: &str) -> Window {
        Window::do_new(title, true, DEFAULT_WIDTH, DEFAULT_HEIGHT, None)
    }

    /// Opens a window then calls a user-defined procedure.
    ///
    /// # Arguments
    /// * `title` - the window title
    pub fn new(title: &str) -> Window {
        Window::do_new(title, false, DEFAULT_WIDTH, DEFAULT_HEIGHT, None)
    }

    /// Opens a window with a custom size then calls a user-defined procedure.
    ///
    /// # Arguments
    /// * `title` - the window title.
    /// * `width` - the window width.
    /// * `height` - the window height.
    pub fn new_with_size(title: &str, width: u32, height: u32) -> Window {
        Window::do_new(title, false, width, height, None)
    }

    /// Opens a window with custom options for vsync and AA.
    pub fn new_with_setup(title: &str, width: u32, height: u32, setup: CanvasSetup) -> Window {
        Window::do_new(title, false, width, height, Some(setup))
    }

    // FIXME: make this pub?
    fn do_new(
        title: &str,
        hide: bool,
        width: u32,
        height: u32,
        setup: Option<CanvasSetup>,
    ) -> Window {
        let (event_send, event_receive) = mpsc::channel();
        let canvas = Canvas::open(title, hide, width, height, setup, event_send);

        init_gl();
        WindowCache::populate();

        let mut usr_window = Window {
            should_close: false,
            min_dur_per_frame: None,
            canvas,
            events: Rc::new(event_receive),
            unhandled_events: Rc::new(RefCell::new(Vec::new())),
            scene: SceneNode::new_empty(),
            scene2: PlanarSceneNode::new_empty(),
            light_mode: Light::Absolute(Point3::new(0.0, 10.0, 0.0)),
            background: Vector3::new(0.0, 0.0, 0.0),
            line_renderer: LineRenderer::new(),
            planar_line_renderer: PlanarLineRenderer::new(),
            point_renderer: PointRenderer::new(),
            text_renderer: TextRenderer::new(),
            #[cfg(feature = "conrod")]
            conrod_context: ConrodContext::new(width as f64, height as f64),
            post_process_render_target: FramebufferManager::new_render_target(
                width as usize,
                height as usize,
                true,
            ),
            framebuffer_manager: FramebufferManager::new(),
            #[cfg(not(target_arch = "wasm32"))]
            curr_time: std::time::Instant::now(),
            planar_camera: Rc::new(RefCell::new(FixedView::new())),
            camera: Rc::new(RefCell::new(ArcBall::new(
                Point3::new(0.0f32, 0.0, -1.0),
                Point3::origin(),
            ))),
        };

        if hide {
            usr_window.canvas.hide()
        }

        // usr_window.framebuffer_size_callback(DEFAULT_WIDTH, DEFAULT_HEIGHT);
        let light = usr_window.light_mode.clone();
        usr_window.set_light(light);

        usr_window
    }

    /// Reference to the scene associated with this window.
    #[inline]
    pub fn scene(&self) -> &SceneNode {
        &self.scene
    }

    /// Mutable reference to the scene associated with this window.
    #[inline]
    pub fn scene_mut(&mut self) -> &mut SceneNode {
        &mut self.scene
    }

    // FIXME: give more options for the snap size and offset.
    /// Read the pixels currently displayed to the screen.
    ///
    /// # Arguments:
    /// * `out` - the output buffer. It is automatically resized.
    pub fn snap(&self, out: &mut Vec<u8>) {
        let (width, height) = self.canvas.size();
        self.snap_rect(out, 0, 0, width as usize, height as usize)
    }

    /// Read a section of pixels from the screen
    ///
    /// # Arguments:
    /// * `out` - the output buffer. It is automatically resized
    /// * `x, y, width, height` - the rectangle to capture
    pub fn snap_rect(&self, out: &mut Vec<u8>, x: usize, y: usize, width: usize, height: usize) {
        let size = (width * height * 3) as usize;

        if out.len() < size {
            let diff = size - out.len();
            out.extend(repeat(0).take(diff));
        } else {
            out.truncate(size)
        }

        // FIXME: this is _not_ the fastest way of doing this.
        let ctxt = Context::get();
        ctxt.pixel_storei(Context::PACK_ALIGNMENT, 1);
        ctxt.read_pixels(
            x as i32,
            y as i32,
            width as i32,
            height as i32,
            Context::RGB,
            Some(out),
        );
    }

    /// Get the current screen as an image
    pub fn snap_image(&self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let (width, height) = self.canvas.size();
        let mut buf = Vec::new();
        self.snap(&mut buf);
        let img_opt = ImageBuffer::from_vec(width as u32, height as u32, buf);
        let img = img_opt.expect("Buffer created from window was not big enough for image.");
        imageops::flip_vertical(&img)
    }

    /// Gets the events manager that gives access to an event iterator.
    pub fn events(&self) -> EventManager {
        EventManager::new(self.events.clone(), self.unhandled_events.clone())
    }

    /// Gets the status of a key.
    pub fn get_key(&self, key: Key) -> Action {
        self.canvas.get_key(key)
    }

    /// Gets the last known position of the mouse.
    ///
    /// The position of the mouse is automatically updated when the mouse moves over the canvas.
    pub fn cursor_pos(&self) -> Option<(f64, f64)> {
        self.canvas.cursor_pos()
    }

    #[inline]
    fn handle_events(
        &mut self,
        camera: &mut Option<&mut dyn Camera>,
        planar_camera: &mut Option<&mut dyn PlanarCamera>,
    ) {
        let unhandled_events = self.unhandled_events.clone(); // FIXME: could we avoid the clone?
        let events = self.events.clone(); // FIXME: could we avoid the clone?

        for event in unhandled_events.borrow().iter() {
            self.handle_event(camera, planar_camera, event)
        }

        for event in events.try_iter() {
            self.handle_event(camera, planar_camera, &event)
        }

        unhandled_events.borrow_mut().clear();
        self.canvas.poll_events();
    }

    fn handle_event(
        &mut self,
        camera: &mut Option<&mut dyn Camera>,
        planar_camera: &mut Option<&mut dyn PlanarCamera>,
        event: &WindowEvent,
    ) {
        match *event {
            WindowEvent::Key(Key::Escape, Action::Release, _) | WindowEvent::Close => {
                self.close();
            }
            WindowEvent::FramebufferSize(w, h) => {
                self.update_viewport(w as f32, h as f32);
            }
            _ => {}
        }

        #[cfg(feature = "conrod")]
        fn window_event_to_conrod_input(
            event: WindowEvent,
            size: Vector2<u32>,
            scale: f64,
        ) -> Option<conrod::event::Input> {
            use conrod::event::Input;
            use conrod::input::{Button, Key as CKey, Motion, MouseButton};

            let transform_coords = |x: f64, y: f64| {
                (
                    (x - size.x as f64 / 2.0) / scale,
                    -(y - size.y as f64 / 2.0) / scale,
                )
            };

            match event {
                WindowEvent::FramebufferSize(w, h) => {
                    Some(Input::Resize(w as f64 / scale, h as f64 / scale))
                }
                WindowEvent::Focus(focus) => Some(Input::Focus(focus)),
                WindowEvent::CursorPos(x, y, _) => {
                    let (x, y) = transform_coords(x, y);
                    Some(Input::Motion(Motion::MouseCursor { x, y }))
                }
                WindowEvent::Scroll(x, y, _) => Some(Input::Motion(Motion::Scroll { x, y: -y })),
                WindowEvent::MouseButton(button, action, _) => {
                    let button = match button {
                        crate::event::MouseButton::Button1 => MouseButton::Left,
                        crate::event::MouseButton::Button2 => MouseButton::Right,
                        crate::event::MouseButton::Button3 => MouseButton::Middle,
                        crate::event::MouseButton::Button4 => MouseButton::X1,
                        crate::event::MouseButton::Button5 => MouseButton::X2,
                        crate::event::MouseButton::Button6 => MouseButton::Button6,
                        crate::event::MouseButton::Button7 => MouseButton::Button7,
                        crate::event::MouseButton::Button8 => MouseButton::Button8,
                    };

                    match action {
                        Action::Press => Some(Input::Press(Button::Mouse(button))),
                        Action::Release => Some(Input::Release(Button::Mouse(button))),
                    }
                }
                WindowEvent::Key(key, action, _) => {
                    let key = match key {
                        Key::Key1 => CKey::D1,
                        Key::Key2 => CKey::D2,
                        Key::Key3 => CKey::D3,
                        Key::Key4 => CKey::D4,
                        Key::Key5 => CKey::D5,
                        Key::Key6 => CKey::D6,
                        Key::Key7 => CKey::D7,
                        Key::Key8 => CKey::D8,
                        Key::Key9 => CKey::D9,
                        Key::Key0 => CKey::D0,
                        Key::A => CKey::A,
                        Key::B => CKey::B,
                        Key::C => CKey::C,
                        Key::D => CKey::D,
                        Key::E => CKey::E,
                        Key::F => CKey::F,
                        Key::G => CKey::G,
                        Key::H => CKey::H,
                        Key::I => CKey::I,
                        Key::J => CKey::J,
                        Key::K => CKey::K,
                        Key::L => CKey::L,
                        Key::M => CKey::M,
                        Key::N => CKey::N,
                        Key::O => CKey::O,
                        Key::P => CKey::P,
                        Key::Q => CKey::Q,
                        Key::R => CKey::R,
                        Key::S => CKey::S,
                        Key::T => CKey::T,
                        Key::U => CKey::U,
                        Key::V => CKey::V,
                        Key::W => CKey::W,
                        Key::X => CKey::X,
                        Key::Y => CKey::Y,
                        Key::Z => CKey::Z,
                        Key::Escape => CKey::Escape,
                        Key::F1 => CKey::F1,
                        Key::F2 => CKey::F2,
                        Key::F3 => CKey::F3,
                        Key::F4 => CKey::F4,
                        Key::F5 => CKey::F5,
                        Key::F6 => CKey::F6,
                        Key::F7 => CKey::F7,
                        Key::F8 => CKey::F8,
                        Key::F9 => CKey::F9,
                        Key::F10 => CKey::F10,
                        Key::F11 => CKey::F11,
                        Key::F12 => CKey::F12,
                        Key::F13 => CKey::F13,
                        Key::F14 => CKey::F14,
                        Key::F15 => CKey::F15,
                        Key::F16 => CKey::F16,
                        Key::F17 => CKey::F17,
                        Key::F18 => CKey::F18,
                        Key::F19 => CKey::F19,
                        Key::F20 => CKey::F20,
                        Key::F21 => CKey::F21,
                        Key::F22 => CKey::F22,
                        Key::F23 => CKey::F23,
                        Key::F24 => CKey::F24,
                        Key::Pause => CKey::Pause,
                        Key::Insert => CKey::Insert,
                        Key::Home => CKey::Home,
                        Key::Delete => CKey::Delete,
                        Key::End => CKey::End,
                        Key::PageDown => CKey::PageDown,
                        Key::PageUp => CKey::PageUp,
                        Key::Left => CKey::Left,
                        Key::Up => CKey::Up,
                        Key::Right => CKey::Right,
                        Key::Down => CKey::Down,
                        Key::Back => CKey::Backspace,
                        Key::Return => CKey::Return,
                        Key::Space => CKey::Space,
                        Key::Caret => CKey::Caret,
                        Key::Numpad0 => CKey::NumPad0,
                        Key::Numpad1 => CKey::NumPad1,
                        Key::Numpad2 => CKey::NumPad2,
                        Key::Numpad3 => CKey::NumPad3,
                        Key::Numpad4 => CKey::NumPad4,
                        Key::Numpad5 => CKey::NumPad5,
                        Key::Numpad6 => CKey::NumPad6,
                        Key::Numpad7 => CKey::NumPad7,
                        Key::Numpad8 => CKey::NumPad8,
                        Key::Numpad9 => CKey::NumPad9,
                        Key::Add => CKey::Plus,
                        Key::At => CKey::At,
                        Key::Backslash => CKey::Backslash,
                        Key::Calculator => CKey::Calculator,
                        Key::Colon => CKey::Colon,
                        Key::Comma => CKey::Comma,
                        Key::Equals => CKey::Equals,
                        Key::LBracket => CKey::LeftBracket,
                        Key::LControl => CKey::LCtrl,
                        Key::LShift => CKey::LShift,
                        Key::Mail => CKey::Mail,
                        Key::MediaSelect => CKey::MediaSelect,
                        Key::Minus => CKey::Minus,
                        Key::Mute => CKey::Mute,
                        Key::NumpadComma => CKey::NumPadComma,
                        Key::NumpadEnter => CKey::NumPadEnter,
                        Key::NumpadEquals => CKey::NumPadEquals,
                        Key::Period => CKey::Period,
                        Key::Power => CKey::Power,
                        Key::RAlt => CKey::RAlt,
                        Key::RBracket => CKey::RightBracket,
                        Key::RControl => CKey::RCtrl,
                        Key::RShift => CKey::RShift,
                        Key::Semicolon => CKey::Semicolon,
                        Key::Slash => CKey::Slash,
                        Key::Sleep => CKey::Sleep,
                        Key::Stop => CKey::Stop,
                        Key::Tab => CKey::Tab,
                        Key::VolumeDown => CKey::VolumeDown,
                        Key::VolumeUp => CKey::VolumeUp,
                        Key::Copy => CKey::Copy,
                        Key::Paste => CKey::Paste,
                        Key::Cut => CKey::Cut,
                        _ => CKey::Unknown,
                    };

                    match action {
                        Action::Press => Some(Input::Press(Button::Keyboard(key))),
                        Action::Release => Some(Input::Release(Button::Keyboard(key))),
                    }
                }
                WindowEvent::Char(ch) => {
                    // Shamelessly taken from kiss3d_conrod/backends/conrod_winit/src/macros.rs:175.
                    let string = match ch {
                        // Ignore control characters and return ascii for Text event (like sdl2).
                        '\u{7f}' | // Delete
                        '\u{1b}' | // Escape
                        '\u{8}'  | // Backspace
                        '\r' | '\n' | '\t' => "".to_string(),
                        _ => ch.to_string()
                    };
                    Some(Input::Text(string))
                }
                _ => None,
            }
        }

        #[cfg(feature = "conrod")]
        {
            let (size, scale) = (self.size(), self.scale_factor());
            let conrod_ui = self.conrod_ui_mut();
            if let Some(input) = window_event_to_conrod_input(*event, size, scale) {
                conrod_ui.handle_event(input);
            }

            let state = &conrod_ui.global_input().current;
            let window_id = Some(conrod_ui.window);

            if event.is_keyboard_event()
                && state.widget_capturing_keyboard.is_some()
                && state.widget_capturing_keyboard != window_id
            {
                return;
            }

            if event.is_mouse_event()
                && state.widget_capturing_mouse.is_some()
                && state.widget_capturing_mouse != window_id
            {
                return;
            }
        }

        match *planar_camera {
            Some(ref mut cam) => cam.handle_event(&self.canvas, event),
            None => self.camera.borrow_mut().handle_event(&self.canvas, event),
        }

        match *camera {
            Some(ref mut cam) => cam.handle_event(&self.canvas, event),
            None => self.camera.borrow_mut().handle_event(&self.canvas, event),
        }
    }

    /// Runs the render and event loop until the window is closed.
    pub fn render_loop<S: State>(self, state: S) {
        // We have to be really careful here about drop order.
        //
        // The State may contain various OpenGL objects (for example, shaders),
        // that, when dropped, make calls to the OpenGL context.
        // Since the Window contains the OpenGL context, we must not drop it
        // until we are done dropping the state.
        //
        // Since we can't directly control the drop order of fields in a closure,
        // we instead put the relevant objects into a struct, for which the
        // drop order _is_ controllable (top-to-bottom).
        struct DropControl<S> {
            state: S,
            window: Window,
        }

        let mut dc = DropControl {
            state,
            window: self,
        };

        Canvas::render_loop(move |_| dc.window.do_render_with_state(&mut dc.state));
    }

    /// Render one frame using the specified state.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_with_state<S: State>(&mut self, state: &mut S) -> bool {
        self.do_render_with_state(state)
    }

    fn do_render_with_state<S: State>(&mut self, state: &mut S) -> bool {
        {
            let (camera, planar_camera, renderer, effect) = state.cameras_and_effect_and_renderer();
            self.should_close = !self.do_render_with(camera, planar_camera, renderer, effect);
        }

        if !self.should_close {
            state.step(self)
        }

        !self.should_close
    }

    /// Renders the scene using the default camera.
    ///
    /// Returns `false` if the window should be closed.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render(&mut self) -> bool {
        self.render_with(None, None, None)
    }

    /// Render using a specific post processing effect.
    ///
    /// Returns `false` if the window should be closed.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_with_effect(&mut self, effect: &mut (dyn PostProcessingEffect)) -> bool {
        self.render_with(None, None, Some(effect))
    }

    /// Render using a specific camera.
    ///
    /// Returns `false` if the window should be closed.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_with_camera(&mut self, camera: &mut (dyn Camera)) -> bool {
        self.render_with(Some(camera), None, None)
    }

    /// Render using a specific 2D and 3D camera.
    ///
    /// Returns `false` if the window should be closed.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_with_cameras(
        &mut self,
        camera: &mut dyn Camera,
        planar_camera: &mut dyn PlanarCamera,
    ) -> bool {
        self.render_with(Some(camera), Some(planar_camera), None)
    }

    /// Render using a specific camera and post processing effect.
    ///
    /// Returns `false` if the window should be closed.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_with_camera_and_effect(
        &mut self,
        camera: &mut dyn Camera,
        effect: &mut dyn PostProcessingEffect,
    ) -> bool {
        self.render_with(Some(camera), None, Some(effect))
    }

    /// Render using a specific 2D and 3D camera and post processing effect.
    ///
    /// Returns `false` if the window should be closed.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_with_cameras_and_effect(
        &mut self,
        camera: &mut dyn Camera,
        planar_camera: &mut dyn PlanarCamera,
        effect: &mut dyn PostProcessingEffect,
    ) -> bool {
        self.render_with(Some(camera), Some(planar_camera), Some(effect))
    }

    /// Draws the scene with the given camera and post-processing effect.
    ///
    /// Returns `false` if the window should be closed.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn render_with(
        &mut self,
        camera: Option<&mut dyn Camera>,
        planar_camera: Option<&mut dyn PlanarCamera>,
        post_processing: Option<&mut dyn PostProcessingEffect>,
    ) -> bool {
        // FIXME: for backward-compatibility, we don't accept any custom renderer here.
        self.do_render_with(camera, planar_camera, None, post_processing)
    }

    fn do_render_with(
        &mut self,
        camera: Option<&mut dyn Camera>,
        planar_camera: Option<&mut dyn PlanarCamera>,
        renderer: Option<&mut dyn Renderer>,
        post_processing: Option<&mut dyn PostProcessingEffect>,
    ) -> bool {
        let mut camera = camera;
        let mut planar_camera = planar_camera;
        self.handle_events(&mut camera, &mut planar_camera);

        let self_cam2 = self.planar_camera.clone(); // FIXME: this is ugly.
        let mut bself_cam2 = self_cam2.borrow_mut();

        let self_cam = self.camera.clone(); // FIXME: this is ugly.
        let mut bself_cam = self_cam.borrow_mut();

        match (camera, planar_camera) {
            (Some(cam), Some(cam2)) => {
                self.render_single_frame(cam, cam2, renderer, post_processing)
            }
            (None, Some(cam2)) => {
                self.render_single_frame(&mut *bself_cam, cam2, renderer, post_processing)
            }
            (Some(cam), None) => {
                self.render_single_frame(cam, &mut *bself_cam2, renderer, post_processing)
            }
            (None, None) => self.render_single_frame(
                &mut *bself_cam,
                &mut *bself_cam2,
                renderer,
                post_processing,
            ),
        }
    }

    fn render_single_frame(
        &mut self,
        camera: &mut dyn Camera,
        planar_camera: &mut dyn PlanarCamera,
        mut renderer: Option<&mut dyn Renderer>,
        mut post_processing: Option<&mut dyn PostProcessingEffect>,
    ) -> bool {
        // XXX: too bad we have to do this at each frame…
        let w = self.width();
        let h = self.height();

        planar_camera.handle_event(&self.canvas, &WindowEvent::FramebufferSize(w, h));
        camera.handle_event(&self.canvas, &WindowEvent::FramebufferSize(w, h));
        planar_camera.update(&self.canvas);
        camera.update(&self.canvas);

        if let Light::StickToCamera = self.light_mode {
            self.set_light(Light::StickToCamera)
        }

        if post_processing.is_some() {
            // if we need post-processing, render to our own frame buffer
            self.framebuffer_manager
                .select(&self.post_process_render_target);
        } else {
            self.framebuffer_manager
                .select(&FramebufferManager::screen());
        }

        for pass in 0usize..camera.num_passes() {
            camera.start_pass(pass, &self.canvas);
            self.render_scene(camera, pass);

            if let Some(ref mut renderer) = renderer {
                renderer.render(pass, camera)
            }
        }

        camera.render_complete(&self.canvas);

        self.render_planar_scene(planar_camera);

        let (znear, zfar) = camera.clip_planes();

        // FIXME: remove this completely?
        // swatch off the wireframe mode for post processing and text rendering.
        // if self.wireframe_mode {
        //     verify!(gl::PolygonMode(Context::FRONT_AND_BACK, Context::FILL));
        // }

        if let Some(ref mut p) = post_processing {
            // switch back to the screen framebuffer …
            self.framebuffer_manager
                .select(&FramebufferManager::screen());
            // … and execute the post-process
            // FIXME: use the real time value instead of 0.016!
            p.update(0.016, w as f32, h as f32, znear, zfar);
            p.draw(&self.post_process_render_target);
        }

        self.text_renderer.render(w as f32, h as f32);
        #[cfg(feature = "conrod")]
        self.conrod_context.renderer.render(
            w as f32,
            h as f32,
            self.canvas.scale_factor() as f32,
            &self.conrod_context.textures,
        );

        // We are done: swap buffers
        self.canvas.swap_buffers();

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Limit the fps if needed.
            if let Some(dur) = self.min_dur_per_frame {
                let elapsed = self.curr_time.elapsed();
                if elapsed < dur {
                    std::thread::sleep(dur - elapsed);
                }
            }

            self.curr_time = std::time::Instant::now();
        }

        // self.transparent_objects.clear();
        // self.opaque_objects.clear();

        !self.should_close()
    }

    fn render_scene(&mut self, camera: &mut dyn Camera, pass: usize) {
        let ctxt = Context::get();
        // Activate the default texture
        verify!(ctxt.active_texture(Context::TEXTURE0));
        // Clear the screen to black
        verify!(ctxt.clear_color(self.background.x, self.background.y, self.background.z, 1.0));
        verify!(ctxt.clear(Context::COLOR_BUFFER_BIT));
        verify!(ctxt.clear(Context::DEPTH_BUFFER_BIT));

        self.line_renderer.render(pass, camera);
        self.point_renderer.render(pass, camera);
        self.scene.data_mut().render(pass, camera, &self.light_mode);
    }

    fn render_planar_scene(&mut self, camera: &mut dyn PlanarCamera) {
        let ctxt = Context::get();
        // Activate the default texture
        verify!(ctxt.active_texture(Context::TEXTURE0));
        // Clear the screen to black

        if self.planar_line_renderer.needs_rendering() {
            self.planar_line_renderer.render(camera);
        }

        // if self.point_renderer2.needs_rendering() {
        //     self.point_renderer2.render(camera);
        // }

        self.scene2.data_mut().render(camera);
    }

    fn update_viewport(&mut self, w: f32, h: f32) {
        // Update the viewport
        verify!(Context::get().scissor(0, 0, w as i32, h as i32));
        FramebufferManager::screen().resize(w, h);
        self.post_process_render_target.resize(w, h);
    }
}

fn init_gl() {
    /*
     * Misc configurations
     */
    let ctxt = Context::get();
    verify!(ctxt.front_face(Context::CCW));
    verify!(ctxt.enable(Context::DEPTH_TEST));
    verify!(ctxt.enable(Context::SCISSOR_TEST));
    #[cfg(not(target_arch = "wasm32"))]
    {
        verify!(ctxt.enable(Context::PROGRAM_POINT_SIZE));
    }
    verify!(ctxt.depth_func(Context::LEQUAL));
    verify!(ctxt.front_face(Context::CCW));
    verify!(ctxt.enable(Context::CULL_FACE));
    verify!(ctxt.cull_face(Context::BACK));
}
