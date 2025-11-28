use crate::{Mesh, model::ToMesh};

#[derive(Clone)]
pub struct ModelWrapper<M: ToMesh> {
    model: M,
    render_mesh: Mesh,
    dirty: bool,
}

impl<M: ToMesh> ModelWrapper<M> {
    pub fn new(model: M) -> Self {
        ModelWrapper {
            render_mesh: model.to_mesh(),
            model: model,
            dirty: false,
        }
    }

    pub fn get_mesh(&self) -> &Mesh {
        &self.render_mesh
    }

    pub fn sync_render_mesh(&mut self) {
        if self.dirty {
            // TODO: this is optimizable
            self.render_mesh = self.model.to_mesh();
            self.dirty = false;
        }
    }
}