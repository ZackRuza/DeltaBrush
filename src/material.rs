use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Material {
    pub color: [f32; 3],
    pub metalness: f32,
    pub roughness: f32,
}
