use crate::{FaceIndex, HalfEdgeMesh, Mesh, Point3, Vertex, VertexIndex};





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

    /// Update mesh to match current state, then pass reference
    pub fn get_mesh(&self) -> &Mesh {
        todo!();
        // Sync render_mesh to be correct
        &self.render_mesh
    }

    pub fn complete_editing(self) -> Mesh {
        if self.dirty {
            self.half_edge_mesh.to_mesh()
        } else {
            self.render_mesh
        }
    }
    
}