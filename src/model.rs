use crate::{Mesh, MeshEditor};

/// Trait for mesh representations that can be edited and rendered
pub trait Model: Clone {
    fn get_mesh(&self) -> &Mesh;
    fn sync_render_mesh(&mut self);
}

/// Enum that can hold different mesh representation types
/// This allows storing mixed mesh types in the same Vec while maintaining zero-cost dispatch
#[derive(Clone)]
pub enum ModelVariant {
    HalfEdge(MeshEditor),
}

impl Model for ModelVariant {
    fn get_mesh(&self) -> &Mesh {
        match self {
            ModelVariant::HalfEdge(editor) => editor.get_mesh(),
        }
    }

    fn sync_render_mesh(&mut self) {
        match self {
            ModelVariant::HalfEdge(editor) => editor.sync_render_mesh(),
        }
    }
}
