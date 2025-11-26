use crate::{HalfEdgeMesh, Mesh, Model};

#[derive(Clone)]
pub struct MeshEditor {
    half_edge_mesh: HalfEdgeMesh,
    render_mesh: Mesh,
    dirty: bool,
}

impl MeshEditor {
    pub fn new(mesh: Mesh) -> Self {
        MeshEditor {
            half_edge_mesh: HalfEdgeMesh::from_mesh(&mesh),
            render_mesh: mesh,
            dirty: false,
        }
    }


    pub fn complete_editing(self) -> Mesh {
        if self.dirty {
            self.half_edge_mesh.to_mesh()
        } else {
            self.render_mesh
        }
    }
}

// Implement the trait for MeshEditor
impl Model for MeshEditor {
    fn get_mesh(&self) -> &Mesh {
        &self.render_mesh
    }

    fn sync_render_mesh(&mut self) {
        if self.dirty {
            // TODO: this is optimizable
            self.render_mesh = self.half_edge_mesh.to_mesh();
            self.dirty = false;
        }
    }
}