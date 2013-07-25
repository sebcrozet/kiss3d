use glfw;
use std::num::{Zero, One};
use std::uint;
use std::libc;
use std::sys;
use std::str;
use std::ptr;
use std::cast;
use std::hashmap::HashMap;
use glcore::consts::GL_VERSION_1_1::*;
use glcore::consts::GL_VERSION_1_2::*;
use glcore::consts::GL_VERSION_1_3::*;
use glcore::consts::GL_VERSION_1_5::*;
use glcore::consts::GL_VERSION_2_0::*;
use glcore::functions::GL_VERSION_1_0::*;
use glcore::functions::GL_VERSION_1_1::*;
use glcore::functions::GL_VERSION_1_3::*;
use glcore::functions::GL_VERSION_1_5::*;
use glcore::functions::GL_VERSION_2_0::*;
use glcore::functions::GL_VERSION_3_0::*;
use glcore::functions::GL_ARB_vertex_array_object::*;
use glcore::types::GL_VERSION_1_5::*;
use glcore::types::GL_VERSION_1_0::*;
use stb_image::image::*;
use nalgebra::traits::inv::Inv;
use nalgebra::traits::homogeneous::ToHomogeneous;
use nalgebra::traits::vec_cast::VecCast;
use nalgebra::traits::mat_cast::MatCast;
use nalgebra::traits::translation::Translation;
use nalgebra::traits::transpose::Transpose;
use nalgebra::mat::Mat4;
use nalgebra::vec::{Vec2, Vec3};
use camera::{Camera, ArcBall};
use object::{GeometryIndices, Object, VerticesNormalsTriangles, Deleted};
use builtins::sphere_obj;
use builtins::cube_obj;
use builtins::cone_obj;
use builtins::cylinder_obj;
use obj;
use event;
use shaders::*;
use arc_ball;

pub enum Light
{
  Absolute(Vec3<GLfloat>),
  StickToCamera
}

// XXX This file is too big. Refactoring is needed.

/// Structure representing a window and a 3D scene. It is the main interface with the 3d engine.
pub struct Window
{
  priv objects:               ~[@mut Object],
  priv light:                 i32,
  priv light_mode:            Light,
  priv window:                @mut glfw::Window,
  priv camera:                Camera,
  priv znear:                 f64,
  priv zfar:                  f64,
  priv textures:              HashMap<~str, GLuint>,
  priv geometries:            HashMap<~str, GeometryIndices>,
  priv usr_loop_callback:     @fn(),
  priv usr_keyboard_callback: @fn(&event::KeyboardEvent) -> bool,
  priv usr_mouse_callback:    @fn(&event::MouseEvent) -> bool,
  priv curr_wireframe_mode:   bool,
  priv background:            Vec3<GLfloat>,
  priv m_2d_to_3d:            Mat4<f64>
}

impl Window
{
  /// The `znear` value used by the perspective projection.
  pub fn znear(&self) -> f64
  { self.znear }

  /// The `zfar` value used by the perspective projection.
  pub fn zfar(&self) -> f64
  { self.zfar }

  /// The width of the window.
  pub fn width(&self) -> f64
  {
    let (w, _) = self.window.get_size();

    w as f64
  }

  /// The height of the window.
  pub fn height(&self) -> f64
  {
    let (_, h) = self.window.get_size();

    h as f64
  }

  /// Closes the window.
  pub fn close(@mut self)
  { self.window.set_should_close(true) }

  /// Hides the window, without closing it. Use `show` to make it visible again.
  pub fn hide(@mut self)
  { self.window.hide() }

  /// Makes the window visible. Use `hide` to hide it.
  pub fn show(@mut self)
  { self.window.show() }

  /// Switch on or off wireframe rendering mode. When set to `true`, everything in the scene will
  /// be drawn using wireframes. Wireframe rendering mode cannot be enabled on a per-object basis.
  pub fn set_wireframe_mode(@mut self, mode: bool)
  {
    if mode
    { unsafe { glPolygonMode(GL_FRONT_AND_BACK, GL_LINE) } }
    else
    { unsafe { glPolygonMode(GL_FRONT_AND_BACK, GL_FILL) } }

    self.curr_wireframe_mode = mode;
  }

  /// Sets the background color.
  pub fn set_background_color(@mut self, r: f64, g: GLfloat, b: f64)
  {
    self.background.x = r as GLfloat;
    self.background.y = g as GLfloat;
    self.background.z = b as GLfloat;
  }

  /// Adds a cube to the scene. The cube is initially axis-aligned and centered at (0, 0, 0).
  ///
  /// # Arguments
  ///   * `wx` - the cube extent along z axis
  ///   * `wy` - the cube extent along the y axis
  ///   * `wz` - the cube length along the z axis
  pub fn add_cube(@mut self, wx: GLfloat, wy: GLfloat, wz: GLfloat) -> @mut Object
  {
    // FIXME: this weird block indirection are here because of Rust issue #6248
    let res = {
      let geom = self.geometries.find(&~"cube").unwrap();
      let tex  = self.textures.find(&~"default").unwrap();
      @mut Object::new(self,
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
  pub fn add_sphere(@mut self, r: GLfloat) -> @mut Object
  {
    // FIXME: this weird block indirection are here because of Rust issue #6248
    let res = {
      let geom = self.geometries.find(&~"sphere").unwrap();
      let tex  = self.textures.find(&~"default").unwrap();
      @mut Object::new(self,
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
  pub fn add_cone(@mut self, h: GLfloat, r: GLfloat) -> @mut Object
  {
    // FIXME: this weird block indirection are here because of Rust issue #6248
    let res = {
      let geom = self.geometries.find(&~"cone").unwrap();
      let tex  = self.textures.find(&~"default").unwrap();
      @mut Object::new(self,
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
  pub fn add_cylinder(@mut self, h: GLfloat, r: GLfloat) -> @mut Object
  {
    // FIXME: this weird block indirection are here because of Rust issue #6248
    let res = {
      let geom = self.geometries.find(&~"cylinder").unwrap();
      let tex  = self.textures.find(&~"default").unwrap();
      @mut Object::new(self,
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
                  w:            f64,
                  h:            f64,
                  wsubdivs:     uint,
                  hsubdivs:     uint) -> @mut Object
  {
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
    for uint::iterate(0u, hsubdivs + 1) |i|
    {
      for uint::iterate(0u, wsubdivs + 1) |j|
      {
        vertices.push(Vec3::new(j as GLfloat * wstep - cw, i as GLfloat * hstep - ch, 0.0));
        tex_coords.push(Vec2::new(1.0 - j as GLfloat * wtexstep, 1.0 - i as GLfloat * htexstep))
      }
    }

    // create the normals
    for ((hsubdivs + 1) * (wsubdivs + 1)).times
    {
      { normals.push(Vec3::new(1.0 as GLfloat, 0.0, 0.0)) }
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
      }
    }

    // FIXME: refactor that to allow custom obj loading
    // create gpu buffers
    let vertex_buf:   GLuint = 0;
    let element_buf:  GLuint = 0;
    let normal_buf:   GLuint = 0;
    let texture_buf:  GLuint = 0;

    unsafe {
      // FIXME: use glGenBuffers(3, ...) ?
      glGenBuffers(1, &vertex_buf);
      glGenBuffers(1, &element_buf);
      glGenBuffers(1, &normal_buf);
      glGenBuffers(1, &texture_buf);
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

    // copy texture coordinates
    unsafe {
      glBindBuffer(GL_ARRAY_BUFFER, texture_buf);
      glBufferData(GL_ARRAY_BUFFER,
                   (tex_coords.len() * 2 * sys::size_of::<GLfloat>()) as GLsizeiptr,
                   cast::transmute(&tex_coords[0]),
                   GL_STATIC_DRAW);
    }


    // FIXME: this weird block indirection are here because of Rust issue #6248
    let res = {
    let tex = self.textures.find(&~"default").unwrap();
      @mut Object::new(
        self,
        GeometryIndices::new(0, triangles.len() * 3 as i32,
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
  pub fn add_texture(@mut self, path: ~str) -> GLuint
  {
    let tex: Option<GLuint> = self.textures.find(&path).map(|e| **e);

    match tex
    {
      Some(id) => id,
      None => {
        let texture: GLuint = 0;

        unsafe {
          glGenTextures(1, &texture);

          match load_with_depth(path.clone(), 3, false)
          {
            ImageU8(image) => {
              glActiveTexture(GL_TEXTURE0);
              glBindTexture(GL_TEXTURE_2D, texture);

              glTexImage2D(
                GL_TEXTURE_2D, 0,
                GL_RGB as GLint,
                image.width as GLsizei,
                image.height as GLsizei,
                0, GL_RGB, GL_UNSIGNED_BYTE,
                cast::transmute(&image.data[0])
                );

              glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE as GLint);
              glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE as GLint);
              glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as GLint);
              glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as GLint);
            }
            _ => { fail!("Failed to load texture " + path); }
          }
        }

        self.textures.insert(path.clone(), texture);

        texture
      }
    }
  }

  /// Retrieves the matrix which transforms a point from the 3d space to the normalized device
  /// coordinate space.
  pub fn space3d_to_space2d_matrix(&self) -> Mat4<f64>
  {
    // XXX: this is clearly not the best way to do that...
    self.m_2d_to_3d
    // XXX: ... in fact, it is better to recompute the value here (instead of recomputing it at
    // each frame). But doing so lead to a 'borrowed' dynamic task failure (and I do not know how
    // to fix this yet).
    // self.projection() * self.camera.transformation().inverse().unwrap().to_homogeneous()
  }

  /// The list of objects on the scene.
  pub fn objects<'r>(&'r self) -> &'r ~[@mut Object]
  { &self.objects }

  fn exec_callback(@mut self)
  { (self.usr_loop_callback)() }

  /// Sets the user-defined callback called at each event-pooling iteration of the engine.
  pub fn set_loop_callback(@mut self, callback: @fn())
  { self.usr_loop_callback = callback }

  /// Sets the user-defined callback called whenever a keyboard event is triggered. It is called
  /// before any specific event handling from the engine (e.g. for the camera).
  ///
  /// # Arguments
  ///   * callback - the user-defined keyboard event handler. If it returns `false`, the event will
  ///   not be further handled by the engine. Handlers overriding some of the default behaviour of
  ///   the engine typically return `false`.
  pub fn set_keyboard_callback(@mut self, callback: @fn(&event::KeyboardEvent) -> bool)
  { self.usr_keyboard_callback = callback }

  /// Sets the user-defined callback called whenever a mouse event is triggered. It is called
  /// before any specific event handling from the engine (e.g. for the camera).
  ///
  /// # Arguments
  ///   * callback - the user-defined mouse event handler. If it returns `false`, the event will
  ///   not be further handled by the engine. Handlers overriding some of the default behaviour of
  ///   the engine typically return `false`.
  pub fn set_mouse_callback(@mut self, callback: @fn(&event::MouseEvent) -> bool)
  { self.usr_mouse_callback = callback }

  /// Sets the light mode. Only one light is supported.
  pub fn set_light(@mut self, pos: Light)
  {
    match pos
    {
      Absolute(p)   => self.set_light_pos(&p),
      StickToCamera => {
        let camera_pos = self.camera.transformation().translation();
        self.set_light_pos(&VecCast::from(camera_pos))
      }
    }

    self.light_mode = pos;
  }

  fn set_light_pos(@mut self, pos: &Vec3<GLfloat>)
  { unsafe { glUniform3f(self.light, pos.x, pos.y, pos.z) } }

  /// The camera used to render the scene. Only one camera is supported.
  pub fn camera<'r>(&'r mut self) -> &'r mut Camera
  { &'r mut self.camera }

  fn parse_builtins(ebuf: GLuint, nbuf: GLuint, vbuf: GLuint, tbuf: GLuint)
    -> (HashMap<~str, GeometryIndices>, ~[GLfloat], ~[GLfloat], ~[GLfloat], ~[GLuint])
  {
    // FIXME: this function is _really_ uggly.

    // load
    let (cv, cn, ct, icv) = obj::parse(cube_obj::CUBE_OBJ);
    let (sv, sn, st, isv) = obj::parse(sphere_obj::SPHERE_OBJ);
    let (pv, pn, pt, ipv) = obj::parse(cone_obj::CONE_OBJ);
    let (yv, yn, yt, iyv) = obj::parse(cylinder_obj::CYLINDER_OBJ);

    let shift_isv = isv.map(|i| i + cv.len() / 3 as GLuint);
    let shift_ipv = ipv.map(|i| i + (sv.len() + cv.len()) / 3 as GLuint);
    let shift_iyv = iyv.map(|i| i + (sv.len() + cv.len() + pv.len()) / 3 as GLuint);

    // register draw informations
    let mut hmap = HashMap::new();

    hmap.insert(~"cube",     GeometryIndices::new(0, icv.len() as i32, ebuf, nbuf, vbuf, tbuf));
    hmap.insert(~"sphere",   GeometryIndices::new(icv.len(), isv.len() as i32, ebuf, nbuf, vbuf, tbuf));
    hmap.insert(~"cone",     GeometryIndices::new(icv.len() + isv.len(), ipv.len() as i32,
                                                  ebuf, nbuf, vbuf, tbuf));
    hmap.insert(~"cylinder", GeometryIndices::new(
        icv.len() + isv.len() + ipv.len(),
        iyv.len() as i32,
        ebuf, nbuf, vbuf, tbuf)
    );

    // concatenate everything
    (hmap,
     cv + sv + pv + yv,
     cn + sn + pn + yn,
     ct + st + pt + yt,
     icv + shift_isv + shift_ipv + shift_iyv)
  }

  /// Opens a window and hide it. Once the window is created and before any event pooling, a
  /// user-defined callback is called once.
  ///
  /// This method contains an infinite loop and returns when the window is closed.
  ///
  /// # Arguments
  ///   * `title` - the window title
  ///   * `callback` - a callback called once the window has been created
  pub fn spawn_hidden(title: ~str, callback: ~fn(@mut Window))
  { Window::do_spawn(title, true, callback) }

  /// Opens a window. Once the window is created and before any event pooling, a user-defined
  /// callback is called once.
  ///
  /// This method contains an infinite loop and returns when the window is closed.
  ///
  /// # Arguments
  ///   * `title` - the window title
  ///   * `callback` - a callback called once the window has been created
  pub fn spawn(title: ~str, callback: ~fn(@mut Window))
  { Window::do_spawn(title, false, callback) }

  fn do_spawn(title: ~str, hide: bool, callback: ~fn(@mut Window))
  {
    glfw::set_error_callback(error_callback);

    do glfw::spawn {
      // The initialization is not really my code (see README)
      let window = @mut glfw::Window::create(800, 600, title, glfw::Windowed).unwrap();

      window.make_context_current();

      unsafe {
        glFrontFace(GL_CCW);
        // glEnable(GL_CULL_FACE);
        glEnable(GL_DEPTH_TEST);
        glDepthFunc(GL_LEQUAL);
      }

      // Create Vertex Array Object
      let vao: GLuint = 0;
      unsafe {
        glGenVertexArrays(1, &vao);
        glBindVertexArray(vao);
      }

      let vertex_buf:  GLuint = 0;
      let element_buf: GLuint = 0;
      let normals_buf: GLuint = 0;
      let texture_buf: GLuint = 0;
      let default_tex: GLuint = 0;

      unsafe {
        // FIXME: use glGenBuffers(3, ...) ?
        glGenBuffers(1, &vertex_buf);
        glGenBuffers(1, &element_buf);
        glGenBuffers(1, &normals_buf);
        glGenBuffers(1, &texture_buf);
        glGenTextures(1, &default_tex);
      }

      let mut hash_textures = HashMap::new();

      hash_textures.insert(~"default", default_tex);


      let (builtins, vbuf, nbuf, tbuf, vibuf) = Window::parse_builtins(element_buf,
                                                                       normals_buf,
                                                                       vertex_buf,
                                                                       texture_buf); 

      // Upload values of vertices
      unsafe {
        glBindBuffer(GL_ARRAY_BUFFER, vertex_buf);
        glBufferData(GL_ARRAY_BUFFER,
                     (vbuf.len() * sys::size_of::<GLfloat>()) as GLsizeiptr,
                     cast::transmute(&vbuf[0]),
                     GL_STATIC_DRAW);
      }

      // Upload values of indices
      unsafe {
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, element_buf);
        glBufferData(GL_ELEMENT_ARRAY_BUFFER,
                     (vibuf.len() * sys::size_of::<GLuint>()) as GLsizeiptr,
                     cast::transmute(&vibuf[0]),
                     GL_STATIC_DRAW);
      }

      // Upload values of normals
      unsafe {
        glBindBuffer(GL_ARRAY_BUFFER, normals_buf);
        glBufferData(GL_ARRAY_BUFFER,
                     (nbuf.len() * sys::size_of::<GLfloat>()) as GLsizeiptr,
                     cast::transmute(&nbuf[0]),
                     GL_STATIC_DRAW);
      }

      // Upload values of texture coordinates
      unsafe {
        glBindBuffer(GL_ARRAY_BUFFER, texture_buf);
        glBufferData(GL_ARRAY_BUFFER,
                     (tbuf.len() * sys::size_of::<GLfloat>()) as GLsizeiptr,
                     cast::transmute(&tbuf[0]),
                     GL_STATIC_DRAW);
      }

      // Create and compile the vertex shader
      let vertex_shader = unsafe { glCreateShader(GL_VERTEX_SHADER) };
      unsafe {
        glShaderSource(vertex_shader, 1, &VERTEX_SRC.as_c_str(|s| s), ptr::null());
        glCompileShader(vertex_shader);
      }

      check_shader_error(vertex_shader);

      // Create and compile the fragment shader
      let fragment_shader = unsafe { glCreateShader(GL_FRAGMENT_SHADER) };
      unsafe {
        glShaderSource(fragment_shader, 1, &FRAGMENT_SRC.as_c_str(|s| s), ptr::null());
        glCompileShader(fragment_shader);
      }

      check_shader_error(fragment_shader);

      // Link the vertex and fragment shader into a shader program
      let shader_program = unsafe { glCreateProgram() };
      unsafe {
        glAttachShader(shader_program, vertex_shader);
        glAttachShader(shader_program, fragment_shader);
        glBindFragDataLocation(shader_program, 0, "outColor".as_c_str(|s| s));
        glLinkProgram(shader_program);
        glUseProgram(shader_program);
      }

      // Specify the layout of the vertex data
      let pos_attrib = unsafe { glGetAttribLocation(shader_program, "position".as_c_str(|s| s)) } as GLuint;
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

      // Specify the layout of the normals data
      let normal_attrib = unsafe { glGetAttribLocation(shader_program, "normal".as_c_str(|s| s)) } as GLuint;
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
      let texture_attrib = unsafe { glGetAttribLocation(shader_program, "tex_coord_v".as_c_str(|s| s)) } as GLuint;
      unsafe {
        glEnableVertexAttribArray(texture_attrib);
        glBindBuffer(GL_ARRAY_BUFFER, texture_buf);
        glVertexAttribPointer(texture_attrib,
                              2,
                              GL_FLOAT,
                              GL_FALSE,
                              2 * sys::size_of::<GLfloat>() as GLsizei,
                              ptr::null());
      }

      // create white texture
      // Black/white checkerboard

      let default_tex_pixels: [ GLfloat, ..3 ] = [
        1.0, 1.0, 1.0
      ];

      unsafe {
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, default_tex);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_BASE_LEVEL, 0);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAX_LEVEL, 0);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR as i32);
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB as i32, 1, 1, 0, GL_RGB, GL_FLOAT,
                     cast::transmute(&default_tex_pixels[0]));
      }

      let light_location = unsafe {
        glGetUniformLocation(shader_program, "light_position".as_c_str(|s| s))
      };

      let usr_window = @mut Window {
        objects:       ~[],
        window:        window,
        zfar:          1024.0,
        znear:         0.1,
        camera:        Camera::new(ArcBall(arc_ball::ArcBall::new(-Vec3::z(), Zero::zero()))),
        usr_loop_callback:     || {},
        usr_keyboard_callback: |_| { true },
        usr_mouse_callback:    |_| { true },
        textures:              hash_textures,   
        light:                 light_location,
        light_mode:            Absolute(Vec3::new(0.0, 10.0, 0.0)),
        geometries:            builtins,
        curr_wireframe_mode:   false,
        background:            Vec3::new(0.0, 0.0, 0.0),
        m_2d_to_3d:            One::one()
      };

      callback(usr_window);

      usr_window.set_light(usr_window.light_mode);

      let color_location = unsafe {
        glGetUniformLocation(shader_program, "color".as_c_str(|s| s))
      };

      let transform_location = unsafe {
        glGetUniformLocation(shader_program, "transform".as_c_str(|s| s))
      };

      let scale_location = unsafe {
        glGetUniformLocation(shader_program, "scale".as_c_str(|s| s))
      };

      let normal_transform_location = unsafe {
        glGetUniformLocation(shader_program, "ntransform".as_c_str(|s| s))
      };

      let proj_location = unsafe {
        glGetUniformLocation(shader_program, "projection".as_c_str(|s| s))
      };

      let view_location = unsafe {
        glGetUniformLocation(shader_program, "view".as_c_str(|s| s))
      };

      unsafe {
        glUniform1i(glGetUniformLocation(shader_program, "tex".as_c_str(|s| s)), 0);
      };

      // setup callbacks
      window.set_key_callback(|_, a, b, c, d| usr_window.key_callback(a, b, c, d));
      window.set_mouse_button_callback(|_, b, a, m| usr_window.mouse_button_callback(b, a, m));
      window.set_scroll_callback(|_, xoff, yoff| usr_window.scroll_callback(xoff, yoff));  

      window.set_cursor_pos_callback(|_, xpos, ypos| usr_window.cursor_pos_callback(xpos, ypos));

      window.set_size_callback(|_, w, h| {
        unsafe { glViewport(0, 0, w as i32, h as i32) }

        let projection: Mat4<GLfloat> = MatCast::from(usr_window.projection().transposed());

        unsafe {
          glUniformMatrix4fv(proj_location,
          1,
          GL_FALSE,
          cast::transmute(&projection));
        }
      });

      window.set_size(800, 600);

      if hide
      { window.hide() }

      while !window.should_close() {
        // Poll events
        glfw::poll_events();

        usr_window.exec_callback();
        usr_window.camera.update();
        usr_window.camera.upload(view_location);
        usr_window.m_2d_to_3d = usr_window.camera.transformation().inverse().unwrap().to_homogeneous();

        match usr_window.light_mode
        {
          StickToCamera => usr_window.set_light(StickToCamera),
          _             => { }
        }

        // Clear the screen to black
        unsafe {
          glClearColor(
            usr_window.background.x,
            usr_window.background.y,
            usr_window.background.z,
            1.0);
          glClear(GL_COLOR_BUFFER_BIT);
          glClear(GL_DEPTH_BUFFER_BIT);

          for usr_window.objects.iter().advance |o|
          {
            o.upload(pos_attrib,
                     normal_attrib,
                     texture_attrib,
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
        glDeleteBuffers(1, &texture_buf);
        glDeleteBuffers(1, &normals_buf);
        glDeleteBuffers(1, &element_buf);

        glDeleteVertexArrays(1, &vao);
      }
    }
  }

  fn key_callback(@mut self,
                  key:    libc::c_int,
                  _:      libc::c_int,
                  action: libc::c_int,
                  _:      libc::c_int)
  {
    let event = if action == glfw::PRESS
                { event::KeyPressed(key) }
                else // if action == glfw::RELEASE
                { event::KeyReleased(key) };

    if !(self.usr_keyboard_callback)(&event)
    { return }

    if action == glfw::PRESS && key == glfw::KEY_ESCAPE
    { self.window.set_should_close(true); }

    if action == glfw::PRESS && key == glfw::KEY_SPACE
    { self.set_wireframe_mode(!self.curr_wireframe_mode); }

    self.camera.handle_keyboard(&event);
  }

  fn cursor_pos_callback(@mut self, xpos: float, ypos: float)
  {
    let event = event::CursorPos(xpos, ypos);

    if (self.usr_mouse_callback)(&event)
    { self.camera.handle_mouse(&event) }
  }

  fn scroll_callback(@mut self, xoff: float, yoff: float)
  {
    let event = event::Scroll(xoff, yoff);

    if (self.usr_mouse_callback)(&event)
    { self.camera.handle_mouse(&event) }
  }

  fn mouse_button_callback(@mut self,
                           button: libc::c_int,
                           action: libc::c_int,
                           mods:   libc::c_int)
  {
    let event = if action == 1
                { event::ButtonPressed(button, mods) }
                else
                { event::ButtonReleased(button, mods) };

    if !(self.usr_mouse_callback)(&event)
    { return }


    self.camera.handle_mouse(&event)
  }

  /// The projection matrix used by the window.
  pub fn projection(&self) -> Mat4<f64>
  {
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

fn check_shader_error(shader: GLuint)
{
  let compiles: i32 = 0;
  unsafe{
      glGetShaderiv(shader, GL_COMPILE_STATUS, &compiles);

      if(compiles == 0)
      {
        let info_log_len = 0;

        glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &info_log_len);

        if (info_log_len > 0)
        {
          // error check for fail to allocate memory omitted
          let chars_written = 0;
          let mut info_log = ~"";

          str::raw::set_len(&mut info_log, (info_log_len + 1) as uint);

          do info_log.as_c_str |c_str|
          { glGetShaderInfoLog(shader, info_log_len, &chars_written, c_str) }
          fail!("Shader compilation failed: " + info_log);
        }
      }
  }
}

fn error_callback(_: libc::c_int, description: ~str)
{ println(fmt!("Kiss3d Error: %s", description)); }
