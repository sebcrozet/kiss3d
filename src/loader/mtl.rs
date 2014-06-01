//! Simplistic mtl loader.

use std::mem;
use std::io::fs::File;
use std::io::{IoResult, Reader};
use std::str::Words;
use std::from_str::FromStr;
use nalgebra::na::Vec3;

fn error(line: uint, err: &str) -> ! {
    fail!("At line {}: {}", line, err)
}

/// Parses a mtl file.
pub fn parse_file(path: &Path) -> IoResult<Vec<MtlMaterial>> {
    match File::open(path) {
        Ok(mut file) => file.read_to_str().map(|mtl| parse(mtl.as_slice())),
        Err(e)       => Err(e)
    }
}

/// Parses a string representing a mtl file.
pub fn parse(string: &str) -> Vec<MtlMaterial> {
    let mut res           = Vec::new();
    let mut curr_material = MtlMaterial::new_default("".to_string());

    for (l, line) in string.lines_any().enumerate() {
        let mut words = line.words();
        let tag       = words.next();

        match tag {
            None    => { },
            Some(w) => {
                if w.len() != 0 && w[0] != ('#' as u8) {
                    let mut p = line.words().peekable();
                    let     _ = p.next();

                    if p.peek().is_none() {
                        continue
                    }

                    match w {
                        // texture name
                        "newmtl"      => {
                            let old = mem::replace(&mut curr_material, MtlMaterial::new_default(parse_name(l, words)));

                            if old.name.len() != 0 {
                                res.push(old);
                            }
                        },
                        // ambiant color
                            "Ka"          => curr_material.ambiant = parse_color(l, words),
                            // diffuse color
                            "Kd"          => curr_material.diffuse = parse_color(l, words),
                            // specular color
                            "Ks"          => curr_material.specular = parse_color(l, words),
                            // shininess
                            "Ns"          => curr_material.shininess = parse_scalar(l, words),
                            // alpha
                            "d"           => curr_material.alpha = parse_scalar(l, words),
                            // ambiant map
                            "map_Ka"      => curr_material.ambiant_texture = Some(parse_name(l, words)),
                            // diffuse texture map
                            "map_Kd"      => curr_material.diffuse_texture = Some(parse_name(l, words)),
                            // specular texture map
                            "map_Ks"      => curr_material.specular_texture = Some(parse_name(l, words)),
                            // specular texture map
                            "map_d" | "map_opacity" => curr_material.opacity_map = Some(parse_name(l, words)),
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

fn parse_name<'a>(_: uint, mut ws: Words<'a>) -> String {
    let res: Vec<&'a str> = ws.collect();
    res.connect(" ")
}

fn parse_color<'a>(l: uint, mut ws: Words<'a>) -> Vec3<f32> {
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

fn parse_scalar<'a>(l: uint, mut ws: Words<'a>) -> f32 {
    let sx = ws.next().unwrap_or_else(|| error(l, "1 component was expected, found 0."));

    let x: Option<f32> = FromStr::from_str(sx);

    let x = x.unwrap_or_else(|| error(l, format!("failed to parse `{}' as a f32.", sx).as_slice()));

    x
}

/// Material informations read from a `.mtl` file.
#[deriving(Clone)]
pub struct MtlMaterial {
    /// Name of the material.
    pub name:             String,
    /// Path to the ambiant texture.
    pub ambiant_texture:  Option<String>,
    /// Path to the diffuse texture.
    pub diffuse_texture:  Option<String>,
    /// Path to the specular texture.
    pub specular_texture: Option<String>,
    /// Path to the opacity map.
    pub opacity_map:      Option<String>,
    /// The ambiant color.
    pub ambiant:          Vec3<f32>,
    /// The diffuse color.
    pub diffuse:          Vec3<f32>,
    /// The specular color.
    pub specular:         Vec3<f32>,
    /// The shininess.
    pub shininess:        f32,
    /// Alpha blending.
    pub alpha:            f32,
}

impl MtlMaterial {
    /// Creates a new mtl material with a name and default values.
    pub fn new_default(name: String) -> MtlMaterial {
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
    pub fn new(name:             String,
               shininess:        f32,
               alpha:            f32,
               ambiant:          Vec3<f32>,
               diffuse:          Vec3<f32>,
               specular:         Vec3<f32>,
               ambiant_texture:  Option<String>,
               diffuse_texture:  Option<String>,
               specular_texture: Option<String>,
               opacity_map:      Option<String>)
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
