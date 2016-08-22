//! Simplistic mtl loader.

use std::mem;
use std::fs::File;
use std::io::Result as IoResult;
use std::io::Read;
use std::str::FromStr;
use std::path::Path;
use na::Vector3;
use loader::obj::Words;
use loader::obj;

fn error(line: usize, err: &str) -> ! {
    panic!("At line {}: {}", line, err)
}

/// Parses a mtl file.
pub fn parse_file(path: &Path) -> IoResult<Vec<MtlMaterial>> {
    match File::open(path) {
        Ok(mut file) => {
            let mut sfile = String::new();
            file.read_to_string(&mut sfile).map(|_| parse(&sfile[..]))
        },
        Err(e) => Err(e)
    }
}

/// Parses a string representing a mtl file.
pub fn parse(string: &str) -> Vec<MtlMaterial> {
    let mut res           = Vec::new();
    let mut curr_material = MtlMaterial::new_default("".to_string());

    for (l, line) in string.lines().enumerate() {
        let mut words = obj::split_words(line);
        let tag       = words.next();

        match tag {
            None    => { },
            Some(w) => {
                if w.len() != 0 && w.as_bytes()[0] != ('#' as u8) {
                    let mut p = obj::split_words(line).peekable();
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
                                println!("Warning: unknown line {} ignored: `{}'", l, line);
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

fn parse_name<'a>(_: usize, ws: Words<'a>) -> String {
    let res: Vec<&'a str> = ws.collect();
    res.join(" ")
}

fn parse_color(l: usize, mut ws: Words) -> Vector3<f32> {
    let sx = ws.next().unwrap_or_else(|| error(l, "3 components were expected, found 0."));
    let sy = ws.next().unwrap_or_else(|| error(l, "3 components were expected, found 1."));
    let sz = ws.next().unwrap_or_else(|| error(l, "3 components were expected, found 2."));

    let x: Result<f32, _> = FromStr::from_str(sx);
    let y: Result<f32, _> = FromStr::from_str(sy);
    let z: Result<f32, _> = FromStr::from_str(sz);

    let x = x.unwrap_or_else(|e| error(l, &format!("failed to parse `{}' as a f32: {}", sx, e)[..]));
    let y = y.unwrap_or_else(|e| error(l, &format!("failed to parse `{}' as a f32: {}", sy, e)[..]));
    let z = z.unwrap_or_else(|e| error(l, &format!("failed to parse `{}' as a f32: {}", sz, e)[..]));

    Vector3::new(x, y, z)
}

fn parse_scalar(l: usize, mut ws: Words) -> f32 {
    let sx = ws.next().unwrap_or_else(|| error(l, "1 component was expected, found 0."));

    let x: Result<f32, _> = FromStr::from_str(sx);

    let x = x.unwrap_or_else(|e| error(l, &format!("failed to parse `{}' as a f32: {}", sx, e)[..]));

    x
}

/// Material informations read from a `.mtl` file.
#[derive(Clone)]
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
    pub ambiant:          Vector3<f32>,
    /// The diffuse color.
    pub diffuse:          Vector3<f32>,
    /// The specular color.
    pub specular:         Vector3<f32>,
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
            ambiant:          Vector3::new(1.0, 1.0, 1.0),
            diffuse:          Vector3::new(1.0, 1.0, 1.0),
            specular:         Vector3::new(1.0, 1.0, 1.0),
        }
    }

    /// Creates a new mtl material.
    pub fn new(name:             String,
               shininess:        f32,
               alpha:            f32,
               ambiant:          Vector3<f32>,
               diffuse:          Vector3<f32>,
               specular:         Vector3<f32>,
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
