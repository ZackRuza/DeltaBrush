use crate::{HalfEdgeMesh, Mesh, ModelWrapper};
use std::string::String;

/// Trait for mesh representations that can be edited and rendered
pub trait ToMesh: Clone {
    fn to_mesh(&self) -> Mesh;
}

#[derive(Clone)]
pub enum ModelVariant {
    HalfEdgeMesh(ModelWrapper<HalfEdgeMesh>),
    Mesh(Mesh),
}

#[derive(Clone)]
pub struct ModelEntry {
    pub model: ModelVariant,
    pub name: String,
}

impl ModelVariant {
    pub fn get_mesh(&self) -> &Mesh {
        match self {
            ModelVariant::HalfEdgeMesh(hemw) => hemw.get_mesh(),
            ModelVariant::Mesh(m) => m,
        }
    }

    pub fn sync_render_mesh(&mut self) {
        match self {
            ModelVariant::HalfEdgeMesh(hemw) => hemw.sync_render_mesh(),
            ModelVariant::Mesh(_) => {
                // No-op: raw Mesh is already in render format
            }
        }
    }
}