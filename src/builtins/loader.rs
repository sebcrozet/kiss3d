use std::sys;
use std::cast;
use std::ptr;
use std::hashmap::HashMap;
use gl;
use gl::types::*;
use object::GeometryIndices;
use shaders_manager::ObjectShaderContext;
use obj;
use builtins::cube_obj;
use builtins::sphere_obj;
use builtins::cone_obj;
use builtins::cylinder_obj;
use builtins::capsule_obj;

pub fn load(ctxt: &ObjectShaderContext, textures: &mut HashMap<~str, GLuint>)
            -> HashMap<~str, GeometryIndices> {
    unsafe {
        let vertex_buf:  GLuint = 0;
        let element_buf: GLuint = 0;
        let normals_buf: GLuint = 0;
        let texture_buf: GLuint = 0;
        let default_tex: GLuint = 0;

        // FIXME: use gl::GenBuffers(3, ...) ?
        gl::GenBuffers(1, &vertex_buf);
        gl::GenBuffers(1, &element_buf);
        gl::GenBuffers(1, &normals_buf);
        gl::GenBuffers(1, &texture_buf);
        gl::GenTextures(1, &default_tex);

        textures.insert(~"default", default_tex);

        let (builtins, vbuf, nbuf, tbuf, vibuf) = parse_builtins(
            element_buf,
            normals_buf,
            vertex_buf,
            texture_buf); 

        // Upload values of vertices
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buf);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vbuf.len() * sys::size_of::<GLfloat>()) as GLsizeiptr,
            cast::transmute(&vbuf[0]),
            gl::STATIC_DRAW);

        // Upload values of indices
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buf);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (vibuf.len() * sys::size_of::<GLuint>()) as GLsizeiptr,
            cast::transmute(&vibuf[0]),
            gl::STATIC_DRAW);

        // Upload values of normals
        gl::BindBuffer(gl::ARRAY_BUFFER, normals_buf);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (nbuf.len() * sys::size_of::<GLfloat>()) as GLsizeiptr,
            cast::transmute(&nbuf[0]),
            gl::STATIC_DRAW);

        // Upload values of texture coordinates
        gl::BindBuffer(gl::ARRAY_BUFFER, texture_buf);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (tbuf.len() * sys::size_of::<GLfloat>()) as GLsizeiptr,
            cast::transmute(&tbuf[0]),
            gl::STATIC_DRAW);

        // Specify the layout of the vertex data
        gl::EnableVertexAttribArray(ctxt.pos);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buf);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buf);
        gl::VertexAttribPointer(
            ctxt.pos,
            3,
            gl::FLOAT,
            gl::FALSE as u8,
            3 * sys::size_of::<GLfloat>() as GLsizei,
            ptr::null());

        // Specify the layout of the normals data
        gl::EnableVertexAttribArray(ctxt.normal);
        gl::BindBuffer(gl::ARRAY_BUFFER, normals_buf);
        gl::VertexAttribPointer(
            ctxt.normal,
            3,
            gl::FLOAT,
            gl::FALSE as u8,
            3 * sys::size_of::<GLfloat>() as GLsizei,
            ptr::null());

        gl::EnableVertexAttribArray(ctxt.tex_coord);
        gl::BindBuffer(gl::ARRAY_BUFFER, texture_buf);
        gl::VertexAttribPointer(
            ctxt.tex_coord,
            2,
            gl::FLOAT,
            gl::FALSE as u8,
            2 * sys::size_of::<GLfloat>() as GLsizei,
            ptr::null());

        // create white texture
        // Black/white checkerboard

        let default_tex_pixels: [ GLfloat, ..3 ] = [
            1.0, 1.0, 1.0
            ];

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, default_tex);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, 1, 1, 0, gl::RGB, gl::FLOAT,
        cast::transmute(&default_tex_pixels[0]));

        gl::Uniform1i(ctxt.tex, 0);

        builtins
    }
}

fn parse_builtins(ebuf: GLuint,
                  nbuf: GLuint,
                  vbuf: GLuint,
                  tbuf: GLuint)
                   -> (HashMap<~str, GeometryIndices>,
                       ~[GLfloat],
                       ~[GLfloat],
                       ~[GLfloat],
                       ~[GLuint]) {
    // load
    let (cv, cn, ct, icv) = obj::parse(cube_obj::CUBE_OBJ);
    let (sv, sn, st, isv) = obj::parse(sphere_obj::SPHERE_OBJ);
    let (pv, pn, pt, ipv) = obj::parse(cone_obj::CONE_OBJ);
    let (yv, yn, yt, iyv) = obj::parse(cylinder_obj::CYLINDER_OBJ);
    let (av, an, at, iav) = obj::parse(capsule_obj::CAPSULE_OBJ);

    let shift_isv = isv.map(|i| i + (cv.len() / 3) as GLuint);
    let shift_ipv = ipv.map(|i| i + ((sv.len() + cv.len()) / 3) as GLuint);
    let shift_iyv = iyv.map(|i| i + ((sv.len() + cv.len() + pv.len()) / 3) as GLuint);
    let shift_iav = iav.map(|i| i + ((sv.len() + cv.len() + pv.len() + yv.len()) / 3) as GLuint);

    // register draw informations
    let mut hmap = HashMap::new();

    hmap.insert(~"cube", GeometryIndices::new(0, icv.len() as i32, ebuf, nbuf, vbuf, tbuf));
    hmap.insert(~"sphere", GeometryIndices::new(icv.len(), isv.len() as i32, ebuf, nbuf, vbuf, tbuf));
    hmap.insert(~"cone", GeometryIndices::new(icv.len() + isv.len(), ipv.len() as i32,
    ebuf, nbuf, vbuf, tbuf));
    hmap.insert(
        ~"cylinder", GeometryIndices::new(
            icv.len() + isv.len() + ipv.len(),
            iyv.len() as i32,
            ebuf, nbuf, vbuf, tbuf)
        );
    hmap.insert(
        ~"capsule", GeometryIndices::new(
            icv.len() + isv.len() + ipv.len() + iyv.len(),
            iav.len() as i32,
            ebuf, nbuf, vbuf, tbuf)
        );

    // concatenate everything
    (hmap,
     cv + sv + pv + yv + av,
     cn + sn + pn + yn + an,
     ct + st + pt + yt + at,
     icv + shift_isv + shift_ipv + shift_iyv + shift_iav)
}
