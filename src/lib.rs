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
mod model_wrapper;
mod model;
mod visitor;

pub use algebra::Vec3;
pub use mesh::Mesh;
pub use half_edge_mesh::{HalfEdgeMesh, Vertex, HalfEdge, Face, VertexIndex, HalfEdgeIndex, FaceIndex};
pub use scene::SceneAPI;
pub use scene_object::SceneObject;
pub use transform::Transform;
pub use transformable::Transformable;
pub use material::Material;
pub use geometry::Point3;
pub use model_wrapper::ModelWrapper;
pub use model::ToMesh;

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
