use std::uint;
use std::num::Zero;
use std::from_str::FromStr;
use std::hashmap::HashMap;
use glcore::types::GL_VERSION_1_0::*;
use nalgebra::vec::{Vec3, Vec2};

enum Mode {
  V,
  VN,
  VT,
  F,
  Unknown
}

type Vertex  = Vec3<GLfloat>;
type Normal  = Vec3<GLfloat>;
type Face    = Vec3<GLuint>;
type Texture = Vec2<GLfloat>;

fn error(line: uint, err: &str) -> !
{ fail!("At line " + line.to_str() + ": " + err) }

pub fn parse(string: &str) -> (~[GLfloat], ~[GLfloat], ~[GLfloat], ~[GLuint])
{
  let mut vertices: ~[Vertex]  = ~[];
  let mut normals:  ~[Normal]  = ~[];
  let mut faces:    ~[Face]    = ~[];
  let mut textures: ~[Texture] = ~[];

  for string.any_line_iter().enumerate().advance |(l, line)|
  {
    let mut mode       = Unknown;
    let mut num_parsed = 0;
    let mut curr_vertex: Vertex  = Zero::zero();
    let mut curr_normal: Normal  = Zero::zero();
    let mut curr_tex:    Texture = Zero::zero();

    for line.word_iter().enumerate().advance |(i, word)|
    {
      if i == 0
      {
        match word
        {
          &"v"  => mode = V,
          &"vn" => mode = VN,
          &"f"  => mode = F,
          &"vt" => mode = VT,
          _     => break
        }
      }
      else
      {
        match mode
        {
          V  => match FromStr::from_str::<GLfloat>(word)
                {
                  Some(v) => curr_vertex.at[i - 1] = v,
                  None    => error(l, "failed to parse `" + word + "' as a GLfloat.")
                },
          VN => match FromStr::from_str::<GLfloat>(word)
                {
                  Some(n) => curr_normal.at[i - 1] = n,
                  None    => error(l, "failed to parse `" + word + "' as a GLfloat.")
                },
          VT => match FromStr::from_str::<GLfloat>(word)
                {
                  Some(t) => curr_tex.at[i - 1] = t,
                  None    => error(l, "failed to parse `" + word + "' as a GLfloat.")
                },
          F  => {
            // Four formats possible:
            //    v
            //    v/t
            //    v//n
            //    v/t/n
            // with:
            // v = vertex
            // t = texture 
            // n = normal
            //
            // We need that each vertex has a normal and a tex coordinate (we concider the three
            // first formats as invalid).
            let words: ~[&str] = word.split_iter('/').collect();

            if words.len() != 3
            { error(l, "vertices without normal or texture informations are not supported.") }

            let mut curr_face: Face = Zero::zero();

            for uint::iterate(0u, 3) |i|
            {
              match FromStr::from_str::<GLuint>(words[i])
              {
                Some(id) => curr_face.at[i] = id - 1,
                None     => error(l, "failed to parse `" + words[i] + "' as a GLuint.")
              }
            }

            faces.push(curr_face);
          }
          _  => { }
        }
      }

      num_parsed = i;
    }

    
    if num_parsed != 0
    {
      match mode
      {
        V  => if num_parsed != 3 { error(l, "vertices must have 3 components.") },
        VN => if num_parsed != 3 { error(l, "normals must have 3 components.")  },
        F  => if num_parsed != 3 { error(l, "faces with more than 3 vertices are not supported.") },
        VT => if num_parsed != 2 { error(l, "texture coordinates must have 2 components.") },
        _  => { }
      }
    }

    match mode
    {
      V  => vertices.push(curr_vertex),
      VN => normals.push(curr_normal),
      VT => textures.push(curr_tex),
      _  => { }
    }
  }

  reformat(vertices, normals, textures, faces)
}

fn reformat(vertices: &[Vertex],
            normals:  &[Normal],
            textures: &[Texture],
            faces:    &[Face]) -> (~[GLfloat], ~[GLfloat], ~[GLfloat], ~[GLuint])
{
  let mut map:  HashMap<(GLuint, GLuint, GLuint), GLuint> = HashMap::new();
  let mut resv: ~[GLfloat] = ~[];
  let mut resn: ~[GLfloat] = ~[];
  let mut rest: ~[GLfloat] = ~[];
  let mut resi: ~[GLuint]  = ~[];

  for faces.iter().advance |face|
  {
    let key = (face.at[0], face.at[1], face.at[2]);

    let idx = match map.find(&key)
    {
      Some(i) => { resi.push(*i); None },
      None    => {
        let idx = resv.len() / 3 as GLuint;
        let v   = vertices[face.at[0]];
        let t   = textures[face.at[1]];
        let n   = normals[face.at[2]];

        resv.push(v.at[0]);
        resv.push(v.at[1]);
        resv.push(v.at[2]);

        resn.push(n.at[0]);
        resn.push(n.at[1]);
        resn.push(n.at[2]);

        rest.push(t.at[0]);
        rest.push(t.at[1]);

        resi.push(idx);

        Some(idx)
      }
    };

    match idx
    {
      Some(i) => { map.insert(key, i); },
      None    => { }
    }

  }

  (resv, resn, rest, resi)
}
