//! Everything related to the scene graph.

pub use self::object::{Object, ObjectData};
pub use self::object2::{Object2, ObjectData2};
pub use self::scene_node::{SceneNode, SceneNodeData};
pub use self::scene_node2::{SceneNode2, SceneNodeData2};

mod object;
mod object2;
mod scene_node;
mod scene_node2;
