use crate::geometry::Point3;

// Type-safe index wrappers (zero runtime cost)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexIndex(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HalfEdgeIndex(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FaceIndex(pub usize);

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Point3,
    // Index into half_edges to get started in traversal from vertex
    // Arbitrary entry point
    pub seed_half_edge: Option<HalfEdgeIndex>,
}

#[derive(Debug, Clone)]
pub struct HalfEdge {
    pub target_vertex_index: VertexIndex, // index in vertices
    pub twin_index: Option<HalfEdgeIndex>, // index in half_edges
    pub next_edge: HalfEdgeIndex, // (around the same face)
    pub prev_edge: HalfEdgeIndex, // (around the same face)
    pub face: Option<FaceIndex> // Face index (none for boundary case)
}


#[derive(Debug, Clone)]
pub struct Face {
    // Arbitrary entry point
    pub seed_half_edge: HalfEdgeIndex,
}



#[derive(Debug, Clone)]
pub struct HalfEdgeMesh {
    pub vertices: Vec<Vertex>,
    pub half_edges: Vec<HalfEdge>,
    pub faces: Vec<Face>,
}

impl HalfEdgeMesh {
    // Helper methods for safe indexing
    pub fn vertex(&self, idx: VertexIndex) -> &Vertex {
        &self.vertices[idx.0]
    }
    
    pub fn vertex_mut(&mut self, idx: VertexIndex) -> &mut Vertex {
        &mut self.vertices[idx.0]
    }
    
    pub fn half_edge(&self, idx: HalfEdgeIndex) -> &HalfEdge {
        &self.half_edges[idx.0]
    }
    
    pub fn half_edge_mut(&mut self, idx: HalfEdgeIndex) -> &mut HalfEdge {
        &mut self.half_edges[idx.0]
    }
    
    pub fn face(&self, idx: FaceIndex) -> &Face {
        &self.faces[idx.0]
    }
    
    pub fn face_mut(&mut self, idx: FaceIndex) -> &mut Face {
        &mut self.faces[idx.0]
    }
    
    // Example traversal method with type safety
    pub fn vertex_outgoing_half_edges(&self, vertex_idx: VertexIndex) -> Vec<HalfEdgeIndex> {
        let mut outgoing = Vec::new();
        
        if let Some(start_he) = self.vertex(vertex_idx).seed_half_edge {
            let mut current_he = start_he;
            
            loop {
                outgoing.push(current_he);
                
                let he = self.half_edge(current_he);
                if let Some(twin_he) = he.twin_index {
                    current_he = self.half_edge(twin_he).next_edge;
                    
                    if current_he == start_he {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        
        outgoing
    }
}