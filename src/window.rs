use glfw;
use std::uint;
use std::libc;
use std::sys;
use std::str;
use std::ptr;
use std::cast;
use std::hashmap::HashMap;
use glcore::*;
use nalgebra::traits::transpose::Transpose;
use nalgebra::mat::Mat4;
use nalgebra::vec::Vec3;
use camera::{Camera, ArcBall};
use object::{GeometryIndices, Object, VerticesTriangles};
use builtins::sphere_obj;
use builtins::cube_obj;
use builtins::cone_obj;
use builtins::cylinder_obj;
use obj;
use shaders::*;

pub enum Light
{
  Absolute(Vec3<GLfloat>),
  StickToCamera
}

pub struct Window
{
  priv objects:       ~[@mut Object],
  priv light:         i32,
  priv light_mode:    Light,
  priv window:        @mut glfw::Window,
  priv camera:        Camera,
  priv geometries:    HashMap<~str, GeometryIndices>,
  priv loop_callback: @fn(&mut Window)
}

impl Window
{
  pub fn add_cube(&mut self, wx: GLfloat, wy: GLfloat, wz: GLfloat) -> @mut Object
  {
    let res = @mut Object::new(*self.geometries.find(&~"cube").unwrap(),
                               1.0, 1.0, 1.0,
                               wx, wy, wz, None);
    // FIXME: get the geometry

    self.objects.push(res);

    res
  }

  pub fn add_sphere(&mut self, r: GLfloat) -> @mut Object
  {
    let res = @mut Object::new(*self.geometries.find(&~"sphere").unwrap(),
                               0.3, 0.3, 0.3,
                               r / 0.5, r / 0.5, r / 0.5,
                               None);
    // FIXME: get the geometry

    self.objects.push(res);

    res
  }

  pub fn add_cone(&mut self, h: GLfloat, r: GLfloat) -> @mut Object
  {
    let res = @mut Object::new(*self.geometries.find(&~"cone").unwrap(),
                               0.3, 0.3, 0.3,
                               r / 0.5, h, r / 0.5,
                               None);
    // FIXME: get the geometry

    self.objects.push(res);

    res
  }

  pub fn add_cylinder(&mut self, h: GLfloat, r: GLfloat) -> @mut Object
  {
    let res = @mut Object::new(*self.geometries.find(&~"cylinder").unwrap(),
                               0.3, 0.3, 0.3,
                               r / 0.5, h, r / 0.5,
                               None);
    // FIXME: get the geometry

    self.objects.push(res);

    res
  }

  pub fn add_quad(&mut self,
                  w:            f64,
                  h:            f64,
                  wsubdivs:     uint,
                  hsubdivs:     uint,
                  double_sided: bool) -> @mut Object
  {
    assert!(wsubdivs > 0 && hsubdivs > 0,
            "The number of subdivisions cannot be zero");

    let wstep = (w as GLfloat) / (wsubdivs as GLfloat);
    let hstep = (h as GLfloat) / (hsubdivs as GLfloat);
    let cw    = w as GLfloat / 2.0;
    let ch    = h as GLfloat / 2.0;

    let mut vertices = ~[];
    let mut normals  = ~[];
    let mut triangles: ~[(GLuint, GLuint, GLuint)] = ~[];

    // create the vertices
    for uint::iterate(0u, hsubdivs + 1) |i|
    {
      for uint::iterate(0u, wsubdivs + 1) |j|
      {
        vertices.push(Vec3::new([ j as GLfloat * wstep - cw,
                                  i as GLfloat * hstep - ch,
                                  0.0]))
      }
    }

    // create the normals
    for ((hsubdivs + 1) * (wsubdivs + 1)).times
    {
      { normals.push(Vec3::new([ 1.0 as GLfloat, 0.0, 0.0])) }
    }

    // create triangles
    fn dl_triangle(i: u32, j: u32, ws: u32) -> (u32, u32, u32)
    { ((i + 1) * ws + j, i * ws + j, (i + 1) * ws + j + 1) }

    fn ur_triangle(i: u32, j: u32, ws: u32) -> (u32, u32, u32)
    { (i * ws + j, i * ws + (j + 1), (i + 1) * ws + j + 1) }

    fn inv_wind((a, b, c): (u32, u32, u32)) -> (u32, u32, u32)
    { (b, a, c) }

    for uint::iterate(0u, hsubdivs) |i|
    {
      for uint::iterate(0u, wsubdivs) |j|
      {
        // build two triangles...
        triangles.push(dl_triangle(i as GLuint,
                                   j as GLuint,
                                   (wsubdivs + 1) as GLuint));
        triangles.push(ur_triangle(i as GLuint,
                                   j as GLuint,
                                   (wsubdivs + 1) as GLuint));
        // ... or four if we are double sided
        if double_sided
        {
          triangles.push(inv_wind(dl_triangle(i as GLuint,
                                              j as GLuint,
                                              (wsubdivs + 1) as GLuint)));
          triangles.push(inv_wind(ur_triangle(i as GLuint,
                                              j as GLuint,
                                              (wsubdivs + 1) as GLuint)));
        }
      }
    }

    // FIXME: refactor that to allow custom obj loading
    // create gpu buffers
    let vertex_buf:  GLuint = 0;
    let element_buf: GLuint = 0;
    let normal_buf:  GLuint = 0;

    unsafe {
      // FIXME: use glGenBuffers(3, ...) ?
      glGenBuffers(1, &vertex_buf);
      glGenBuffers(1, &element_buf);
      glGenBuffers(1, &normal_buf);
    }

    // copy vertices
    unsafe {
      glBindBuffer(GL_ARRAY_BUFFER, vertex_buf);
      glBufferData(GL_ARRAY_BUFFER,
                   (vertices.len() * 3 * sys::size_of::<GLfloat>()) as GLsizeiptr,
                   cast::transmute(&vertices[0]),
                   GL_DYNAMIC_DRAW);
    }

    // copy elements
    unsafe {
      glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, element_buf);
      glBufferData(GL_ELEMENT_ARRAY_BUFFER,
                   (triangles.len() * 3 * sys::size_of::<GLuint>()) as GLsizeiptr,
                   cast::transmute(&triangles[0]),
                   GL_STATIC_DRAW);
    }

    // copy normals
    unsafe {
      glBindBuffer(GL_ARRAY_BUFFER, normal_buf);
      glBufferData(GL_ARRAY_BUFFER,
                   (normals.len() * 3 * sys::size_of::<GLfloat>()) as GLsizeiptr,
                   cast::transmute(&normals[0]),
                   GL_DYNAMIC_DRAW);
    }


    let res = @mut Object::new(
      GeometryIndices::new(0, triangles.len() * 3 as i32,
                           element_buf, normal_buf, vertex_buf),
      0.3, 0.3, 0.3,
      1.0, 1.0, 1.0,
      Some(VerticesTriangles(vertices, triangles))
    );

    self.objects.push(res);

    res
  }

  pub fn objects<'r>(&'r self) -> &'r ~[@mut Object]
  { &self.objects }

  pub fn exec_callback(&mut self)
  { (self.loop_callback)(self) }

  pub fn set_loop_callback(&mut self, callback: @fn(&mut Window))
  { self.loop_callback = callback }

  pub fn set_light(&mut self, pos: Light)
  {
    let camera_pos = self.camera.position();
    match pos
    {
      Absolute(p)   => self.set_light_pos(&p),
      StickToCamera => self.set_light_pos(&camera_pos)
    }

    self.light_mode = pos;
  }

  fn set_light_pos(&mut self, pos: &Vec3<GLfloat>)
  { unsafe { glUniform3f(self.light, pos.at[0], pos.at[1], pos.at[2]) } }

  // FIXME: this is not very well supported yet
  // FIXME: pub fn set_camera(&mut self, mode: CameraMode)
  // FIXME: { self.camera.set_mode(mode) }

  fn parse_builtins(ebuf: GLuint, nbuf: GLuint, vbuf: GLuint)
    -> (HashMap<~str, GeometryIndices>, ~[GLfloat], ~[GLfloat], ~[GLuint])
  {
    // FIXME: this function is _really_ uggly.

    // load
    let (cv, cn, icv) = obj::parse(cube_obj::CUBE_OBJ);
    let (sv, sn, isv) = obj::parse(sphere_obj::SPHERE_OBJ);
    let (pv, pn, ipv) = obj::parse(cone_obj::CONE_OBJ);
    let (yv, yn, iyv) = obj::parse(cylinder_obj::CYLINDER_OBJ);

    let shift_isv = isv.map(|i| i + cv.len() / 3 as GLuint);
    let shift_ipv = ipv.map(|i| i + (sv.len() + cv.len()) / 3 as GLuint);
    let shift_iyv = iyv.map(|i| i + (sv.len() + cv.len() + pv.len()) / 3 as GLuint);

    // register draw informations
    let mut hmap = HashMap::new();

    hmap.insert(~"cube",     GeometryIndices::new(0, icv.len() as i32, ebuf, nbuf, vbuf));
    hmap.insert(~"sphere",   GeometryIndices::new(icv.len(), isv.len() as i32, ebuf, nbuf, vbuf));
    hmap.insert(~"cone",     GeometryIndices::new(icv.len() + isv.len(), ipv.len() as i32, ebuf,
                                                  nbuf, vbuf));
    hmap.insert(~"cylinder", GeometryIndices::new(
        icv.len() + isv.len() + ipv.len(),
        iyv.len() as i32,
        ebuf, nbuf, vbuf)
    );

    // concatenate everything
    (hmap,
     cv + sv + pv + yv,
     cn + sn + pn + yn,
     icv + shift_isv + shift_ipv + shift_iyv)
  }

  pub fn spawn(title: ~str, callback: ~fn(&mut Window))
  {
    glfw::set_error_callback(error_callback);

    do glfw::spawn {
      // The initialization is not really my code (see README)
      let window = @mut glfw::Window::create(800, 600, title, glfw::Windowed).unwrap();

      window.make_context_current();

      unsafe {
        glFrontFace(GL_CCW);
        glEnable(GL_CULL_FACE);
        glEnable(GL_DEPTH_TEST);
        glDepthFunc(GL_LEQUAL);
      }

      // Create Vertex Array Object
      let vao: GLuint = 0;
      unsafe {
        glGenVertexArrays(1, &vao);
        glBindVertexArray(vao);
      }

      let vertex_buf: GLuint = 0;
      let element_buf:  GLuint = 0;
      let normals_buf:  GLuint = 0;

      unsafe {
        // FIXME: use glGenBuffers(3, ...) ?
        glGenBuffers(1, &vertex_buf);
        glGenBuffers(1, &element_buf);
        glGenBuffers(1, &normals_buf);
      }

      let (builtins, vbuf, nbuf, vibuf) = Window::parse_builtins(element_buf, normals_buf,
                                                                 vertex_buf); 

      // Create a Vertex Buffer Object and copy the vertex data to it
      unsafe {
        glBindBuffer(GL_ARRAY_BUFFER, vertex_buf);
        glBufferData(GL_ARRAY_BUFFER,
                     (vbuf.len() * sys::size_of::<GLfloat>()) as GLsizeiptr,
                     cast::transmute(&vbuf[0]),
                     GL_STATIC_DRAW);
      }

      unsafe {
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, element_buf);
        glBufferData(GL_ELEMENT_ARRAY_BUFFER,
                     (vibuf.len() * sys::size_of::<GLuint>()) as GLsizeiptr,
                     cast::transmute(&vibuf[0]),
                     GL_STATIC_DRAW);
      }

      // Create a Vertex Buffer Object and copy the vertex data to it
      unsafe {
        glBindBuffer(GL_ARRAY_BUFFER, normals_buf);
        glBufferData(GL_ARRAY_BUFFER,
                     (nbuf.len() * sys::size_of::<GLfloat>()) as GLsizeiptr,
                     cast::transmute(&nbuf[0]),
                     GL_STATIC_DRAW);
      }

      // Create and compile the vertex shader
      let vertex_shader = unsafe { glCreateShader(GL_VERTEX_SHADER) };
      unsafe {
        glShaderSource(vertex_shader, 1, &str::as_c_str(VERTEX_SRC, |s|s), ptr::null());
        glCompileShader(vertex_shader);
      }

      // Create and compile the fragment shader
      let fragment_shader = unsafe { glCreateShader(GL_FRAGMENT_SHADER) };
      unsafe {
        glShaderSource(fragment_shader, 1, &str::as_c_str(FRAGMENT_SRC, |s|s), ptr::null());
        glCompileShader(fragment_shader);
      }

      // Link the vertex and fragment shader into a shader program
      let shader_program = unsafe { glCreateProgram() };
      unsafe {
        glAttachShader(shader_program, vertex_shader);
        glAttachShader(shader_program, fragment_shader);
        glBindFragDataLocation(shader_program, 0, str::as_c_str("outColor", |s|s));
        glLinkProgram(shader_program);
        glUseProgram(shader_program);
      }

      // Specify the layout of the vertex data
      let pos_attrib = unsafe { glGetAttribLocation(shader_program, str::as_c_str("position", |s| s)) } as GLuint;
      unsafe {
        glEnableVertexAttribArray(pos_attrib);
        glBindBuffer(GL_ARRAY_BUFFER, vertex_buf);
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, element_buf);
        glVertexAttribPointer(pos_attrib,
                              3,
                              GL_FLOAT,
                              GL_FALSE,
                              3 * sys::size_of::<GLfloat>() as GLsizei,
                              ptr::null());
      }

      let normal_attrib = unsafe { glGetAttribLocation(shader_program, str::as_c_str("normal", |s| s)) } as GLuint;
      unsafe {
        glEnableVertexAttribArray(normal_attrib);
        glBindBuffer(GL_ARRAY_BUFFER, normals_buf);
        glVertexAttribPointer(normal_attrib,
                              3,
                              GL_FLOAT,
                              GL_FALSE,
                              3 * sys::size_of::<GLfloat>() as GLsizei,
                              ptr::null());
      }

      let light_location = unsafe {
        glGetUniformLocation(shader_program, str::as_c_str("light_position", |s| s))
      };

      let usr_window = @mut Window {
        objects:       ~[],
        window:        window,
        camera:        Camera::new(
                          ArcBall(Vec3::new([2.0, 2.0, 2.0]),
                                  Vec3::new([0.0, 0.0, 0.0]),
                                  40.0)
                       ),
        loop_callback: |_| {},
        light:         light_location,
        light_mode:    Absolute(Vec3::new([0.0, 10.0, 0.0])),
        geometries:    builtins
      };

      callback(usr_window);

      usr_window.set_light(usr_window.light_mode);

      let color_location = unsafe {
        glGetUniformLocation(shader_program, str::as_c_str("color", |s| s))
      };

      let transform_location = unsafe {
        glGetUniformLocation(shader_program, str::as_c_str("transform", |s| s))
      };

      let scale_location = unsafe {
        glGetUniformLocation(shader_program, str::as_c_str("scale", |s| s))
      };

      let normal_transform_location = unsafe {
        glGetUniformLocation(shader_program, str::as_c_str("ntransform", |s| s))
      };

      let proj_location = unsafe {
        glGetUniformLocation(shader_program, str::as_c_str("projection", |s| s))
      };

      let view_location = unsafe {
        glGetUniformLocation(shader_program, str::as_c_str("view", |s| s))
      };

      // setup callbacks
      window.set_size_callback(|win, w, h| resize_callback(win, w as i32, h as i32, proj_location));
      window.set_key_callback(|_, a, b, c, d| usr_window.key_callback(a, b, c, d));
      window.set_mouse_button_callback(|_, b, a, m| usr_window.mouse_button_callback(b, a, m));
      window.set_cursor_pos_callback(|_, xpos, ypos| usr_window.cursor_pos_callback(xpos, ypos));
      window.set_scroll_callback(|_, xoff, yoff| usr_window.scroll_callback(xoff, yoff));  

      resize_callback(window, 800, 600, proj_location);

      // unsafe { glPolygonMode( GL_FRONT_AND_BACK, GL_LINE ); }
      while !window.should_close() {
        // Poll events
        glfw::poll_events();

        usr_window.exec_callback();
        usr_window.camera.upload(view_location);

        match usr_window.light_mode
        {
          StickToCamera => usr_window.set_light(StickToCamera),
          _             => { }
        }

        // Clear the screen to black
        unsafe {
          glClearColor(0.0, 0.0, 0.0, 1.0);
          glClear(GL_COLOR_BUFFER_BIT);
          glClear(GL_DEPTH_BUFFER_BIT);

          for usr_window.objects.iter().advance |o|
          {
            o.upload(pos_attrib,
                     normal_attrib,
                     color_location,
                     transform_location,
                     scale_location,
                     normal_transform_location)
          }
        }

        // Swap buffers
        window.swap_buffers();
      }

      unsafe {
        glDeleteProgram(shader_program);
        glDeleteShader(fragment_shader);
        glDeleteShader(vertex_shader);

        glDeleteBuffers(1, &vertex_buf);
        glDeleteBuffers(1, &normals_buf);

        glDeleteVertexArrays(1, &vao);
      }
    }
  }

  fn key_callback(&mut self,
                  key:    libc::c_int,
                  _:      libc::c_int,
                  action: libc::c_int,
                  _:      libc::c_int)
  {
      if action == glfw::PRESS && key == glfw::KEY_ESCAPE
      { self.window.set_should_close(true); }

      self.camera.handle_keyboard(key as int, action as int);
  }

  fn cursor_pos_callback(&mut self, xpos: float, ypos: float)
  { self.camera.handle_cursor_pos(xpos, ypos); }

  fn scroll_callback(&mut self, xoff: float, yoff: float)
  { self.camera.handle_scroll(xoff, yoff); }

  fn mouse_button_callback(&mut self,
                           button: libc::c_int,
                           action: libc::c_int,
                           mods:   libc::c_int)
  { self.camera.handle_mouse_button(button as int, action as int, mods as int); }
}

fn resize_callback(_: &glfw::Window, w: i32, h: i32, proj_location: i32)
{
  let fov    = (90.0 as GLfloat).to_radians();
  let aspect = w as GLfloat / (h as GLfloat);
  let zfar   = 1024.0;
  let znear  = 0.1;

  // adjust the viewport to the full window
  unsafe { glViewport(0, 0, w, h) }

  // adjust the projection transformation
  let mut proj = Mat4::new::<GLfloat>(
    [
      fov / aspect, 0.0,  0.0                            , 0.0,
      0.0         , fov, 0.0                             , 0.0,
      0.0         , 0.0,  (zfar + znear) / (znear - zfar), 2.0 * zfar * znear / (znear - zfar),
      0.0         , 0.0,  -1.0                           , 0.0
    ]);

  proj.transpose();

  unsafe {
    glUniformMatrix4fv(proj_location,
                       1,
                       GL_FALSE,
                       ptr::to_unsafe_ptr(&proj.mij[0]));
  }
}

fn error_callback(_: libc::c_int, description: ~str)
{ println(fmt!("Kiss3d Error: %s", description)); }
