extern crate kiss3d;
extern crate nalgebra as na;
extern crate ncollide3d;
extern crate rand;

use kiss3d::light::Light;
use kiss3d::loader::obj;
use kiss3d::resource::{AllocationType, BufferType, GPUVec, Mesh};
use kiss3d::window::Window;
use na::{Translation3, Vector3};
use rand::random;
use std::cell::RefCell;
use std::env;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Instant;

fn usage(exe_name: &str) {
    println!("Usage: {} obj_file scale clusters concavity", exe_name);
    println!("");
    println!("Options:");
    println!("    obj_file  - the obj file to decompose.");
    println!("    scale     - the scale to apply to the displayed model.");
    println!("    clusters  - minimum number of clusters.");
    println!("    concavity - the maximum concavity accepted by the decomposition.");
}

fn main() {
    /*
     * Parse arguments.
     */
    let mut args = env::args();
    let exname = args.next().unwrap();

    if args.len() != 4 {
        usage(&exname[..]);
        return;
    }

    let path = &args.next().unwrap()[..];
    let scale: f32 = FromStr::from_str(&args.next().unwrap()[..]).unwrap();
    let clusters: usize = FromStr::from_str(&args.next().unwrap()[..]).unwrap();
    let concavity: f32 = FromStr::from_str(&args.next().unwrap()[..]).unwrap();

    let scale = Vector3::from_element(scale);

    /*
     * Create the window.
     */
    let mut window = Window::new("Kiss3d: convex decomposition");

    /*
     * Convex decomposition.
     */
    let obj_path = Path::new(path);
    let mtl_path = Path::new("none");
    let teapot = obj::parse_file(&obj_path, &mtl_path, "none").unwrap();

    let mut m = window.add_obj(&obj_path, &mtl_path, scale);
    m.set_surface_rendering_activation(false);
    // m.set_lines_width(1.0);
    let data = m.data();
    let coords = data
        .object()
        .expect("here")
        .mesh()
        .borrow()
        .coords()
        .clone();
    let normals = data.object().unwrap().mesh().borrow().normals().clone();
    let uvs = data.object().unwrap().mesh().borrow().uvs().clone();
    let faces = data.object().unwrap().mesh().borrow().faces().clone();

    // println!("objs: {}", teapot.len());

    let mut total_time = 0.0f64;
    for &(_, ref mesh, _) in teapot.iter() {
        match mesh.to_trimesh() {
            Some(mut trimesh) => {
                trimesh.split_index_buffer(true);
                let begin = Instant::now();
                let (decomp, partitioning) =
                    ncollide3d::transformation::hacd(trimesh, concavity, clusters);
                let elapsed = begin.elapsed();
                total_time =
                    elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1000000000.0;

                println!("num comps: {}", decomp.len());

                for (comp, partitioning) in decomp.into_iter().zip(partitioning.into_iter()) {
                    let r = random();
                    let g = random();
                    let b = random();

                    let mut m = window.add_trimesh(comp, scale);
                    m.set_color(r, g, b);
                    m.append_translation(&Translation3::new(-0.1, 0.1, 0.0));
                    // m.set_surface_rendering_activation(false);
                    // m.enable_backface_culling(false);
                    // m.set_lines_width(1.0);

                    let mut part_faces = Vec::new();

                    for i in partitioning.into_iter() {
                        part_faces.push(faces.read().unwrap().data().as_ref().unwrap()[i]);
                    }

                    let faces = GPUVec::new(
                        part_faces,
                        BufferType::ElementArray,
                        AllocationType::StaticDraw,
                    );
                    let faces = Arc::new(RwLock::new(faces));

                    let mesh = Mesh::new_with_gpu_vectors(
                        coords.clone(),
                        faces,
                        normals.clone(),
                        uvs.clone(),
                    );
                    let mesh = Rc::new(RefCell::new(mesh));
                    let mut m = window.add_mesh(mesh, scale);
                    m.set_color(r, g, b);
                    m.append_translation(&Translation3::new(0.1, 0.1, 0.0));
                    // m.set_surface_rendering_activation(false);
                    // m.enable_backface_culling(false);
                    m.set_lines_width(1.0);
                }
            }
            None => {}
        }
    }

    println!("Decomposition time: {}", total_time);

    /*
     *
     * Rendering.
     *
     */
    window.set_light(Light::StickToCamera);

    while window.render() {}
}
