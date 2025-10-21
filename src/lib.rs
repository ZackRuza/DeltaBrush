use wasm_bindgen::prelude::*;

mod geometry;
mod scene;

pub use geometry::{Vec3, Mesh};
pub use scene::{Scene, SceneObject, Transform, Material, MeshData};

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
