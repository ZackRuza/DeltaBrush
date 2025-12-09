use serde::Serialize;
use crate::Transform;

/// Type-safe mesh ID to prevent confusion with other IDs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct MeshId(pub usize);

impl MeshId {
    pub fn new(id: usize) -> Self {
        MeshId(id)
    }
    
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

// Value retrieved by JavaScript
#[derive(Clone, Serialize)]
pub struct RenderInstance {
    pub mesh_id: MeshId,
    pub transform: Transform,
    pub id: usize,
}
