use wasm_bindgen::prelude::*;

mod algebra;
mod mesh;
mod half_edge_mesh;
mod transform;
mod transformable;
mod material;
mod geometry;
mod scene;
mod scene_object;
mod algorithms;
mod mesh_editor;

pub use algebra::Vec3;
pub use mesh::Mesh;
pub use half_edge_mesh::{HalfEdgeMesh, Vertex, HalfEdge, Face, VertexIndex, HalfEdgeIndex, FaceIndex};
pub use scene::SceneAPI;
pub use transform::Transform;
pub use transformable::Transformable;
pub use material::Material;
pub use geometry::Point3;
pub use mesh_editor::MeshEditor;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => ($crate::log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("DeltaBrush Rust core initialized!");
}
