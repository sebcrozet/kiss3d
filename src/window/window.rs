#![allow(clippy::await_holding_refcell_ref)]

//! The kiss3d window.
/*
 * FIXME: this file is too big. Some heavy refactoring need to be done here.
 */
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use na::{Point2, Point3, Vector2, Vector3};

use crate::camera::{ArcBall, Camera};
use crate::context::Context;
use crate::event::MouseButton;
use crate::event::{Action, EventManager, Key, WindowEvent};
use crate::light::Light;
use crate::planar_camera::{PlanarCamera, PlanarFixedView};
use crate::planar_line_renderer::PlanarLineRenderer;
use crate::post_processing::PostProcessingEffect;
#[cfg(feature = "egui")]
use crate::renderer::EguiRenderer;
use crate::renderer::{LineRenderer, PointRenderer, Renderer};
use crate::resource::{
    FramebufferManager, GpuMesh, PlanarMesh, RenderTarget, Texture, TextureManager,
};
use crate::scene::{PlanarSceneNode, SceneNode};
use crate::text::{Font, TextRenderer};
use crate::verify;
use crate::window::canvas::CanvasSetup;
use crate::window::Canvas;
use image::imageops;
use image::{GenericImage, Pixel};
use image::{ImageBuffer, Rgb};
use parry3d::shape::TriMesh;

use super::window_cache::WindowCache;
use crate::procedural::RenderMesh;
#[cfg(feature = "egui")]
use egui::RawInput;
use std::sync::Arc;

static DEFAULT_WIDTH: u32 = 800u32;
static DEFAULT_HEIGHT: u32 = 600u32;

#[cfg(feature = "egui")]
struct EguiContext {
    renderer: EguiRenderer,
    raw_input: RawInput,
    #[cfg(not(target_arch = "wasm32"))]
    start_time: std::time::Instant,
}

#[cfg(feature = "egui")]
impl EguiContext {
    fn new() -> Self {
        Self {
            renderer: EguiRenderer::new(),
            raw_input: RawInput::default(),
            #[cfg(not(target_arch = "wasm32"))]
            start_time: std::time::Instant::now(),
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
    planar_camera: Rc<RefCell<PlanarFixedView>>,
    camera: Rc<RefCell<ArcBall>>,
    should_close: bool,
    #[cfg(feature = "egui")]
    egui_context: EguiContext,
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
        font: &Arc<Font>,
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
    pub fn add_mesh(&mut self, mesh: Rc<RefCell<GpuMesh>>, scale: Vector3<f32>) -> SceneNode {
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

    /// Creates and adds a new object using a mesh descriptor.
    pub fn add_trimesh(&mut self, mesh: TriMesh, scale: Vector3<f32>) -> SceneNode {
        self.scene.add_trimesh(mesh, scale)
    }

    /// Creates and adds a new object using the geometry generated by a given procedural generator.
    pub fn add_render_mesh(&mut self, mesh: RenderMesh, scale: Vector3<f32>) -> SceneNode {
        self.scene.add_render_mesh(mesh, scale)
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
    ///   which will be placed horizontally on each line. Must not be `0`.
    /// * `hsubdivs` - number of vertical subdivisions. This correspond to the number of squares
    ///   which will be placed vertically on each line. Must not be `0`.
    ///   update.
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

    /// Retrieve a mutable reference to the egui Context.
    #[cfg(feature = "egui")]
    pub fn egui_context_mut(&mut self) -> &mut egui::Context {
        self.egui_context.renderer.context_mut()
    }

    /// Retrieve a reference to the egui Context.
    #[cfg(feature = "egui")]
    pub fn egui_context(&self) -> &egui::Context {
        self.egui_context.renderer.context()
    }

    /// Returns `true` if the mouse is currently interacting with an egui widget.
    #[cfg(feature = "egui")]
    pub fn is_egui_capturing_mouse(&self) -> bool {
        self.egui_context.renderer.wants_pointer_input()
    }

    /// Returns `true` if the keyboard is currently interacting with an egui widget.
    #[cfg(feature = "egui")]
    pub fn is_egui_capturing_keyboard(&self) -> bool {
        self.egui_context.renderer.wants_keyboard_input()
    }

    /// Feed a window event to egui for processing.
    #[cfg(feature = "egui")]
    fn feed_egui_event(&mut self, event: &WindowEvent) {
        let scale_factor = self.scale_factor() as f32;

        match *event {
            WindowEvent::CursorPos(x, y, _) => {
                // Convert physical pixels to logical coordinates
                let pos = egui::Pos2::new((x as f32) / scale_factor, (y as f32) / scale_factor);
                self.egui_context
                    .raw_input
                    .events
                    .push(egui::Event::PointerMoved(pos));
            }
            WindowEvent::MouseButton(button, action, _) => {
                let button = match button {
                    crate::event::MouseButton::Button1 => egui::PointerButton::Primary,
                    crate::event::MouseButton::Button2 => egui::PointerButton::Secondary,
                    crate::event::MouseButton::Button3 => egui::PointerButton::Middle,
                    _ => return,
                };

                if let Some(pos) = self.cursor_pos() {
                    // Convert physical pixels to logical coordinates
                    let pos = egui::Pos2::new(
                        (pos.0 as f32) / scale_factor,
                        (pos.1 as f32) / scale_factor,
                    );
                    let pressed = action == Action::Press;

                    self.egui_context
                        .raw_input
                        .events
                        .push(egui::Event::PointerButton {
                            pos,
                            button,
                            pressed,
                            modifiers: self.get_egui_modifiers(),
                        });
                }
            }
            WindowEvent::Scroll(_x, y, _) => {
                self.egui_context
                    .raw_input
                    .events
                    .push(egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Line,
                        delta: egui::Vec2::new(0.0, y as f32),
                        modifiers: self.get_egui_modifiers(),
                    });
            }
            WindowEvent::Char(ch) => {
                if !ch.is_control() {
                    self.egui_context
                        .raw_input
                        .events
                        .push(egui::Event::Text(ch.to_string()));
                }
            }
            WindowEvent::Key(key, action, _modifiers) => {
                if let Some(egui_key) = self.translate_key_to_egui(key) {
                    self.egui_context.raw_input.events.push(egui::Event::Key {
                        key: egui_key,
                        physical_key: None,
                        pressed: action == Action::Press,
                        repeat: false,
                        modifiers: self.get_egui_modifiers(),
                    });
                }
            }
            _ => {}
        }
    }

    #[cfg(feature = "egui")]
    fn get_egui_modifiers(&self) -> egui::Modifiers {
        egui::Modifiers {
            alt: self.get_key(Key::LAlt) == Action::Press
                || self.get_key(Key::RAlt) == Action::Press,
            ctrl: self.get_key(Key::LControl) == Action::Press
                || self.get_key(Key::RControl) == Action::Press,
            shift: self.get_key(Key::LShift) == Action::Press
                || self.get_key(Key::RShift) == Action::Press,
            mac_cmd: false,
            command: self.get_key(Key::LControl) == Action::Press
                || self.get_key(Key::RControl) == Action::Press,
        }
    }

    #[cfg(feature = "egui")]
    fn translate_key_to_egui(&self, key: Key) -> Option<egui::Key> {
        Some(match key {
            Key::A => egui::Key::A,
            Key::B => egui::Key::B,
            Key::C => egui::Key::C,
            Key::D => egui::Key::D,
            Key::E => egui::Key::E,
            Key::F => egui::Key::F,
            Key::G => egui::Key::G,
            Key::H => egui::Key::H,
            Key::I => egui::Key::I,
            Key::J => egui::Key::J,
            Key::K => egui::Key::K,
            Key::L => egui::Key::L,
            Key::M => egui::Key::M,
            Key::N => egui::Key::N,
            Key::O => egui::Key::O,
            Key::P => egui::Key::P,
            Key::Q => egui::Key::Q,
            Key::R => egui::Key::R,
            Key::S => egui::Key::S,
            Key::T => egui::Key::T,
            Key::U => egui::Key::U,
            Key::V => egui::Key::V,
            Key::W => egui::Key::W,
            Key::X => egui::Key::X,
            Key::Y => egui::Key::Y,
            Key::Z => egui::Key::Z,
            Key::Escape => egui::Key::Escape,
            Key::Tab => egui::Key::Tab,
            Key::Back => egui::Key::Backspace,
            Key::Return => egui::Key::Enter,
            Key::Space => egui::Key::Space,
            Key::Insert => egui::Key::Insert,
            Key::Delete => egui::Key::Delete,
            Key::Home => egui::Key::Home,
            Key::End => egui::Key::End,
            Key::PageUp => egui::Key::PageUp,
            Key::PageDown => egui::Key::PageDown,
            Key::Left => egui::Key::ArrowLeft,
            Key::Up => egui::Key::ArrowUp,
            Key::Right => egui::Key::ArrowRight,
            Key::Down => egui::Key::ArrowDown,
            _ => return None,
        })
    }

    /// Runs the egui UI code. Call this once per frame after render().await returns true.
    ///
    /// # Example
    /// ```no_run
    /// # use kiss3d::window::Window;
    /// # #[kiss3d::main]
    /// # async fn main() {
    /// let mut window = Window::new("egui example");
    /// while window.render().await {
    ///     window.draw_ui(|ctx| {
    ///         egui::Window::new("My Window").show(ctx, |ui| {
    ///             ui.label("Hello, egui!");
    ///         });
    ///     });
    /// }
    /// # }
    /// ```
    #[cfg(feature = "egui")]
    pub fn draw_ui<F>(&mut self, ui_fn: F)
    where
        F: FnOnce(&egui::Context),
    {
        // Get time for animations - use egui context's own start time
        #[cfg(not(target_arch = "wasm32"))]
        let time = Some(self.egui_context.start_time.elapsed().as_secs_f64());
        #[cfg(target_arch = "wasm32")]
        let time = {
            // On WASM, use instant which is already configured
            use instant::Instant;
            static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
            let start = START.get_or_init(Instant::now);
            Some(start.elapsed().as_secs_f64())
        };

        let scale_factor = self.canvas.scale_factor() as f32;

        // Set pixels_per_point on the context to match our DPI scale
        self.egui_context
            .renderer
            .context()
            .set_pixels_per_point(scale_factor);

        // Build raw input with accumulated events
        let mut raw_input = std::mem::take(&mut self.egui_context.raw_input);
        raw_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(
                self.width() as f32 / scale_factor,
                self.height() as f32 / scale_factor,
            ),
        ));
        raw_input.time = time;
        raw_input.predicted_dt = 1.0 / 60.0;

        self.egui_context.renderer.begin_frame(raw_input);
        ui_fn(self.egui_context.renderer.context());
        self.egui_context.renderer.end_frame();

        // Reset raw_input for next frame (but keep it properly initialized)
        self.egui_context.raw_input = RawInput::default();
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
            #[cfg(feature = "egui")]
            egui_context: EguiContext::new(),
            post_process_render_target: FramebufferManager::new_render_target(
                width as usize,
                height as usize,
                true,
            ),
            framebuffer_manager: FramebufferManager::new(),
            #[cfg(not(target_arch = "wasm32"))]
            curr_time: std::time::Instant::now(),
            planar_camera: Rc::new(RefCell::new(PlanarFixedView::new())),
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
        let size = width * height * 3;

        if out.len() < size {
            let diff = size - out.len();
            out.extend(std::iter::repeat_n(0, diff));
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
        let img_opt = ImageBuffer::from_vec(width, height, buf);
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

    /// Gets the state of a mouse button.
    pub fn get_mouse_button(&self, button: MouseButton) -> Action {
        self.canvas.get_mouse_button(button)
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

        // Feed events to egui and check if it wants to capture input
        #[cfg(feature = "egui")]
        {
            self.feed_egui_event(event);

            if event.is_keyboard_event() && self.is_egui_capturing_keyboard() {
                return;
            }

            if event.is_mouse_event() && self.is_egui_capturing_mouse() {
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

    /// Renders the scene using the default camera.
    ///
    /// Returns `false` if the window should be closed.
    // TODO: would be good to have a 2D version of this that renders with a 2D side-scroll camera.
    pub async fn render(&mut self) -> bool {
        self.render_with(None, None, None, None).await
    }

    /// Render using a specific post processing effect.
    ///
    /// Returns `false` if the window should be closed.
    pub async fn render_with_effect(&mut self, effect: &mut dyn PostProcessingEffect) -> bool {
        self.render_with(None, None, Some(effect), None).await
    }

    /// Render using a specific camera.
    ///
    /// Returns `false` if the window should be closed.
    pub async fn render_with_camera(&mut self, camera: &mut dyn Camera) -> bool {
        self.render_with(Some(camera), None, None, None).await
    }

    /// Render using a specific 2D and 3D camera.
    ///
    /// Returns `false` if the window should be closed.
    pub async fn render_with_cameras(
        &mut self,
        camera: &mut dyn Camera,
        planar_camera: &mut dyn PlanarCamera,
    ) -> bool {
        self.render_with(Some(camera), Some(planar_camera), None, None)
            .await
    }

    /// Render using a specific camera and post processing effect.
    ///
    /// Returns `false` if the window should be closed.
    pub async fn render_with_camera_and_effect(
        &mut self,
        camera: &mut dyn Camera,
        effect: &mut dyn PostProcessingEffect,
    ) -> bool {
        self.render_with(Some(camera), None, Some(effect), None)
            .await
    }

    /// Render using a specific 2D and 3D camera and post processing effect.
    ///
    /// Returns `false` if the window should be closed.
    pub async fn render_with_cameras_and_effect(
        &mut self,
        camera: &mut dyn Camera,
        planar_camera: &mut dyn PlanarCamera,
        effect: &mut dyn PostProcessingEffect,
    ) -> bool {
        self.render_with(Some(camera), Some(planar_camera), Some(effect), None)
            .await
    }

    /// Draws the scene with the given cameras, post-processing effect, and custom renderer.
    ///
    /// Returns `false` if the window should be closed.
    pub async fn render_with(
        &mut self,
        camera: Option<&mut dyn Camera>,
        planar_camera: Option<&mut dyn PlanarCamera>,
        post_processing: Option<&mut dyn PostProcessingEffect>,
        renderer: Option<&mut dyn Renderer>,
    ) -> bool {
        // FIXME: for backward-compatibility, we don't accept any custom renderer here.
        self.do_render_with(camera, planar_camera, renderer, post_processing)
            .await
    }

    async fn do_render_with(
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
        let self_cam = self.camera.clone(); // FIXME: this is ugly.

        match (camera, planar_camera) {
            (Some(cam), Some(cam2)) => {
                self.render_single_frame(cam, cam2, renderer, post_processing)
                    .await
            }
            (None, Some(cam2)) => {
                self.render_single_frame(
                    &mut *self_cam.borrow_mut(),
                    cam2,
                    renderer,
                    post_processing,
                )
                .await
            }
            (Some(cam), None) => {
                self.render_single_frame(
                    cam,
                    &mut *self_cam2.borrow_mut(),
                    renderer,
                    post_processing,
                )
                .await
            }
            (None, None) => {
                self.render_single_frame(
                    &mut *self_cam.borrow_mut(),
                    &mut *self_cam2.borrow_mut(),
                    renderer,
                    post_processing,
                )
                .await
            }
        }
    }

    async fn render_single_frame(
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
        #[cfg(feature = "egui")]
        self.egui_context
            .renderer
            .render(w as f32, h as f32, self.canvas.scale_factor() as f32);

        // We are done: swap buffers
        self.canvas.swap_buffers();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use web_sys::wasm_bindgen::closure::Closure;

            if let Some(window) = web_sys::window() {
                let (s, r) = oneshot::channel();

                let closure = Closure::once(move || s.send(()).unwrap());

                window
                    .request_animation_frame(closure.as_ref().unchecked_ref())
                    .unwrap();

                r.await.unwrap();
            }
        }

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
