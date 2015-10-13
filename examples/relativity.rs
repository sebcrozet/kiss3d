extern crate num;
extern crate gl;
extern crate glfw;
extern crate kiss3d;
extern crate nalgebra as na;

use std::f32;
use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, RwLock};
use num::Float;
use gl::types::{GLint, GLfloat};
use glfw::{Key, Action, WindowEvent};
use na::{Pnt2, Pnt3, Vec3, Mat3, Mat4, Rot3, Iso3, Rotation, Translation, Norm};
use kiss3d::window::Window;
use kiss3d::text::Font;
use kiss3d::scene::ObjectData;
use kiss3d::camera::{Camera, FirstPerson};
use kiss3d::light::Light;
use kiss3d::resource::{Shader, ShaderAttribute, ShaderUniform, Material, Mesh};

fn main() {
    let mut window = Window::new("Kiss3d: relativity");

    let eye      = Pnt3::new(0.0f32, -399.0, 400.0);
    let at       = Pnt3::new(0.0f32, -399.0, 0.0);
    let fov      = f32::consts::PI / 4.0;
    let font     = Font::new(&Path::new("media/font/Inconsolata.otf"), 60);
    let context  = Arc::new(RwLock::new(Context::new(1000.0, na::zero(), eye)));
    let material = Rc::new(RefCell::new(Box::new(RelativisticMaterial::new(context.clone())) as Box<Material + 'static>));
    let mut observer = InertialCamera::new(fov, 0.1, 100000.0, eye, at);

    window.set_framerate_limit(Some(60));

    let mut c = window.add_quad(800.0, 800.0, 40, 40);
    c.set_material(material.clone());
    c.set_texture_from_file(&Path::new("media/kitten.png"), "kitten");

    let mut c = window.add_quad(800.0, 800.0, 40, 40);
    c.append_rotation(&(Vec3::x() * f32::consts::PI / 2.0));
    c.append_translation(&(Vec3::new(0.0, -400.0, 400.0)));
    c.set_material(material.clone());
    c.set_texture_with_name("kitten");

    let mut c = window.add_quad(800.0, 800.0, 40, 40);
    c.append_rotation(&(Vec3::y() * f32::consts::PI / 2.0));
    c.append_translation(&(Vec3::new(400.0, 0.0, 400.0)));
    c.set_material(material.clone());
    c.set_texture_with_name("kitten");

    let mut c = window.add_quad(800.0, 800.0, 40, 40);
    c.append_rotation(&(Vec3::y() * f32::consts::PI / 2.0));
    c.append_translation(&(Vec3::new(-400.0, 0.0, 400.0)));
    c.set_material(material.clone());
    c.set_texture_with_name("kitten");

    window.set_light(Light::StickToCamera);

    /*
     * Render
     */
    while window.render_with_camera(&mut observer) {
        let mut c = context.write().unwrap();

        for event in window.events().iter() {
            match event.value {
                WindowEvent::Key(code, _, Action::Release, _) => {
                    if code == Key::Num1 {
                        c.speed_of_light = c.speed_of_light + 100.0;
                    }
                    else if code == Key::Num2 {
                        c.speed_of_light = (c.speed_of_light - 100.0).max(0.1);
                    }
                },
                _ => { }
            }
        }

        let obs_vel = observer.velocity;
        let sop = na::norm(&obs_vel);

        window.draw_text(
            &format!("Speed of light: {}\nSpeed of player: {}", c.speed_of_light, sop)[..],
            &na::orig(),
            &font,
            &Pnt3::new(1.0, 1.0, 1.0));

        observer.max_vel  = c.speed_of_light * 0.85;
        c.speed_of_player = obs_vel;
        c.position        = observer.eye();
    }
}

struct InertialCamera {
    cam:          FirstPerson,
    acceleration: f32,
    deceleration: f32,
    max_vel:      f32,
    velocity:     Vec3<f32>
}

impl InertialCamera {
    fn new(fov: f32, znear: f32, zfar: f32, eye: Pnt3<f32>, at: Pnt3<f32>) -> InertialCamera {
        let mut fp = FirstPerson::new_with_frustrum(fov, znear, zfar, eye, at);

        fp.set_move_step(0.0);

        InertialCamera {
            cam:          fp,
            acceleration: 200.0f32,
            deceleration: 0.95f32,
            max_vel:      1.0,
            velocity:     na::zero()
        }
    }
}

impl Camera for InertialCamera {
    fn clip_planes(&self) -> (f32, f32) {
        self.cam.clip_planes()
    }

    fn view_transform(&self) -> Iso3<f32> {
        self.cam.view_transform()
    }

    fn handle_event(&mut self, window: &glfw::Window, event: &glfw::WindowEvent) {
        self.cam.handle_event(window, event)
    }

    fn eye(&self) -> Pnt3<f32> {
        self.cam.eye()
    }

    fn transformation(&self) -> Mat4<f32> {
        self.cam.transformation()
    }

    fn inv_transformation(&self) -> Mat4<f32> {
        self.cam.inv_transformation()
    }

    fn update(&mut self, window: &glfw::Window) {
        let up    = window.get_key(Key::Up)    == Action::Press;
        let down  = window.get_key(Key::Down)  == Action::Press;
        let right = window.get_key(Key::Right) == Action::Press;
        let left  = window.get_key(Key::Left)  == Action::Press;

        let dir = self.cam.move_dir(up, down, right, left);

        if !na::is_zero(&dir) {
            self.velocity = self.velocity + dir * self.acceleration * 0.016f32;
        }
        else {
            self.velocity = self.velocity * self.deceleration;
        }

        let speed = self.velocity.normalize_mut().min(self.max_vel);

        if speed != 0.0 {
            self.velocity.y = 0.0;
            self.velocity.normalize_mut();
            self.velocity = self.velocity * speed;
        }
        else {
            self.velocity = na::zero();
        }

        self.cam.append_translation(&(self.velocity * 0.016f32));
    }
}

struct Context {
    speed_of_light:  f32,
    speed_of_player: Vec3<f32>,
    position:        Pnt3<f32>
}

impl Context {
    fn new(speed_of_light: f32, speed_of_player: Vec3<f32>, position: Pnt3<f32>) -> Context {
        Context {
            speed_of_light:  speed_of_light,
            speed_of_player: speed_of_player,
            position:        position
        }
    }
}

/// The default material used to draw objects.
struct RelativisticMaterial {
    context:         Arc<RwLock<Context>>,
    shader:          Shader,
    pos:             ShaderAttribute<Pnt3<f32>>,
    normal:          ShaderAttribute<Vec3<f32>>,
    tex_coord:       ShaderAttribute<Pnt2<f32>>,
    light:           ShaderUniform<Pnt3<f32>>,
    color:           ShaderUniform<Pnt3<f32>>,
    transform:       ShaderUniform<Mat4<f32>>,
    scale:           ShaderUniform<Mat3<f32>>,
    ntransform:      ShaderUniform<Mat3<f32>>,
    view:            ShaderUniform<Mat4<f32>>,
    light_vel:       ShaderUniform<GLfloat>,
    rel_vel:         ShaderUniform<Vec3<f32>>,
    rot:             ShaderUniform<Rot3<f32>>,
    player_position: ShaderUniform<Pnt3<f32>>
}

impl RelativisticMaterial {
    /// Creates a new `RelativisticMaterial`.
    fn new(context: Arc<RwLock<Context>>) -> RelativisticMaterial {
        // load the shader
        let mut shader = Shader::new_from_str(RELATIVISTIC_VERTEX_SRC, RELATIVISTIC_FRAGMENT_SRC);

        shader.use_program();

        // get the variables locations
        RelativisticMaterial {
            context:         context,
            pos:             shader.get_attrib("position").unwrap(),
            normal:          shader.get_attrib("normal").unwrap(),
            tex_coord:       shader.get_attrib("tex_coord_v").unwrap(),
            light:           shader.get_uniform("light_position").unwrap(),
            player_position: shader.get_uniform("player_position").unwrap(),
            light_vel:       shader.get_uniform("light_vel").unwrap(),
            rel_vel:         shader.get_uniform("rel_vel").unwrap(),
            rot:             shader.get_uniform("rot").unwrap(),
            color:           shader.get_uniform("color").unwrap(),
            transform:       shader.get_uniform("transform").unwrap(),
            scale:           shader.get_uniform("scale").unwrap(),
            ntransform:      shader.get_uniform("ntransform").unwrap(),
            view:            shader.get_uniform("view").unwrap(),
            shader:          shader
        }
    }

    fn activate(&mut self) {
        self.shader.use_program();
        self.pos.enable();
        self.normal.enable();
        self.tex_coord.enable();
    }

    fn deactivate(&mut self) {
        self.pos.disable();
        self.normal.disable();
        self.tex_coord.disable();
    }
}

impl Material for RelativisticMaterial {
    fn render(&mut self,
              pass:      usize,
              transform: &Iso3<f32>, 
              scale:     &Vec3<f32>,
              camera:    &mut Camera,
              light:     &Light,
              data:      &ObjectData,
              mesh:      &mut Mesh) {
        self.activate();

        /*
         *
         * Setup camera and light.
         *
         */
        camera.upload(pass, &mut self.view);

        let pos = match *light {
            Light::Absolute(ref p) => p.clone(),
            Light::StickToCamera   => camera.eye()
        };

        self.light.upload(&pos);

        let ctxt = self.context.clone();

        {
            let c = ctxt.read().unwrap();
            // XXX: this relative velocity est very wrong!
            self.rel_vel.upload(&c.speed_of_player);
            self.light_vel.upload(&c.speed_of_light);
            self.player_position.upload(&c.position);

            let mut rot = na::one::<Rot3<f32>>();

            if na::sqnorm(&c.speed_of_player) != 0.0 {
                rot = Rot3::look_at_z(&(-c.speed_of_player), &Vec3::y());
            }

            self.rot.upload(&rot);
        }

        /*
         *
         * Setup object-related stuffs.
         *
         */
        let formated_transform:  Mat4<f32> = na::to_homogeneous(transform);
        let formated_ntransform: Mat3<f32> = *transform.rotation.submat();
        let formated_scale:      Mat3<f32> = Mat3::new(scale.x, 0.0, 0.0, 0.0, scale.y, 0.0, 0.0, 0.0, scale.z);
        // XXX: there should be a `na::diagonal(scale)` function on nalgebra.

        self.transform.upload(&formated_transform);
        self.ntransform.upload(&formated_ntransform);
        self.scale.upload(&formated_scale);
        self.color.upload(data.color());

        mesh.bind(&mut self.pos, &mut self.normal, &mut self.tex_coord);

        unsafe {
            if data.backface_culling_enabled() {
                gl::Enable(gl::CULL_FACE);
            }
            else {
                gl::Disable(gl::CULL_FACE);
            }

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, data.texture().id());

            gl::DrawElements(gl::TRIANGLES,
                             mesh.num_pts() as GLint,
                             gl::UNSIGNED_INT,
                             ptr::null());
        }

        mesh.unbind();
        self.deactivate();
    }
}

static RELATIVISTIC_VERTEX_SRC:   &'static str =
   "#version 120
    attribute vec3 position;
    attribute vec3 normal;
    attribute vec3 color;
    attribute vec2 tex_coord_v;
    varying vec3   ws_normal;
    varying vec3   ws_position;
    varying vec2   tex_coord;
    uniform mat4   view;
    uniform mat4   transform;
    uniform mat3   scale;
    uniform mat3   ntransform;
    uniform float  light_vel;
    uniform vec3   rel_vel;
    uniform mat3   rot;
    uniform vec3   player_position;
    void main() {
        // mat4 scale4 = mat4(scale);
        // vec4 pos4   = transform * scale4 * vec4(position, 1.0);
        // tex_coord   = tex_coord_v;
        // ws_position = pos4.xyz;
        // pos4.z      /= (1.0 - sqrt(dot(rel_vel, rel_vel)));
        // gl_Position = view * pos4;
        // ws_normal   = normalize(ntransform * scale * normal);


        mat4 scale4  = mat4(scale);

        vec4 pos4    = transform * scale4 * vec4(position, 1.0);

        ws_position  = pos4.xyz - player_position;
        ws_position  = rot * ws_position;


        vec3 rot_vel = rot * rel_vel;

        vec3 norm_ws_position = normalize(ws_position);

        float dt;
        
        dt = sqrt(dot(ws_position, ws_position)) / light_vel;

        ws_position.z += rot_vel.z * dt;

        ws_position.z /= sqrt(1.0 - dot(rel_vel, rel_vel) / (light_vel * light_vel));


        ws_position   =  ws_position * rot;

        gl_Position = view * vec4(player_position + ws_position, 1.0);

        ws_normal   = normalize(ntransform * scale * normal);
        tex_coord   = tex_coord_v;
    }";

static RELATIVISTIC_FRAGMENT_SRC: &'static str =
   "#version 120
    uniform vec3      color;
    uniform vec3      light_position;
    uniform vec3      rel_vel;
    uniform sampler2D tex;
    varying vec2      tex_coord;
    varying vec3      ws_normal;
    varying vec3      ws_position;
    uniform float     light_vel;
    uniform vec3      player_position;




// According to the CIE RGB (http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html)
    vec3 rgb2xyz(vec3 c) {
        vec3 res;
        res.x = 0.4887180 * c.x + 0.3106803 * c.y + 0.2006017 * c.z;
        res.y = 0.1762044 * c.x + 0.8129847 * c.y + 0.0108109 * c.z;
        res.z = 0.0102048 * c.y + 0.9897952 * c.z;
        return res;
    }

    vec3 xyz2rgb(vec3 c) {
        vec3 res;
        res.x = 2.3706743 * c.x - 0.9000405 * c.y - 0.4706338 * c.z;
        res.y = -0.5138850 * c.x + 1.4253036 * c.y + 0.0885814 * c.z;
        res.z = 0.0052982 * c.x - 0.0146949 * c.y + 1.0093968 * c.z;
        return res;
    }

// XYZ gaussian equations according to CIE 1964 http://jcgt.org/published/0002/02/01/paper.pdf
    vec3 wl2xyz(float lambda) {
        float x1 = 0.398 * exp(-1250.0 * pow(log((lambda + 570.1) / 1014.0), 2.0));
        float x2 = 1.132 * exp(-234.0 * pow(log((138.0 - lambda) / 743.5), 2.0));
        float y = 1.011 * exp(-0.5 * pow((lambda - 556.1) / 46.14, 2));
        float z = 2.060 * exp(-32.0 * pow(log((lambda - 265.8) / 180.4), 2));
        return vec3(x1 + x2, y, z);
    }

    vec3 wav2rgb(float w) {
        vec3 rgb = vec3(1.0, 0.0, 0.0);
        float intens = 0.0;
        if (w < 350.0) {
            rgb.x = 0.5;
            rgb.y = 0.0;
            rgb.z = 1.0;
        }
        else if (w >= 350.0 && w < 440.0) {
            rgb.x = (440.0 - w) / 90.0;
            rgb.y = 0.0;
            rgb.z = 1.0;
        }
        else if (w >= 440 && w <= 490.0) {
            rgb.x = 0.0;
            rgb.y = (w - 440.0) / 50.0;
            rgb.z = 1.0;
        }
        else if (w >= 490 && w < 510) {
            rgb.x = 0.0;
            rgb.y = 1.0;
            rgb.z = -(w - 510.) / 20.;
        }
        else if (w >= 510 && w < 580) {
            rgb.x = (w - 510.) / 70.0;
            rgb.y = 1.0;
            rgb.z = 0.0;
        }
        else if (w >= 580.0 && w < 645.0) {
            rgb.x = 1.0;
            rgb.y = -(w - 645.0) / 65.0;
            rgb.z = 0.0;
        }


        if (w < 350.0) {
            intens = 0.3;
        }
        else if (w >= 350 && w < 420) {
            intens = 0.3 + 0.7 * (w - 350.0) / 70.0;
        }
        else if (w >= 420 && w <= 700) {
            intens = 1.0;
        }
        else if (w > 700 && w <= 780) {
            intens = 0.3 + 0.7 * (780.0 - w) / 80.0;
        }
        else {
            intens = 0.3;
        }

        return vec3(intens * rgb.x,
                    intens * rgb.y,
                    intens * rgb.z);
    }


    void main() {
      vec3 L = normalize(light_position - ws_position);
      vec3 E = normalize(-ws_position);

      //calculate Ambient Term:
      vec4 Iamb = vec4(1.0, 1.0, 1.0, 1.0);

      //calculate Diffuse Term:
      vec4 Idiff1 = vec4(1.0, 1.0, 1.0, 1.0) * max(dot(ws_normal,L), 0.0);
      Idiff1      = clamp(Idiff1, 0.0, 1.0);

      // double sided lighting:
      vec4 Idiff2 = vec4(1.0, 1.0, 1.0, 1.0) * max(dot(-ws_normal,L), 0.0);
      Idiff2      = clamp(Idiff2, 0.0, 1.0);

      vec4 tex_color              = texture2D(tex, tex_coord);
      vec4 non_relativistic_color = tex_color * (vec4(color, 1.0) + Iamb + (Idiff1 + Idiff2) / 2) / 3;



      // apply doppler effect here, on `non_relativistic_color`

      vec3 diff = normalize(ws_position - player_position);


      float shift_coef = 1.0;
      float v_norm = sqrt(dot(rel_vel, rel_vel));
      float rel_v = dot(rel_vel, diff);

      vec4 real_color = non_relativistic_color;

      if (v_norm > 0.1) {
         shift_coef = sqrt((1.0 - (rel_v / light_vel)) /
                           (1.0 + (rel_v / light_vel)));

          float intens = sqrt(dot(real_color.xyz, real_color.xyz));
          vec3 ir_col = intens * wav2rgb(1500.0 * shift_coef);
          vec3 red_col = real_color.x * wav2rgb(700.0 * shift_coef);
          vec3 green_col = real_color.y * wav2rgb(510.0 * shift_coef);
          vec3 blue_col = real_color.z * wav2rgb(440.0 * shift_coef);
          vec3 uv_col = intens * wav2rgb(100.0 * shift_coef);

          vec3 rgb_col = (red_col + green_col + blue_col);
          rgb_col = clamp(rgb_col, 0.0, 1.0);
          real_color = vec4(rgb_col.x, rgb_col.y, rgb_col.z, 1.0);

      }


      gl_FragColor =  real_color;
    }";
