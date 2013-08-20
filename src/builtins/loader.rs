use std::sys;
use std::cast;
use std::ptr;
use std::hashmap::HashMap;
use glcore::consts::GL_VERSION_1_1::*;
use glcore::consts::GL_VERSION_1_2::*;
use glcore::consts::GL_VERSION_1_3::*;
use glcore::consts::GL_VERSION_1_5::*;
use glcore::functions::GL_VERSION_1_0::*;
use glcore::functions::GL_VERSION_1_1::*;
use glcore::functions::GL_VERSION_1_3::*;
use glcore::functions::GL_VERSION_1_5::*;
use glcore::functions::GL_VERSION_2_0::*;
use glcore::types::GL_VERSION_1_5::*;
use glcore::types::GL_VERSION_1_0::*;
use object::GeometryIndices;
use shaders_manager::ObjectShaderContext;
use obj;
use builtins::cube_obj;
use builtins::sphere_obj;
use builtins::cone_obj;
use builtins::cylinder_obj;
use builtins::capsule_obj;

#[fixed_stack_segment] #[inline(never)]
pub fn load(ctxt: &ObjectShaderContext, textures: &mut HashMap<~str, GLuint>)
            -> HashMap<~str, GeometryIndices> {
    unsafe {
        let vertex_buf:  GLuint = 0;
        let element_buf: GLuint = 0;
        let normals_buf: GLuint = 0;
        let texture_buf: GLuint = 0;
        let default_tex: GLuint = 0;

        // FIXME: use glGenBuffers(3, ...) ?
        glGenBuffers(1, &vertex_buf);
        glGenBuffers(1, &element_buf);
        glGenBuffers(1, &normals_buf);
        glGenBuffers(1, &texture_buf);
        glGenTextures(1, &default_tex);

        textures.insert(~"default", default_tex);

        let (builtins, vbuf, nbuf, tbuf, vibuf) = parse_builtins(
            element_buf,
            normals_buf,
            vertex_buf,
            texture_buf); 

        // Upload values of vertices
        glBindBuffer(GL_ARRAY_BUFFER, vertex_buf);
        glBufferData(
            GL_ARRAY_BUFFER,
            (vbuf.len() * sys::size_of::<GLfloat>()) as GLsizeiptr,
            cast::transmute(&vbuf[0]),
            GL_STATIC_DRAW);

        // Upload values of indices
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, element_buf);
        glBufferData(
            GL_ELEMENT_ARRAY_BUFFER,
            (vibuf.len() * sys::size_of::<GLuint>()) as GLsizeiptr,
            cast::transmute(&vibuf[0]),
            GL_STATIC_DRAW);

        // Upload values of normals
        glBindBuffer(GL_ARRAY_BUFFER, normals_buf);
        glBufferData(
            GL_ARRAY_BUFFER,
            (nbuf.len() * sys::size_of::<GLfloat>()) as GLsizeiptr,
            cast::transmute(&nbuf[0]),
            GL_STATIC_DRAW);

        // Upload values of texture coordinates
        glBindBuffer(GL_ARRAY_BUFFER, texture_buf);
        glBufferData(
            GL_ARRAY_BUFFER,
            (tbuf.len() * sys::size_of::<GLfloat>()) as GLsizeiptr,
            cast::transmute(&tbuf[0]),
            GL_STATIC_DRAW);

        // Specify the layout of the vertex data
        glEnableVertexAttribArray(ctxt.pos);
        glBindBuffer(GL_ARRAY_BUFFER, vertex_buf);
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, element_buf);
        glVertexAttribPointer(
            ctxt.pos,
            3,
            GL_FLOAT,
            GL_FALSE,
            3 * sys::size_of::<GLfloat>() as GLsizei,
            ptr::null());

        // Specify the layout of the normals data
        glEnableVertexAttribArray(ctxt.normal);
        glBindBuffer(GL_ARRAY_BUFFER, normals_buf);
        glVertexAttribPointer(
            ctxt.normal,
            3,
            GL_FLOAT,
            GL_FALSE,
            3 * sys::size_of::<GLfloat>() as GLsizei,
            ptr::null());

        glEnableVertexAttribArray(ctxt.tex_coord);
        glBindBuffer(GL_ARRAY_BUFFER, texture_buf);
        glVertexAttribPointer(
            ctxt.tex_coord,
            2,
            GL_FLOAT,
            GL_FALSE,
            2 * sys::size_of::<GLfloat>() as GLsizei,
            ptr::null());

        // create white texture
        // Black/white checkerboard

        let default_tex_pixels: [ GLfloat, ..3 ] = [
            1.0, 1.0, 1.0
            ];

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

        glUniform1i(ctxt.tex, 0);

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
