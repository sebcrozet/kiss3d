//! Simplistic obj loader.

use std::num::Bounded;
use std::io::fs::File;
use std::io::Reader;
use std::str::Words;
use std::from_str::FromStr;
use std::io::IoResult;
use std::collections::HashMap;
use std::collections::hashmap::{Occupied, Vacant};
use sync::{Arc, RWLock};
use gl::types::GLfloat;
use na::{Vec3, Pnt2, Pnt3, Indexable};
use na;
use resource::Mesh;
use loader::mtl::MtlMaterial;
use loader::mtl;
use resource::{GPUVector, StaticDraw, ArrayBuffer, ElementArrayBuffer};

pub type Coord  = Pnt3<GLfloat>;
pub type Normal = Vec3<GLfloat>;
pub type UV     = Pnt2<GLfloat>;

fn error(line: uint, err: &str) -> ! {
    fail!("At line {}: {:s}", line, err)
}

fn warn(line: uint, err: &str) {
    println!("At line {}: {:s}", line, err)
}

/// Parses an obj file.
pub fn parse_file(path: &Path, mtl_base_dir: &Path, basename: &str) -> IoResult<Vec<(String, Mesh, Option<MtlMaterial>)>> {
    match File::open(path) {
        Ok(mut file) => file.read_to_string().map(|obj| parse(obj.as_slice(), mtl_base_dir, basename)),
        Err(e)       => Err(e)
    }
}

/// Parses a string representing an obj file.
pub fn parse(string: &str, mtl_base_dir: &Path, basename: &str) -> Vec<(String, Mesh, Option<MtlMaterial>)> {
    let mut coords:     Vec<Coord>            = Vec::new();
    let mut normals:    Vec<Normal>           = Vec::new();
    let mut uvs:        Vec<UV>               = Vec::new();
    let mut groups:     HashMap<String, uint> = HashMap::new();
    let mut groups_ids: Vec<Vec<Vec3<u32>>>   = Vec::new();
    let mut curr_group: uint                  = 0;
    let mut ignore_normals                    = false;
    let mut ignore_uvs                        = false;
    let mut mtllib                            = HashMap::new();
    let mut group2mtl                         = HashMap::new();
    let mut curr_mtl                          = None::<MtlMaterial>;

    groups_ids.push(Vec::new());
    groups.insert(basename.to_string(), 0);

    for (l, line) in string.lines_any().enumerate() {
        let mut words = line.words();
        let tag = words.next();
        match tag {
            None    => { },
            Some(w) => {
                if w.len() != 0 && w.as_bytes()[0] != ('#' as u8) {
                    match w {
                        "v"      => coords.push(parse_v_or_vn(l, words).to_pnt()),
                        "vn"     => if !ignore_normals { normals.push(parse_v_or_vn(l, words)) },
                        "f"      => parse_f(l, words, coords.as_slice(), uvs.as_slice(), normals.as_slice(), &mut ignore_uvs, &mut ignore_normals, &mut groups_ids, curr_group),
                        "vt"     => if !ignore_uvs { uvs.push(parse_vt(l, words)) },
                        "g"      => {
                            curr_group = parse_g(l, words, basename, &mut groups, &mut groups_ids);
                            let _ = curr_mtl.as_ref().map(|mtl| group2mtl.insert(curr_group, mtl.clone()));
                        },
                        "mtllib" => parse_mtllib(l, words, mtl_base_dir, &mut mtllib),
                        "usemtl" => curr_group = parse_usemtl(l, words, curr_group, &mtllib, &mut group2mtl, &mut groups, &mut groups_ids, &mut curr_mtl),
                        _         => {
                            println!("Warning: unknown line {} ignored: `{:s}'", l, line);
                        }
                    }
                }
            }
        }
    }

    if !uvs.is_empty() && ignore_uvs {
        println!("Warning: some texture coordinates are missing. Dropping texture coordinates infos for every vertex.");
    }

    if !normals.is_empty() && ignore_normals {
        println!("Warning: some normals are missing. Dropping normals infos for every vertex.");
    }

    reformat(
        coords,
        if ignore_normals { None } else { Some(normals) },
        if ignore_uvs { None } else { Some(uvs) },
        groups_ids,
        groups,
        group2mtl)
}

fn parse_usemtl<'a>(l:          uint,
                    mut ws:     Words<'a>,
                    curr_group: uint,
                    mtllib:     &HashMap<String, MtlMaterial>,
                    group2mtl:  &mut HashMap<uint, MtlMaterial>,
                    groups:     &mut HashMap<String, uint>,
                    groups_ids: &mut Vec<Vec<Vec3<u32>>>,
                    curr_mtl:   &mut Option<MtlMaterial>)
                    -> uint {
    let mname: Vec<&'a str> = ws.collect();
    let mname = mname.connect(" ");
    let none  = "None";
    if mname.as_slice() != none.as_slice() {
        match mtllib.find(&mname) {
            None    => {
                *curr_mtl = None;
                warn(l, format!("could not find the material {}", mname).as_slice());

                curr_group
            },
            Some(m) => {
                if !group2mtl.contains_key(&curr_group) {
                    group2mtl.insert(curr_group, m.clone());
                    *curr_mtl = Some(m.clone());
                    curr_group
                }
                else {
                    // multiple usemtls for one group
                    // NOTE: this is a violation of the obj specification, but we support it anyway
                    let mut g = curr_group.to_string();
                    g.push_str(mname.as_slice());

                    let new_group = parse_g(l, g.as_slice().words(), "auto_generated_group_", groups, groups_ids);

                    group2mtl.insert(new_group, m.clone());
                    *curr_mtl = Some(m.clone());

                    new_group
                }
            }
        }
    }
    else {
        *curr_mtl = None;
        curr_group
    }
}

fn parse_mtllib<'a>(l:            uint,
                    mut ws:       Words<'a>,
                    mtl_base_dir: &Path,
                    mtllib:       &mut HashMap<String, MtlMaterial>) {
    let filename: Vec<&'a str> = ws.collect();
    let filename = filename.connect(" ");

    let mut path = mtl_base_dir.clone();

    path.push(filename);

    let ms = mtl::parse_file(&path);

    match ms {
        Ok(ms) =>
            for m in ms.into_iter() {
                mtllib.insert(m.name.to_string(), m);
            },
        Err(err) => warn(l, format!("{}", err).as_slice())
    }
}

fn parse_v_or_vn<'a>(l: uint, mut ws: Words<'a>) -> Vec3<f32> {
    let sx = ws.next().unwrap_or_else(|| error(l, "3 components were expected, found 0."));
    let sy = ws.next().unwrap_or_else(|| error(l, "3 components were expected, found 1."));
    let sz = ws.next().unwrap_or_else(|| error(l, "3 components were expected, found 2."));

    let x: Option<f32> = FromStr::from_str(sx);
    let y: Option<f32> = FromStr::from_str(sy);
    let z: Option<f32> = FromStr::from_str(sz);

    let x = x.unwrap_or_else(|| error(l, format!("failed to parse `{}' as a f32.", sx).as_slice()));
    let y = y.unwrap_or_else(|| error(l, format!("failed to parse `{}' as a f32.", sy).as_slice()));
    let z = z.unwrap_or_else(|| error(l, format!("failed to parse `{}' as a f32.", sz).as_slice()));

    Vec3::new(x, y, z)
}

fn parse_f<'a>(l:              uint,
               mut ws:         Words<'a>,
               coords:         &[Pnt3<f32>],
               uvs:            &[Pnt2<f32>],
               normals:        &[Vec3<f32>],
               ignore_uvs:     &mut bool,
               ignore_normals: &mut bool,
               groups_ids:     &mut Vec<Vec<Vec3<u32>>>,
               curr_group:     uint) {
    // Four formats possible: v   v/t   v//n   v/t/n
    let mut i = 0;
    for word in ws {
        let mut curr_ids: Vec3<i32> = Bounded::max_value();

        for (i, w) in word.split('/').enumerate() {
            if i == 0 || w.len() != 0 {
                let idx: Option<i32> = FromStr::from_str(w);
                match idx {
                    Some(id) => curr_ids.set(i, id - 1),
                    None     => error(l, format!("failed to parse `{}' as a i32.", w).as_slice())
                }
            }
        }

        if i > 2 {
            // on the fly triangulation as trangle fan
            let g = groups_ids.get_mut(curr_group);
            let p1 = (*g)[g.len() - i];
            let p2 = (*g)[g.len() - 1];
            g.push(p1);
            g.push(p2);
        }

        if curr_ids.y == Bounded::max_value() {
            *ignore_uvs = true;
        }

        if curr_ids.z == Bounded::max_value() {
            *ignore_normals = true;
        }

        // Handle relatives indice
        let x;
        let y;
        let z;

        if curr_ids.x < 0 {
            x = (coords.len() as i32 + curr_ids.x + 1) as u32;
        }
        else {
            x = curr_ids.x as u32;
        }

        if curr_ids.y < 0 {
            y = (uvs.len() as i32 + curr_ids.y + 1) as u32;
        }
        else {
            y = curr_ids.y as u32;
        }

        if curr_ids.z < 0 {
            z = (normals.len() as i32 + curr_ids.z + 1) as u32;
        }
        else {
            z = curr_ids.z as u32;
        }

        groups_ids.get_mut(curr_group).push(Vec3::new(x, y, z));

        i = i + 1;
    }

    // there is not enough vertex to form a triangle. Complete it.
    if i < 2 {
        for _ in range(0u, 3 - i) {
            let last = (*groups_ids)[curr_group].last().unwrap().clone();
            groups_ids.get_mut(curr_group).push(last);
        }
    }
}

fn parse_vt<'a>(l: uint, mut ws: Words<'a>) -> UV {
    let _0 = "0.0";
    let sx  = ws.next().unwrap_or_else(|| error(l, "at least 2 components were expected, found 0."));
    let sy  = ws.next().unwrap_or_else(|| error(l, "at least 2 components were expected, found 1."));
    // let sz  = ws.next().unwrap_or(_0);

    let x: Option<f32> = FromStr::from_str(sx);
    let y: Option<f32> = FromStr::from_str(sy);
    // let z: Option<f32> = FromStr::from_str(sz);

    let x = x.unwrap_or_else(|| error(l, format!("failed to parse `{}' as a f32.", sx).as_slice()));
    let y = y.unwrap_or_else(|| error(l, format!("failed to parse `{}' as a f32.", sy).as_slice()));
    // let z = z.unwrap_or_else(|| error(l, "failed to parse `" + sz + "' as a f32."));

    Pnt2::new(x, y)
}

fn parse_g<'a>(_:          uint,
               mut ws:     Words<'a>,
               prefix:     &str,
               groups:     &mut HashMap<String, uint>,
               groups_ids: &mut Vec<Vec<Vec3<u32>>>)
               -> uint {
    let suffix: Vec<&'a str> = ws.collect();
    let suffix = suffix.connect(" ");
    let name   = if suffix.len() == 0 { prefix.to_string() } else { format!("{}/{}", prefix, suffix) };

    match groups.entry(name) {
        Occupied(entry) => *entry.into_mut(),
        Vacant(entry)   => {
            groups_ids.push(Vec::new());

            let val = groups_ids.len() - 1;
            *entry.set(val)
        }
    }
}

fn reformat(coords:     Vec<Coord>,
            normals:    Option<Vec<Normal>>,
            uvs:        Option<Vec<UV>>,
            groups_ids: Vec<Vec<Vec3<u32>>>,
            groups:     HashMap<String, uint>,
            group2mtl:  HashMap<uint, MtlMaterial>)
            -> Vec<(String, Mesh, Option<MtlMaterial>)> {
    let mut vt2id:  HashMap<Vec3<u32>, u32> = HashMap::new();
    let mut vertex_ids: Vec<u32>            = Vec::new();
    let mut resc: Vec<Coord>                = Vec::new();
    let mut resn: Option<Vec<Normal>>       = normals.as_ref().map(|_| Vec::new());
    let mut resu: Option<Vec<UV>>           = uvs.as_ref().map(|_| Vec::new());
    let mut resfs: Vec<Vec<Vec3<u32>>>      = Vec::new();
    let mut allfs: Vec<Vec3<u32>>           = Vec::new();
    let mut names: Vec<String>              = Vec::new();
    let mut mtls:  Vec<Option<MtlMaterial>> = Vec::new();

    for (name, i) in groups.into_iter() {
        names.push(name);
        mtls.push(group2mtl.find(&i).map(|m| m.clone()));

        for point in groups_ids[i].iter() {
            let idx = match vt2id.find(point) {
                Some(i) => { vertex_ids.push(*i); None },
                None    => {
                    let idx = resc.len() as u32;

                    resc.push(coords[point.x as uint]);

                    let _ = resu.as_mut().map(|l| l.push((*uvs.as_ref().unwrap())[point.y as uint]));
                    let _ = resn.as_mut().map(|l| l.push((*normals.as_ref().unwrap())[point.z as uint]));

                    vertex_ids.push(idx);

                    Some(idx)
                }
            };

            let _ = idx.map(|i| vt2id.insert(point.clone(), i));
        }

        let mut resf = Vec::with_capacity(vertex_ids.len() / 3);

        assert!(vertex_ids.len() % 3 == 0);

        for f in vertex_ids.as_slice().chunks(3) {
            resf.push(Vec3::new(f[0], f[1], f[2]));
            allfs.push(Vec3::new(f[0], f[1], f[2]));
        }

        resfs.push(resf);
        vertex_ids.clear();
    }

    let resn = resn.unwrap_or_else(|| Mesh::compute_normals_array(resc.as_slice(), allfs.as_slice()));
    let resn = Arc::new(RWLock::new(GPUVector::new(resn, ArrayBuffer, StaticDraw)));
    let resu = resu.unwrap_or_else(|| Vec::from_elem(resc.len(), na::orig()));
    let resu = Arc::new(RWLock::new(GPUVector::new(resu, ArrayBuffer, StaticDraw)));
    let resc = Arc::new(RWLock::new(GPUVector::new(resc, ArrayBuffer, StaticDraw)));

    let mut meshes = Vec::new();
    for ((fs, name), mtl) in resfs.into_iter().zip(names.into_iter()).zip(mtls.into_iter()) {
        if fs.len() != 0 {
            let fs   = Arc::new(RWLock::new(GPUVector::new(fs, ElementArrayBuffer, StaticDraw)));
            let mesh = Mesh::new_with_gpu_vectors(resc.clone(), fs, resn.clone(), resu.clone());
            meshes.push((name, mesh, mtl))
        }
    }

    meshes
}
