//! Simplistic obj loader.

use std::io::fs::File;
use std::io::Reader;
use std::vec;
use std::str;
use std::str::WordIterator;
use std::from_str::FromStr;
use std::hashmap::HashMap;
use extra::arc::Arc;
use nalgebra::na::{Vec3, Vec2, Indexable};
use mesh::{Mesh, Coord, Normal, UV, SharedImmutable};
use mtl::MtlMaterial;
use mtl;

fn error(line: uint, err: &str) -> ! {
    fail!("At line " + line.to_str() + ": " + err)
}

fn warn(line: uint, err: &str) {
    println("At line " + line.to_str() + ": " + err)
}

/// Parses an obj file.
pub fn parse_file(path: &Path, mtl_base_dir: &Path, basename: &str) -> Option<~[(~str, Mesh, Option<MtlMaterial>)]> {
    if !path.exists() {
        None
    }
    else {
        let s   = File::open(path).expect("Cannot open the file: " + path.as_str().unwrap()).read_to_end();
        let obj = str::from_utf8_owned(s);
        Some(parse(obj, mtl_base_dir, basename))
    }
}

/// Parses a string representing an obj file.
pub fn parse(string: &str, mtl_base_dir: &Path, basename: &str) -> ~[(~str, Mesh, Option<MtlMaterial>)] {
    let mut coords:     ~[Coord]            = ~[];
    let mut normals:    ~[Normal]           = ~[];
    let mut uvs:        ~[UV]               = ~[];
    let mut groups:     HashMap<~str, uint> = HashMap::new();
    let mut groups_ids: ~[~[Vec3<i32>]]     = ~[];
    let mut curr_group: uint                = 0;
    let mut ignore_normals                  = false;
    let mut ignore_uvs                      = false;
    let mut mtllib                          = HashMap::new();
    let mut group2mtl                       = HashMap::new();

    groups_ids.push(~[]);
    groups.insert(basename.to_owned(), 0);

    for (l, line) in string.lines_any().enumerate() {
        let mut words = line.words();
        let tag = words.next();
        match tag {
            None    => { },
            Some(w) => {
                if w.len() != 0 && w[0] != ('#' as u8) {
                    match w {
                        &"v"      => coords.push(parse_v_or_vn(l, words)),
                        &"vn"     => if !ignore_normals { normals.push(parse_v_or_vn(l, words)) },
                        &"f"      => parse_f(l, words, &mut ignore_uvs, &mut ignore_normals, &mut groups_ids, curr_group),
                        &"vt"     => if !ignore_uvs { uvs.push(parse_vt(l, words)) },
                        &"g"      => curr_group = parse_g(l, words, basename, &mut groups, &mut groups_ids),
                        &"mtllib" => parse_mtllib(l, words, mtl_base_dir, &mut mtllib),
                        &"usemtl" => curr_group = parse_usemtl(l, words, curr_group, &mtllib, &mut group2mtl, &mut groups, &mut groups_ids),
                        _         => {
                            println("Warning: unknown line " + l.to_str() + " ignored: `" + line + "'");
                        }
                    }
                }
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
        groups_ids,
        groups,
        group2mtl)
}

fn parse_usemtl<'a>(l:          uint,
                    mut ws:     WordIterator<'a>,
                    curr_group: uint,
                    mtllib:     &HashMap<~str, MtlMaterial>,
                    group2mtl:  &mut HashMap<uint, MtlMaterial>,
                    groups:     &mut HashMap<~str, uint>,
                    groups_ids: &mut ~[~[Vec3<i32>]])
                    -> uint {
    let mname = ws.to_owned_vec().connect(" ");
    let none  = "None";
    if mname.as_slice() != none.as_slice() {
        match mtllib.find(&mname) {
            None    => {
                warn(l, "could not find the material " + mname);

                curr_group
            },
            Some(m) => {
                if !group2mtl.contains_key(&curr_group) {
                    group2mtl.insert(curr_group, m.clone());
                    curr_group
                }
                else {
                    // multiple usemtls for one group
                    // NOTE: this is a violation of the obj specification, but we support it anyway
                    let g         = curr_group.to_str() + mname;
                    let new_group = parse_g(l, g.words(), "auto_generated_group_", groups, groups_ids);

                    group2mtl.insert(new_group, m.clone());

                    new_group
                }
            }
        }
    }
    else {
        curr_group
    }
}

fn parse_mtllib<'a>(l:            uint,
                    mut ws:       WordIterator<'a>,
                    mtl_base_dir: &Path,
                    mtllib:       &mut HashMap<~str, MtlMaterial>) {
    let filename = ws.to_owned_vec().connect(" ");

    let mut path = mtl_base_dir.clone();

    path.push(filename);

    let ms = mtl::parse_file(&path);

    match ms {
        Some(ms) =>
            for m in ms.move_iter() {
                mtllib.insert(m.name.clone(), m);
            },
        None => warn(l, "could not find the mtl file " + path.as_str().unwrap())
    }
}

fn parse_v_or_vn<'a>(l: uint, mut ws: WordIterator<'a>) -> Vec3<f32> {
    let sx = ws.next().unwrap_or_else(|| error(l, "3 components were expected, found 0."));
    let sy = ws.next().unwrap_or_else(|| error(l, "3 components were expected, found 1."));
    let sz = ws.next().unwrap_or_else(|| error(l, "3 components were expected, found 2."));

    let x: Option<f32> = FromStr::from_str(sx);
    let y: Option<f32> = FromStr::from_str(sy);
    let z: Option<f32> = FromStr::from_str(sz);

    let x = x.unwrap_or_else(|| error(l, "failed to parse `" + sx + "' as a f32."));
    let y = y.unwrap_or_else(|| error(l, "failed to parse `" + sy + "' as a f32."));
    let z = z.unwrap_or_else(|| error(l, "failed to parse `" + sz + "' as a f32."));

    Vec3::new(x, y, z)
}

fn parse_f<'a>(l:              uint,
               ws:             WordIterator<'a>,
               ignore_uvs:     &mut bool,
               ignore_normals: &mut bool,
               groups_ids:     &mut ~[~[Vec3<i32>]],
               curr_group:     uint) {
    // Four formats possible: v   v/t   v//n   v/t/n
    for (i, word) in ws.enumerate() {
        let mut curr_ids: Vec3<i32> = Bounded::max_value();

        for (i, w) in word.split('/').enumerate() {
            if i == 0 || w.len() != 0 {
                let idx: Option<i32> = FromStr::from_str(w);
                match idx {
                    Some(id) => curr_ids.set(i, id - 1),
                    None     => error(l, "failed to parse `" + w + "' as a i32.")
                }
            }
        }

        if i > 2 {
            // on the fly triangulation as trangle fan
            let g = &mut groups_ids[curr_group];
            let p1 = g[g.len() - i];
            let p2 = g[g.len() - 1];
            g.push(p1);
            g.push(p2);
        }

        if curr_ids.y == Bounded::max_value() {
            *ignore_uvs = true;
        }
        if curr_ids.z == Bounded::max_value() {
            *ignore_normals = true;
        }

        groups_ids[curr_group].push(curr_ids);
    }
}

fn parse_vt<'a>(l: uint, mut ws: WordIterator<'a>) -> UV {
    let _0 = "0.0";
    let sx  = ws.next().unwrap_or_else(|| error(l, "at least 2 components were expected, found 0."));
    let sy  = ws.next().unwrap_or_else(|| error(l, "at least 2 components were expected, found 1."));
    // let sz  = ws.next().unwrap_or(_0);

    let x: Option<f32> = FromStr::from_str(sx);
    let y: Option<f32> = FromStr::from_str(sy);
    // let z: Option<f32> = FromStr::from_str(sz);

    let x = x.unwrap_or_else(|| error(l, "failed to parse `" + sx + "' as a f32."));
    let y = y.unwrap_or_else(|| error(l, "failed to parse `" + sy + "' as a f32."));
    // let z = z.unwrap_or_else(|| error(l, "failed to parse `" + sz + "' as a f32."));

    Vec2::new(x, y)
}

fn parse_g<'a>(_:          uint,
               mut ws:     WordIterator<'a>,
               prefix:     &str,
               groups:     &mut HashMap<~str, uint>,
               groups_ids: &mut ~[~[Vec3<i32>]])
               -> uint {
    let suffix = ws.to_owned_vec().connect(" ");
    let name   = if suffix.len() == 0 { prefix.to_owned() } else { prefix + "/" + suffix };

    *groups.find_or_insert_with(name, |_| { groups_ids.push(~[]); groups_ids.len() - 1 })
}

fn reformat(coords:     ~[Coord],
            normals:    Option<~[Normal]>,
            uvs:        Option<~[UV]>,
            groups_ids: ~[~[Vec3<i32>]],
            groups:     HashMap<~str, uint>,
            group2mtl:  HashMap<uint, MtlMaterial>) -> ~[(~str, Mesh, Option<MtlMaterial>)] {
    let mut vt2id:  HashMap<Vec3<i32>, u32> = HashMap::new();
    let mut vertex_ids: ~[u32]      = ~[];
    let mut resc: ~[Coord]          = ~[];
    let mut resn: Option<~[Normal]> = normals.as_ref().map(|_| ~[]);
    let mut resu: Option<~[UV]>     = uvs.as_ref().map(|_| ~[]);
    let mut resfs: ~[~[Vec3<u32>]]  = ~[];
    let mut names: ~[~str]          = ~[];
    let mut mtls:  ~[Option<MtlMaterial>] = ~[];

    for (name, i) in groups.move_iter() {
        names.push(name);
        mtls.push(group2mtl.find(&i).map(|m| m.clone()));

        for point in groups_ids[i].iter() {
            let mut abs_point = *point;
            if abs_point.x < 0 { abs_point.x = coords.len() as i32 + abs_point.x + 1 };
            uvs.as_ref().map(|uvs| if abs_point.y < 0
                    { abs_point.y = uvs.len() as i32 + abs_point.y + 1 });
            normals.as_ref().map(|normals| if abs_point.z < 0
                        { abs_point.z = normals.len() as i32 + abs_point.z + 1 });

            let idx = match vt2id.find(&abs_point) {
                Some(i) => { vertex_ids.push(*i); None },
                None    => {
                    let idx = resc.len() as u32;
                    resc.push(coords[abs_point.x]);
                    resu.as_mut().map(|l| l.push(uvs.get_ref()[abs_point.y]));
                    resn.as_mut().map(|l| l.push(normals.get_ref()[abs_point.z]));

                    vertex_ids.push(idx);

                    Some(idx)
                }
            };

            idx.map(|i| vt2id.insert(abs_point, i));
        }

        let mut resf = vec::with_capacity(vertex_ids.len() / 3);

        assert!(vertex_ids.len() % 3 == 0);

        for f in vertex_ids.chunks(3) {
            resf.push(Vec3::new(f[0], f[1], f[2]))
        }

        resfs.push(resf);
        vertex_ids.clear();
    }

    let resc = SharedImmutable(Arc::new(resc));
    let resn = resn.map(|n| SharedImmutable(Arc::new(n)));
    let resu = resu.map(|u| SharedImmutable(Arc::new(u)));

    let mut meshes = ~[];
    for ((fs, name), mtl) in resfs.move_iter().zip(names.move_iter()).zip(mtls.move_iter()) {
        if fs.len() != 0 {
            let fs   = SharedImmutable(Arc::new(fs));
            let mesh = Mesh::new(resc.clone(), fs, resn.clone(), resu.clone(), false);
            meshes.push((name, mesh, mtl))
        }
    }

    meshes
}
