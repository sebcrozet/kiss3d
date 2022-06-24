use crate::camera::Camera;
use crate::event::{Action, Key, Modifiers, MouseButton, WindowEvent};
use crate::resource::ShaderUniform;
use crate::window::Canvas;
use na::{self, Isometry3, Matrix4, Perspective3, Point3, Unit, UnitQuaternion, Vector2, Vector3};
use std::f32;

/// Arc-ball camera mode.
///
/// An arc-ball camera is a camera rotating around a fixed point (the focus point) and always
/// looking at it. The following inputs are handled:
///
/// * Left button press + drag - rotates the camera around the focus point
/// * Right button press + drag - translates the focus point on the plane orthogonal to the view
/// direction
/// * Scroll in/out - zoom in/out
/// * Enter key - set the focus point to the origin
#[derive(Clone, Debug)]
pub struct ArcBall {
    /// The focus point.
    at: Point3<f32>,
    /// Yaw of the camera (rotation along the y axis).
    yaw: f32,
    /// Pitch of the camera (rotation along the x axis).
    pitch: f32,
    /// Distance from the camera to the `at` focus point.
    dist: f32,
    /// Minimum distance from the camera to the `at` focus point.
    min_dist: f32,
    /// Maximum distance from the camera to the `at` focus point.
    max_dist: f32,

    /// Increment of the yaw per unit mouse movement. The default value is 0.005.
    yaw_step: f32,
    /// Increment of the pitch per unit mouse movement. The default value is 0.005.
    pitch_step: f32,
    /// Minimum pitch of the camera.
    min_pitch: f32,
    /// Maximum pitch of the camera.
    max_pitch: f32,
    /// Distance change factor per unit scrolling. The default value is 1.01.
    dist_step: f32,
    rotate_button: Option<MouseButton>,
    rotate_modifiers: Option<Modifiers>,
    drag_button: Option<MouseButton>,
    drag_modifiers: Option<Modifiers>,
    reset_key: Option<Key>,

    projection: Perspective3<f32>,
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
    proj_view: Matrix4<f32>,
    inverse_proj_view: Matrix4<f32>,
    last_cursor_pos: Vector2<f32>,
    last_framebuffer_size: Vector2<f32>,
    coord_system: CoordSystemRh,
}

impl ArcBall {
    /// Create a new arc-ball camera.
    pub fn new(eye: Point3<f32>, at: Point3<f32>) -> ArcBall {
        ArcBall::new_with_frustrum(f32::consts::PI / 4.0, 0.001, 1024.0, eye, at)
    }

    /// Creates a new arc ball camera with default sensitivity values.
    pub fn new_with_frustrum(
        fov: f32,
        znear: f32,
        zfar: f32,
        eye: Point3<f32>,
        at: Point3<f32>,
    ) -> ArcBall {
        let mut res = ArcBall {
            at: Point3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            dist: 0.0,
            min_dist: 0.00001,
            max_dist: 1.0e4,
            yaw_step: 0.005,
            pitch_step: 0.005,
            min_pitch: 0.01,
            max_pitch: std::f32::consts::PI - 0.01,
            dist_step: 1.01,
            rotate_button: Some(MouseButton::Button1),
            rotate_modifiers: None,
            drag_button: Some(MouseButton::Button2),
            drag_modifiers: None,
            reset_key: Some(Key::Return),
            projection: Perspective3::new(800.0 / 600.0, fov, znear, zfar),
            view: na::zero(),
            proj: na::zero(),
            proj_view: na::zero(),
            inverse_proj_view: na::zero(),
            last_framebuffer_size: Vector2::new(800.0, 600.0),
            last_cursor_pos: na::zero(),
            coord_system: CoordSystemRh::from_up_axis(Vector3::y_axis()),
        };

        res.look_at(eye, at);

        res
    }

    /// The point the arc-ball is looking at.
    pub fn at(&self) -> Point3<f32> {
        self.at
    }

    /// Get a mutable reference to the point the camera is looking at.
    pub fn set_at(&mut self, at: Point3<f32>) {
        self.at = at;
        self.update_projviews();
    }

    /// The arc-ball camera `yaw`.
    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    /// Sets the camera `yaw`. Change this to modify the rotation along the `up` axis.
    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;

        self.update_restrictions();
        self.update_projviews();
    }

    /// The arc-ball camera `pitch`.
    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    /// Sets the camera `pitch`.
    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch;

        self.update_restrictions();
        self.update_projviews();
    }

    /// The minimum pitch of the camera.
    pub fn min_pitch(&self) -> f32 {
        self.min_pitch
    }

    /// Set the minimum pitch of the camera.
    pub fn set_min_pitch(&mut self, min_pitch: f32) {
        self.min_pitch = min_pitch;
    }

    /// The maximum pitch of the camera.
    pub fn max_pitch(&self) -> f32 {
        self.max_pitch
    }

    /// Set the maximum pitch of the camera.
    pub fn set_max_pitch(&mut self, max_pitch: f32) {
        self.max_pitch = max_pitch;
    }

    /// The distance from the camera position to its view point.
    pub fn dist(&self) -> f32 {
        self.dist
    }

    /// Move the camera such that it is at a given distance from the view point.
    pub fn set_dist(&mut self, dist: f32) {
        self.dist = dist;

        self.update_restrictions();
        self.update_projviews();
    }

    /// The minimum distance from the camera position to its view point.
    pub fn min_dist(&self) -> f32 {
        self.min_dist
    }

    /// Set the minimum distance from the camera position to its view point.
    pub fn set_min_dist(&mut self, min_dist: f32) {
        self.min_dist = min_dist;
    }

    /// The maximum distance from the camera position to its view point.
    pub fn max_dist(&self) -> f32 {
        self.max_dist
    }

    /// Set the maximum distance from the camera position to its view point.
    pub fn set_max_dist(&mut self, max_dist: f32) {
        self.max_dist = max_dist;
    }

    /// Set the distance change factor for a unit scroll (default at 1.01).
    pub fn set_dist_step(&mut self, dist_step: f32) {
        self.dist_step = dist_step;
    }

    /// Move and orient the camera such that it looks at a specific point.
    pub fn look_at(&mut self, eye: Point3<f32>, at: Point3<f32>) {
        let dist = (eye - at).norm();

        let view_eye = self.coord_system.rotation_to_y_up * eye;
        let view_at = self.coord_system.rotation_to_y_up * at;
        let pitch = ((view_eye.y - view_at.y) / dist).acos();
        let yaw = (view_eye.z - view_at.z).atan2(view_eye.x - view_at.x);

        self.at = at;
        self.dist = dist;
        self.yaw = yaw;
        self.pitch = pitch;

        self.update_restrictions();
        self.update_projviews();
    }

    /// Transformation applied by the camera without perspective.
    fn update_restrictions(&mut self) {
        if self.dist < self.min_dist {
            self.dist = self.min_dist
        }

        if self.dist > self.max_dist {
            self.dist = self.max_dist
        }

        if self.pitch <= self.min_pitch {
            self.pitch = self.min_pitch
        }

        if self.pitch > self.max_pitch {
            self.pitch = self.max_pitch
        }
    }

    /// The button used to rotate the ArcBall camera.
    pub fn rotate_button(&self) -> Option<MouseButton> {
        self.rotate_button
    }

    /// Set the button used to rotate the ArcBall camera.
    /// Use None to disable rotation.
    pub fn rebind_rotate_button(&mut self, new_button: Option<MouseButton>) {
        self.rotate_button = new_button;
    }

    /// Modifiers that must be pressed for the camera rotation to occur.
    pub fn rotate_modifiers(&self) -> Option<Modifiers> {
        self.rotate_modifiers
    }

    /// Sets the modifiers that must be pressed for the camera rotation to occur.
    ///
    /// If this is set to `None`, then pressing any modifier will not prevent rotation from occurring.
    /// If this is different from `None` then rotation will occur only if the exact specified set of modifiers is pressed.
    /// In particular, if this is set to `Some(Modifiers::empty())` then, rotation will occur only of no modifier is pressed.
    pub fn set_rotate_modifiers(&mut self, modifiers: Option<Modifiers>) {
        self.rotate_modifiers = modifiers
    }

    /// Modifiers that must be pressed for the camera drag to occur.
    pub fn drag_modifiers(&self) -> Option<Modifiers> {
        self.drag_modifiers
    }

    /// Sets the modifiers that must be pressed for the camera drag to occur.
    ///
    /// If this is set to `None`, then pressing any modifier will not prevent dragging from occurring.
    /// If this is different from `None` then drag will occur only if the exact specified set of modifiers is pressed.
    /// In particular, if this is set to `Some(Modifiers::empty())` then, drag will occur only of no modifier is pressed.
    pub fn set_drag_modifiers(&mut self, modifiers: Option<Modifiers>) {
        self.drag_modifiers = modifiers
    }

    /// The button used to drag the ArcBall camera.
    pub fn drag_button(&self) -> Option<MouseButton> {
        self.drag_button
    }

    /// Set the button used to drag the ArcBall camera.
    /// Use None to disable dragging.
    pub fn rebind_drag_button(&mut self, new_button: Option<MouseButton>) {
        self.drag_button = new_button;
    }

    /// The key used to reset the ArcBall camera.
    pub fn reset_key(&self) -> Option<Key> {
        self.reset_key
    }

    /// Set the key used to reset the ArcBall camera.
    /// Use None to disable reset.
    pub fn rebind_reset_key(&mut self, new_key: Option<Key>) {
        self.reset_key = new_key;
    }

    fn handle_left_button_displacement(&mut self, dpos: &Vector2<f32>) {
        self.yaw += dpos.x * self.yaw_step;
        self.pitch -= dpos.y * self.pitch_step;

        self.update_restrictions();
        self.update_projviews();
    }

    /// Performs a translation of the camera eye and focus.
    /// The delta coordinates are expected to be normalized to the [-1, 1] range.
    fn handle_right_button_displacement(&mut self, dpos_norm: &Vector2<f32>) {
        let eye = self.eye();
        let dir = (self.at - eye).normalize();
        let tangent = self.coord_system.up_axis.cross(&dir).normalize();
        let bitangent = dir.cross(&tangent);
        self.at =
            self.at + tangent * (dpos_norm.x * self.dist) + bitangent * (dpos_norm.y * self.dist);
        self.update_projviews();
    }

    fn handle_scroll(&mut self, off: f32) {
        // To "focus" the zoom towards the point under the cursor, first we
        // translate the camera to bring that point in the center of the view
        // and then undo the translation.
        let mut dpos = Vector2::new(
            0.5 - self.last_cursor_pos.x / self.last_framebuffer_size.x,
            0.5 - self.last_cursor_pos.y / self.last_framebuffer_size.y,
        );
        self.handle_right_button_displacement(&dpos);

        self.dist *= self.dist_step.powf(off);
        self.update_restrictions();
        self.update_projviews();

        dpos = -dpos;
        self.handle_right_button_displacement(&dpos);
    }

    fn update_projviews(&mut self) {
        self.proj = *self.projection.as_matrix();
        self.view = self.view_transform().to_homogeneous();
        self.proj_view = self.proj * self.view;
        self.inverse_proj_view = self.proj_view.try_inverse().unwrap();
    }

    /// Sets the up vector of this camera. Prefer using [`set_up_axis_dir`](#method.set_up_axis_dir)
    /// if your up vector is already normalized.
    #[inline]
    pub fn set_up_axis(&mut self, up_axis: Vector3<f32>) {
        self.set_up_axis_dir(Unit::new_normalize(up_axis));
    }

    /// Sets the up-axis direction of this camera.
    #[inline]
    pub fn set_up_axis_dir(&mut self, up_axis: Unit<Vector3<f32>>) {
        if self.coord_system.up_axis != up_axis {
            let new_coord_system = CoordSystemRh::from_up_axis(up_axis);
            // Since setting the up axis changes the meaning of pitch and yaw
            // angles, we need to recalculate them in order to preserve the eye
            // position.
            let old_eye = self.eye();
            self.coord_system = new_coord_system;
            self.look_at(old_eye, self.at);
        }
    }
}

impl Camera for ArcBall {
    fn clip_planes(&self) -> (f32, f32) {
        (self.projection.znear(), self.projection.zfar())
    }

    fn view_transform(&self) -> Isometry3<f32> {
        Isometry3::look_at_rh(&self.eye(), &self.at, &self.coord_system.up_axis)
    }

    fn eye(&self) -> Point3<f32> {
        let view_at = self.coord_system.rotation_to_y_up * self.at;
        let px = view_at.x + self.dist * self.yaw.cos() * self.pitch.sin();
        let py = view_at.y + self.dist * self.pitch.cos();
        let pz = view_at.z + self.dist * self.yaw.sin() * self.pitch.sin();
        self.coord_system.rotation_to_y_up.inverse() * Point3::new(px, py, pz)
    }

    fn handle_event(&mut self, canvas: &Canvas, event: &WindowEvent) {
        match *event {
            WindowEvent::CursorPos(x, y, modifiers) => {
                let curr_pos = Vector2::new(x as f32, y as f32);

                if let Some(rotate_button) = self.rotate_button {
                    if canvas.get_mouse_button(rotate_button) == Action::Press
                        && self
                            .rotate_modifiers
                            .map(|m| m == modifiers)
                            .unwrap_or(true)
                    {
                        let dpos = curr_pos - self.last_cursor_pos;
                        self.handle_left_button_displacement(&dpos)
                    }
                }

                if let Some(drag_button) = self.drag_button {
                    if canvas.get_mouse_button(drag_button) == Action::Press
                        && self.drag_modifiers.map(|m| m == modifiers).unwrap_or(true)
                    {
                        let dpos = curr_pos - self.last_cursor_pos;
                        let dpos_norm = dpos.component_div(&self.last_framebuffer_size);
                        self.handle_right_button_displacement(&dpos_norm)
                    }
                }

                self.last_cursor_pos = curr_pos;
            }
            WindowEvent::Key(key, Action::Press, _) if Some(key) == self.reset_key => {
                self.at = Point3::origin();
                self.update_projviews();
            }
            WindowEvent::Scroll(_, off, _) => self.handle_scroll(off as f32),
            WindowEvent::FramebufferSize(w, h) => {
                self.last_framebuffer_size = Vector2::new(w as f32, h as f32);
                self.projection.set_aspect(w as f32 / h as f32);
                self.update_projviews();
            }
            _ => {}
        }
    }

    #[inline]
    fn upload(
        &self,
        _: usize,
        proj: &mut ShaderUniform<Matrix4<f32>>,
        view: &mut ShaderUniform<Matrix4<f32>>,
    ) {
        proj.upload(&self.proj);
        view.upload(&self.view);
    }

    fn transformation(&self) -> Matrix4<f32> {
        self.proj_view
    }

    fn inverse_transformation(&self) -> Matrix4<f32> {
        self.inverse_proj_view
    }

    fn update(&mut self, _: &Canvas) {}
}

#[derive(Clone, Copy, Debug)]
struct CoordSystemRh {
    up_axis: Unit<Vector3<f32>>,
    rotation_to_y_up: UnitQuaternion<f32>,
}

impl CoordSystemRh {
    #[inline]
    fn from_up_axis(up_axis: Unit<Vector3<f32>>) -> Self {
        let rotation_to_y_up = UnitQuaternion::rotation_between_axis(&up_axis, &Vector3::y_axis())
            .unwrap_or_else(|| {
                UnitQuaternion::from_axis_angle(&Vector3::x_axis(), std::f32::consts::PI)
            });
        Self {
            up_axis,
            rotation_to_y_up,
        }
    }
}
