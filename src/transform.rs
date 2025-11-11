use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 4], // quaternion (x, y, z, w)
    pub scale: [f32; 3],
}
