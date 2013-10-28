#[link(name     = "kiss3d"
       , vers   = "0.0"
       , author = "Sébastien Crozet"
       , uuid   = "0914a60f-37cd-40dc-9779-d72f237d99cc")];
#[crate_type = "lib"];
#[deny(non_camel_case_types)];
#[deny(non_uppercase_statics)];
#[deny(unnecessary_qualification)];
#[warn(missing_doc)];
#[feature(globs)];
#[feature(macro_rules)];
#[feature(managed_boxes)];

extern mod std;
extern mod extra;
extern mod glfw;
extern mod gl;
extern mod nalgebra;
extern mod stb_image;

pub mod window;
pub mod event;
pub mod object;
pub mod obj;
pub mod mesh;
pub mod camera;

/*
 * the user should not see/use the following modules
 */
#[doc(hidden)]
pub mod shaders;

#[doc(hidden)]
pub mod lines_manager;

#[doc(hidden)]
pub mod builtins
{
    pub mod loader;
    pub mod sphere_obj;
    pub mod cube_obj;
    pub mod cone_obj;
    pub mod cylinder_obj;
    pub mod capsule_obj;
}

/// A bunch of post-processing effects.
pub mod post_processing {
    pub mod post_processing_effect;
    pub mod waves;
    pub mod grayscales;
    pub mod sobel_edge_highlight;
    pub mod oculus_stereo;
}

/// GPU resource managers.
pub mod resources {
    pub mod framebuffers_manager;
    pub mod textures_manager;
    pub mod shaders_manager;
}

// pub mod draw {
//     pub mod depth_peeling;
// }
