




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

    pub fn split_face(&mut self, face: FaceIndex) {
        // Split the face
        self.dirty = true;
    }

    pub fn to_mesh(self) -> Mesh {
        self.half_edge_mesh.to_mesh()
    }
}