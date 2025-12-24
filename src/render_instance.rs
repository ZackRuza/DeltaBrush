use serde::Serialize;
use crate::Transform;
use uuid::Uuid;

/// Type-safe mesh ID using UUID to prevent index fragility
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct MeshId(pub Uuid);

impl MeshId {
    pub fn new() -> Self {
        MeshId(Uuid::new_v4())
    }
}

// Value retrieved by JavaScript
#[derive(Clone, Serialize)]
pub struct RenderInstance {
    pub mesh_id: MeshId,
    pub transform: Transform,
    pub id: usize,
    pub is_selected: bool,
}
