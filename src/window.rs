use glfw;
use std::libc;
use std::sys;
use std::str;
use std::ptr;
use std::cast;
use glcore::*;
use glcore::consts::GL_VERSION_1_1::*;
use glcore::consts::GL_VERSION_2_0::*;
use glcore::functions::GL_VERSION_1_0::*;
use glcore::functions::GL_VERSION_2_0::*;
use glcore::types::GL_VERSION_1_5::*;
use nalgebra::traits::transpose::Transpose;
use nalgebra::mat::Mat4;
use object::Object;
use vertices::*;
use shaders::*;


pub struct Window
{
  objects: ~[@mut Object],
  window:  @mut glfw::Window
}

impl Window
{
  pub fn add_cube(&mut self) -> @mut Object
  {
    let res = @mut Object::new(cube_begin, cube_end, 1.0, 1.0, 1.0);

    self.objects.push(res);

    res
  }

  pub fn spawn(callback: ~fn(&mut Window))
  {
    do glfw::spawn {
      // The initialization is not really my code (see README)
      glfw::window_hint::context_version_major(3);
      glfw::window_hint::context_version_minor(2);
      glfw::window_hint::opengl_profile(glfw::OPENGL_CORE_PROFILE);
      glfw::window_hint::opengl_forward_compat(true);

      let window = @mut glfw::Window::create(800, 600, "kiss3d", glfw::Windowed).unwrap();

      window.make_context_current();

      // Create Vertex Array Object
      let vao: GLuint = 0;
      unsafe {
        glGenVertexArrays(1, &vao);
        glBindVertexArray(vao);
      }

      // Create a Vertex Buffer Object and copy the vertex data to it
      let vbo: GLuint = 0;
      unsafe {
        glGenBuffers(1, &vbo);
        glBindBuffer(GL_ARRAY_BUFFER, vbo);
        glBufferData(GL_ARRAY_BUFFER,
                     (vertices.len() * sys::size_of::<GLfloat>()) as GLsizeiptr,
                     cast::transmute(&vertices[0]),
                     GL_STATIC_DRAW);
      }

      // Create and compile the vertex shader
      let vertex_shader = unsafe { glCreateShader(GL_VERTEX_SHADER) };
      unsafe {
        glShaderSource(vertex_shader, 1, &str::as_c_str(vertex_src, |s|s), ptr::null());
        glCompileShader(vertex_shader);
      }

      // Create and compile the fragment shader
      let fragment_shader = unsafe { glCreateShader(GL_FRAGMENT_SHADER) };
      unsafe {
        glShaderSource(fragment_shader, 1, &str::as_c_str(fragment_src, |s|s), ptr::null());
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
      let pos_attrib = unsafe { glGetAttribLocation(shader_program, str::as_c_str("position", |s|s)) } as GLuint;
      unsafe {
        glEnableVertexAttribArray(pos_attrib);
        glVertexAttribPointer(pos_attrib, 3, GL_FLOAT, GL_FALSE,
                              3 * sys::size_of::<GLfloat>() as GLsizei,
                              ptr::null());
      }

      let mut usr_window = Window{ objects: ~[], window: window };
      callback(&mut usr_window);

      let color_location = unsafe {
        glGetUniformLocation(shader_program, str::as_c_str("color", |s| s))
      };

      let transform_location = unsafe {
        glGetUniformLocation(shader_program, str::as_c_str("transform", |s| s))
      };

      let proj_location = unsafe {
        glGetUniformLocation(shader_program, str::as_c_str("projection", |s| s))
      };

      window.set_size_callback(|win, w, h| { resize_callback(win, w as i32, h as i32, proj_location) });

      while !window.should_close() {
        // Poll events
        glfw::poll_events();

        // Clear the screen to black
        unsafe {
          glClearColor(0.1, 0.1, 0.1, 1.0);
          glClear(GL_COLOR_BUFFER_BIT);

          // Draw a triangle from the 3 vertices
          for usr_window.objects.iter().advance |o|
          { o.upload(color_location, transform_location) }
        }

        // Swap buffers
        window.swap_buffers();
      }

      unsafe {
        glDeleteProgram(shader_program);
        glDeleteShader(fragment_shader);
        glDeleteShader(vertex_shader);

        glDeleteBuffers(1, &vbo);

        glDeleteVertexArrays(1, &vao);
      }
    }
  }

  /*
  pub fn loop()
  {
    while !window.should_close()
    { glfw::poll_events(); }
  }
  */
}

fn resize_callback(_: &glfw::Window, w: i32, h: i32, proj_location: i32)
{
  let fov    = (45.0 as GLfloat).to_radians();
  let aspect = w as GLfloat / (h as GLfloat);
  let zfar   = 1024.0;
  let znear  = 1.0;

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

fn key_callback(window: &glfw::Window,
                key:    libc::c_int,
                _:      libc::c_int,
                action: libc::c_int,
                _:      libc::c_int)
{
    if action == glfw::PRESS && key == glfw::KEY_ESCAPE
    { window.set_should_close(true); }
}

fn error_callback(_: libc::c_int, description: ~str)
{ println(fmt!("Kiss3d Error: %s", description)); }
