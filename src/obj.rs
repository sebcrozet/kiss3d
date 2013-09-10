use std::vec;
use std::io;
use std::num::Zero;
use std::from_str::FromStr;
use std::hashmap::HashMap;
use gl::types::*;
use nalgebra::vec::{Vec3, Indexable};
use mesh::{Mesh, Coord, Vertex, Normal, UV};

enum Mode {
    V,
    VN,
    VT,
    F,
    Unknown
}

fn error(line: uint, err: &str) -> ! {
    fail!("At line " + line.to_str() + ": " + err)
}

/// Parses an obj file.
pub fn parse_file(path: &str) -> Mesh {
    parse(io::read_whole_file_str(&PosixPath(path)).expect("Unable to open the file: " + path))
}

/// Parses a string representing an obj file and returns (vertices, normals, texture coordinates, indices)
pub fn parse(string: &str) -> Mesh {
    let mut coords:  ~[Coord]        = ~[];
    let mut normals: ~[Normal]       = ~[];
    let mut mesh:    ~[Vec3<GLuint>] = ~[];
    let mut uvs:     ~[UV]           = ~[];

    for (l, line) in string.any_line_iter().enumerate() {
        let mut mode       = Unknown;
        let mut num_parsed = 0u;
        let mut curr_coords: Coord  = Zero::zero();
        let mut curr_normal: Normal = Zero::zero();
        let mut curr_tex:    UV     = Zero::zero();

        for (i, word) in line.word_iter().enumerate() {
            if i == 0 {
                match word {
                    &"v"  => mode = V,
                    &"vn" => mode = VN,
                    &"f"  => mode = F,
                    &"vt" => mode = VT,
                    _     => {
                        println("Warning: unknown line " + l.to_str() + " ignored: `" + line + "'");
                        break
                    }
                }
            }
            else {
                let word_val: Option<GLfloat> = FromStr::from_str(word);
                match mode {
                    V  => match word_val {
                        Some(v) => {
                            if i - 1 >= curr_coords.len() { error(l, "vertices must have 3 components.") }
                            curr_coords.set(i - 1, v)
                        },
                        None    => error(l, "failed to parse `" + word + "' as a GLfloat.")
                    },
                    VN => match word_val {
                        Some(n) => {
                            if i - 1 >= curr_normal.len() { error(l, "normals must have 3 components.") }
                            curr_normal.set(i - 1, n)
                        },
                        None    => error(l, "failed to parse `" + word + "' as a GLfloat.")
                    },
                    VT => match word_val {
                        Some(t) => {
                            if i - 1 >= curr_tex.len() { error(l, "texture coordinates must have 2 components.") }
                            curr_tex.set(i - 1, t)
                        },
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
                        // When the `t` or `n` coordinate is missing, we set `Bounded::max_value()`
                        // instead: they will be dealt with later.
                        let mut curr_ids: Vec3<GLuint> = Bounded::max_value();

                        for (i, w) in word.split_iter('/').enumerate() {
                            if i == 0 || w.len() != 0 {
                                let idx: Option<GLuint> = FromStr::from_str(w);
                                match idx {
                                    Some(id) => curr_ids.set(i, id - 1),
                                    None     => error(l, "failed to parse `" + w + "' as a GLuint.")
                                }
                            }
                        }

                        if i > 3 {
                            // on the fly triangulation as trangle fan
                            let p1 = mesh[mesh.len() - (i - 1)];
                            let p2 = mesh[mesh.len() - 1];
                            mesh.push(p1);
                            mesh.push(p2);
                        }

                        mesh.push(curr_ids);
                    }
                    _  => { }
                }
            }

            num_parsed = i;
        }


        if num_parsed != 0 {
            match mode {
                V  => if num_parsed != 3 { error(l, "vertices must have 3 components.") },
                VN => if num_parsed != 3 { error(l, "normals must have 3 components.")  },
                F  => if num_parsed < 3 { error(l, "faces must have at least 3 vertices.") },
                VT => if num_parsed != 2 { error(l, "texture coordinates must have 2 components.") },
                _  => { }
            }
        }

        match mode {
            V  => coords.push(curr_coords),
            VN => normals.push(curr_normal),
            VT => uvs.push(curr_tex),
            _  => { }
        }
    }

    let mut ignore_uvs     = false;
    let mut ignore_normals = false;

    for v in mesh.iter() {
        if v.y == Bounded::max_value() {
            ignore_uvs = true;

            if ignore_normals {
                break;
            }
        }

        if v.z == Bounded::max_value() {
            ignore_normals = true;

            if ignore_uvs {
                break;
            }
        }
    }

    if !uvs.is_empty() && ignore_uvs {
        println("Warning: some texture coordinates are missing. Dropping texture coordinates"
                + " infos for every vertex.");
    }

    if !normals.is_empty() && ignore_normals {
        println("Warning: some normals are missing. Dropping normals infos for every vertex.");
    }

    reformat(
        coords,
        if ignore_normals { None } else { Some(normals) },
        if ignore_uvs { None } else { Some(uvs) },
        mesh)
}

fn reformat(coords:  ~[Coord],
            normals: Option<~[Normal]>,
            uvs:     Option<~[UV]>,
            mesh:    ~[Vec3<GLuint>]) -> Mesh {
    let mut map:  HashMap<Vec3<GLuint>, GLuint> = HashMap::new();
    let mut vertex_ids: ~[Vertex]   = ~[];
    let mut resc: ~[Coord]          = ~[];
    let mut resn: Option<~[Normal]> = normals.map(|_| ~[]);
    let mut resu: Option<~[UV]>     = uvs.map(|_| ~[]);

    for point in mesh.iter() {
        let idx = match map.find(point) {
            Some(i) => { vertex_ids.push(*i); None },
            None    => {
                let idx = resc.len() as GLuint;
                resc.push(coords[point.x]);
                resu.map_mut(|l| l.push(uvs.get_ref()[point.y]));
                resn.map_mut(|l| l.push(normals.get_ref()[point.z]));

                vertex_ids.push(idx);

                Some(idx)
            }
        };

        idx.map(|i| map.insert(*point, *i));
    }

    let mut resf = vec::with_capacity(vertex_ids.len() / 3);

    for f in vertex_ids.chunk_iter(3) {
        resf.push(Vec3::new(f[0], f[1], f[2]))
    }

    Mesh::new(resc, resf, resn, resu, false)
}
