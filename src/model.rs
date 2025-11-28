use crate::{HalfEdgeMesh, Mesh, ModelWrapper};

/// Trait for mesh representations that can be edited and rendered
pub trait ToMesh: Clone {
    fn to_mesh(&self) -> Mesh;
}

#[derive(Clone)]
pub enum ModelVariant {
    HalfEdgeMesh(ModelWrapper<HalfEdgeMesh>),
}

impl ModelVariant {
    pub fn get_mesh(&self) -> &Mesh {
        match self {
            ModelVariant::HalfEdgeMesh(hemw) => hemw.get_mesh(),
        }
    }

    pub fn sync_render_mesh(&mut self) {
        match self {
            ModelVariant::HalfEdgeMesh(hemw) => hemw.sync_render_mesh(),
        }
    }
}