use wasm_bindgen::prelude::*;

mod algebra;
mod mesh;
mod transform;
mod material;
mod scene;
mod scene_object;
mod algorithms;

pub use algebra::Vec3;
pub use mesh::Mesh;
pub use scene::Scene;
pub use transform::Transform;
pub use material::Material;

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
