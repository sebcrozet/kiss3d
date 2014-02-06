extern mod extra;
extern mod sync;
extern mod gl;
extern mod glfw;
extern mod kiss3d;
extern mod nalgebra;

use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;
use sync::RWArc;
use gl::types::{GLint, GLuint, GLfloat};
use nalgebra::na::{Vec2, Vec3, Mat3, Mat4, Rot3, Rotation, Translation};
use nalgebra::na;
use kiss3d::window::Window;
use kiss3d::event;
use kiss3d::text::Font;
use kiss3d::object::ObjectData;
use kiss3d::camera::{Camera, FirstPerson};
use kiss3d::light::{Light, Absolute, StickToCamera};
use kiss3d::resource::{Shader, ShaderAttribute, ShaderUniform, Material, Mesh};

fn main() {
    Window::spawn("Kiss3d: relativity", proc(window) {
        let eye                 = Vec3::new(0.0f32, 0.0, 200.0);
        let at                  = na::zero();
        let fov                 = 45.0f32.to_radians();
        let font                = Font::new(&Path::new("media/font/Inconsolata.otf"), 60);
        let mut first_person    = FirstPerson::new_with_frustrum(fov, 0.1, 100000.0, eye, at);
        let     context         = RWArc::new(Context::new(1.0, na::zero(), eye));
        let     acceleration    = 0.2f32;
        let     deceleration    = -0.9f32;
        let     material        = Rc::new(RefCell::new(~RelativisticMaterial::new(context.clone()) as ~Material));

        window.set_camera(&mut first_person as &mut Camera);
        window.set_framerate_limit(Some(60));

        /*
         * Setup the grid.
         */
        let width     = 2;
        let spacing   = 1000.0;
        let thickness = 10.0;
        let total     = (width - 1) as f32 * spacing;

        for i in range(0, width) {
            for j in range(0, width) {
                let x = i as f32 * spacing - total / 2.0;
                let y = j as f32 * spacing - total / 2.0;

                let mut c = window.add_cube(thickness, thickness, total);
                c.set_material(material.clone());
                c.append_translation(&Vec3::new(x, y, 0.0));

                let mut c = window.add_cube(thickness, total, thickness);
                c.set_material(material.clone());
                c.append_translation(&Vec3::new(x, 0.0, y));

                let mut c = window.add_cube(total, thickness, thickness);
                c.set_material(material.clone());
                c.append_translation(&Vec3::new(0.0, x, y));
            }
        }

        window.set_light(StickToCamera);
        first_person.set_move_step(0.0);

        /*
         * Render
         */
        window.render_loop(|w| {
            context.write(|c| {
                w.poll_events(|_, event| {
                    match *event {
                        event::KeyReleased(code) => {
                            if code == glfw::Key1 {
                                c.speed_of_light = c.speed_of_light + 0.1;
                            }
                            else if code == glfw::Key2 {
                                c.speed_of_light = (c.speed_of_light - 0.1).max(&0.1);
                            }
                        },
                        _ => { }
                    }

                    true
                });

                let sop = first_person.move_step() / 60.0;

                w.draw_text(format!("Speed of light: {}\nSpeed of player: {}", c.speed_of_light, sop),
                            &na::zero(), &font, &Vec3::new(1.0, 1.0, 1.0));

                let curr_acceleration;
                if w.glfw_window().get_key(glfw::KeyUp)    == glfw::Press ||
                   w.glfw_window().get_key(glfw::KeyDown)  == glfw::Press ||
                   w.glfw_window().get_key(glfw::KeyRight) == glfw::Press ||
                   w.glfw_window().get_key(glfw::KeyLeft)  == glfw::Press {
                       curr_acceleration = acceleration;
                }
                else {
                    curr_acceleration = deceleration;
                }

                let new_sop = (sop + curr_acceleration / 60.0).clamp(&0.0, &(c.speed_of_light - 0.1));

                first_person.set_move_step(new_sop * 60.0);

                let eye = first_person.eye();
                let at  = first_person.at();
                let dir = na::normalize(&(at - eye));

                c.speed_of_player = dir * new_sop;
                c.position        = first_person.eye();
            })
        })
    })
}

struct Context {
    speed_of_light:  f32,
    speed_of_player: Vec3<f32>,
    position:        Vec3<f32>
}

impl Context {
    pub fn new(speed_of_light: f32, speed_of_player: Vec3<f32>, position: Vec3<f32>) -> Context {
        Context {
            speed_of_light:  speed_of_light,
            speed_of_player: speed_of_player,
            position:        position
        }
    }
}

/// The default material used to draw objects.
pub struct RelativisticMaterial {
    priv context:         RWArc<Context>,
    priv shader:          Shader,
    priv pos:             ShaderAttribute<Vec3<f32>>,
    priv normal:          ShaderAttribute<Vec3<f32>>,
    priv tex_coord:       ShaderAttribute<Vec2<f32>>,
    priv light:           ShaderUniform<Vec3<f32>>,
    priv color:           ShaderUniform<Vec3<f32>>,
    priv transform:       ShaderUniform<Mat4<f32>>,
    priv scale:           ShaderUniform<Mat3<f32>>,
    priv ntransform:      ShaderUniform<Mat3<f32>>,
    priv view:            ShaderUniform<Mat4<f32>>,
    priv tex:             ShaderUniform<GLuint>,
    priv light_vel:       ShaderUniform<GLfloat>,
    priv rel_vel:         ShaderUniform<Vec3<f32>>,
    priv rot:             ShaderUniform<Rot3<f32>>,
    priv player_position: ShaderUniform<Vec3<f32>>
}

impl RelativisticMaterial {
    /// Creates a new `RelativisticMaterial`.
    pub fn new(context: RWArc<Context>) -> RelativisticMaterial {
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
            tex:             shader.get_uniform("tex").unwrap(),
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
              pass:   uint,
              camera: &mut Camera,
              light:  &Light,
              data:   &ObjectData,
              mesh:   &mut Mesh) {
        self.activate();

        /*
         *
         * Setup camera and light.
         *
         */
        camera.upload(pass, &mut self.view);

        let pos = match *light {
            Absolute(ref p) => p.clone(),
            StickToCamera   => camera.eye()
        };

        self.light.upload(&pos);

        self.context.read(|c| {
            // XXX: this relative velocity est very wrong!
            self.rel_vel.upload(&-c.speed_of_player);
            self.light_vel.upload(&-c.speed_of_light);

            let mut rot = na::one::<Rot3<f32>>();

            if na::sqnorm(&c.speed_of_player) != 0.0 {
                rot.look_at_z(&(-c.speed_of_player), &Vec3::y());
                println!("speed: {:?}", c.speed_of_player);
                println!("{:?}", rot);
            }

            self.rot.upload(&rot);
        });

        /*
         *
         * Setup object-related stuffs.
         *
         */
        let formated_transform:  Mat4<f32> = na::to_homogeneous(data.transform());
        let formated_ntransform: Mat3<f32> = *data.transform().rotation.submat();

        unsafe {
            self.transform.upload(&formated_transform);
            self.ntransform.upload(&formated_ntransform);
            self.scale.upload(data.scale());
            self.color.upload(data.color());

            mesh.bind(&mut self.pos, &mut self.normal, &mut self.tex_coord);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, data.texture().borrow().id());

            gl::DrawElements(gl::TRIANGLES,
                             mesh.num_pts() as GLint,
                             gl::UNSIGNED_INT,
                             ptr::null());
        }

        mesh.unbind();
        self.deactivate();
    }
}

pub static RELATIVISTIC_VERTEX_SRC:   &'static str =
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


        mat4 scale4 = mat4(scale);

        vec4 pos4   = transform * scale4 * vec4(position, 1.0);

        ws_position   =  rot * (pos4.xyz - player_position);
        ws_position.z /= (1.0 - sqrt(dot(rel_vel, rel_vel)));
        ws_position   =  ws_position * rot;

        gl_Position = view * vec4(player_position + ws_position, 1.0);

        ws_normal   = normalize(ntransform * scale * normal);
        tex_coord   = tex_coord_v;
    }";

pub static RELATIVISTIC_FRAGMENT_SRC: &'static str =
   "#version 120
    uniform vec3      color;
    uniform vec3      light_position;
    uniform sampler2D tex;
    uniform float     light_vel;
    uniform vec3      rel_vel;
    varying vec2      tex_coord;
    varying vec3      ws_normal;
    varying vec3      ws_position;
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

      gl_FragColor =  vec4(abs(rel_vel) * light_vel + 0.5, 1.0) * non_relativistic_color;
    }";
