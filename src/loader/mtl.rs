//! Simplistic mtl loader.

use std::util;
use std::io::fs::File;
use std::io::Reader;
use std::str;
use std::str::Words;
use std::from_str::FromStr;
use nalgebra::na::Vec3;

fn error(line: uint, err: &str) -> ! {
    fail!("At line " + line.to_str() + ": " + err)
}

/// Parses a mtl file.
pub fn parse_file(path: &Path) -> Option<~[MtlMaterial]> {
    if !path.exists() {
        None
    }
    else {
        let s   = File::open(path).expect("Cannot open the file: " + path.as_str().unwrap()).read_to_end();
        let obj = str::from_utf8_owned(s).unwrap();
        Some(parse(obj))
    }
}

/// Parses a string representing a mtl file.
pub fn parse(string: &str) -> ~[MtlMaterial] {
    let mut res           = ~[];
    let mut curr_material = MtlMaterial::new_default(~"");

    for (l, line) in string.lines_any().enumerate() {
        let mut words = line.words();
        let tag       = words.next();

        match tag {
            None    => { },
            Some(w) => {
                if w.len() != 0 && w[0] != ('#' as u8) {
                    let mut p = line.words().peekable();
                    p.next();
                    if p.peek().is_none() {
                        continue
                    }

                    match w {
                        // texture name
                        &"newmtl"      => {
                            let old = util::replace(&mut curr_material, MtlMaterial::new_default(parse_name(l, words)));

                            if old.name.len() != 0 {
                                res.push(old);
                            }
                        },
                        // ambiant color
                            &"Ka"          => curr_material.ambiant = parse_color(l, words),
                            // diffuse color
                            &"Kd"          => curr_material.diffuse = parse_color(l, words),
                            // specular color
                            &"Ks"          => curr_material.specular = parse_color(l, words),
                            // shininess
                            &"Ns"          => curr_material.shininess = parse_scalar(l, words),
                            // alpha
                            &"d"           => curr_material.alpha = parse_scalar(l, words),
                            // ambiant map
                            &"map_Ka"      => curr_material.ambiant_texture = Some(parse_name(l, words)),
                            // diffuse texture map
                            &"map_Kd"      => curr_material.diffuse_texture = Some(parse_name(l, words)),
                            // specular texture map
                            &"map_Ks"      => curr_material.specular_texture = Some(parse_name(l, words)),
                            // specular texture map
                            &"map_d" | &"map_opacity" => curr_material.opacity_map = Some(parse_name(l, words)),
                            _     => {
                                println!("Warning: unknown line {} ignored: `{:s}'", l, line);
                            }
                    }
                }
            }
        }
    }

    if curr_material.name.len() != 0 {
        res.push(curr_material);
    }

    res
}

fn parse_name<'a>(_: uint, mut ws: Words<'a>) -> ~str {
    ws.to_owned_vec().connect(" ")
}

fn parse_color<'a>(l: uint, mut ws: Words<'a>) -> Vec3<f32> {
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

fn parse_scalar<'a>(l: uint, mut ws: Words<'a>) -> f32 {
    let sx = ws.next().unwrap_or_else(|| error(l, "1 component was expected, found 0."));

    let x: Option<f32> = FromStr::from_str(sx);

    let x = x.unwrap_or_else(|| error(l, "failed to parse `" + sx + "' as a f32."));

    x
}

/// Material informations read from a `.mtl` file.
#[deriving(Clone)]
pub struct MtlMaterial {
    /// Name of the material.
    name:             ~str,
    /// Path to the ambiant texture.
    ambiant_texture:  Option<~str>,
    /// Path to the diffuse texture.
    diffuse_texture:  Option<~str>,
    /// Path to the specular texture.
    specular_texture: Option<~str>,
    /// Path to the opacity map.
    opacity_map:      Option<~str>,
    /// The ambiant color.
    ambiant:          Vec3<f32>,
    /// The diffuse color.
    diffuse:          Vec3<f32>,
    /// The specular color.
    specular:         Vec3<f32>,
    /// The shininess.
    shininess:        f32,
    /// Alpha blending.
    alpha:            f32,
}

impl MtlMaterial {
    /// Creates a new mtl material with a name and default values.
    pub fn new_default(name: ~str) -> MtlMaterial {
        MtlMaterial {
            name:             name,
            shininess:        60.0,
            alpha:            1.0,
            ambiant_texture:  None,
            diffuse_texture:  None,
            specular_texture: None,
            opacity_map:      None,
            ambiant:          Vec3::new(1.0, 1.0, 1.0),
            diffuse:          Vec3::new(1.0, 1.0, 1.0),
            specular:         Vec3::new(1.0, 1.0, 1.0),
        }
    }

    /// Creates a new mtl material.
    pub fn new(name:             ~str,
               shininess:        f32,
               alpha:            f32,
               ambiant:          Vec3<f32>,
               diffuse:          Vec3<f32>,
               specular:         Vec3<f32>,
               ambiant_texture:  Option<~str>,
               diffuse_texture:  Option<~str>,
               specular_texture: Option<~str>,
               opacity_map:      Option<~str>)
               -> MtlMaterial {
        MtlMaterial {
            name:             name,
            ambiant:          ambiant,
            diffuse:          diffuse,
            specular:         specular,
            ambiant_texture:  ambiant_texture,
            diffuse_texture:  diffuse_texture,
            specular_texture: specular_texture,
            opacity_map:      opacity_map,
            shininess:        shininess,
            alpha:            alpha
        }
    }
}
