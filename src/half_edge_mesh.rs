use std::collections::HashMap;

use crate::{Mesh, geometry::Point3};

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
    pub face_index: Option<FaceIndex> // Face index (none for boundary case)
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
    // Creating half edge data structure from mesh

    pub fn from_mesh(mesh: &Mesh) -> Self {

        let mut vertices = Vec::with_capacity(mesh.vertex_count());
        let mut half_edges = Vec::with_capacity(mesh.face_indices.len());
        let mut faces = Vec::with_capacity(mesh.face_count());

        // Creating vertices (seed to be set later)
        for coord in mesh.vertex_coords.chunks_exact(3) {
            vertices.push(
                Vertex {
                    position: Point3::new(coord[0], coord[1], coord[2]),
                    // Set seed half-edge later
                    seed_half_edge: None,
                }
            );
        }

        // Creating half-edges and faces
        // Half-edge's twin determined at next step
        // Vertex seeds set here
        for (idx, triangle) in mesh.face_indices.chunks_exact(3).enumerate() {
            let &[vi0, vi1, vi2] = triangle else {unreachable!()};
            let v0 = VertexIndex(vi0 as usize);
            let v1 = VertexIndex(vi1 as usize);
            let v2 = VertexIndex(vi2 as usize);
            
            let face_index = FaceIndex(idx);
            let triple_idx = idx * 3;
            
            // half edge v0 -> v1. At idx * 3.
            let he01 = HalfEdge {
                target_vertex_index: v1,
                twin_index: None,
                next_edge: HalfEdgeIndex(triple_idx + 1),
                prev_edge: HalfEdgeIndex(triple_idx + 2),
                face_index: Some(face_index),
            };
            half_edges.push(he01);

            // Set vertex seed. Vertex 0.
            if vertices[vi0 as usize].seed_half_edge.is_none() {
                vertices[vi0 as usize].seed_half_edge = Some(HalfEdgeIndex(triple_idx));
            }

            // half edge v1 -> v2. At idx * 3 + 1.
            let he12 = HalfEdge {
                target_vertex_index: v2,
                twin_index: None,
                next_edge: HalfEdgeIndex(triple_idx + 2),
                prev_edge: HalfEdgeIndex(triple_idx),
                face_index: Some(face_index),
            };
            half_edges.push(he12);

            // Set vertex seed. Vertex 1.
            if vertices[vi1 as usize].seed_half_edge.is_none() {
                vertices[vi1 as usize].seed_half_edge = Some(HalfEdgeIndex(triple_idx + 1));
            }

            // half edge v2 -> v0. At idx * 3 + 2.
            let he20 = HalfEdge {
                target_vertex_index: v0,
                twin_index: None,
                next_edge: HalfEdgeIndex(triple_idx),
                prev_edge: HalfEdgeIndex(triple_idx + 1),
                face_index: Some(face_index),
            };
            half_edges.push(he20);

            // Set vertex seed. Vertex 2.
            if vertices[vi2 as usize].seed_half_edge.is_none() {
                vertices[vi2 as usize].seed_half_edge = Some(HalfEdgeIndex(triple_idx + 2));
            }

            // Create face
            faces.push(
                Face {
                    seed_half_edge: HalfEdgeIndex(triple_idx),
                }
            );

        }
        

        // Quick exploring and connecting half-edges

        let mut edge_map: HashMap<(VertexIndex, VertexIndex), HalfEdgeIndex> = HashMap::new();
        
        // Create half edge map
        for (half_edge_idx, half_edge) in half_edges.iter().enumerate() {
            let source = half_edges[half_edge.prev_edge.0].target_vertex_index;
            let target = half_edge.target_vertex_index;

            edge_map.insert((source, target), HalfEdgeIndex(half_edge_idx));
        }

        // Collect the twins
        let twins: Vec<Option<HalfEdgeIndex>> = half_edges.iter().map(
            |half_edge| {
                let source = half_edges[half_edge.prev_edge.0].target_vertex_index;
                let target = half_edge.target_vertex_index;

                edge_map.get(&(target, source)).copied()
            }
        ).collect();

        for (half_edge, twin) in half_edges.iter_mut().zip(twins.into_iter()) {
            half_edge.twin_index = twin;
        }

        HalfEdgeMesh {
            vertices,
            half_edges,
            faces,
        }
    }

    pub fn to_mesh(&self) -> Mesh {

        let vertex_coords = 
        self.vertices.iter().flat_map(
            |vertex| [
                vertex.position.vec3.x,
                vertex.position.vec3.y,
                vertex.position.vec3.z
            ]
        ).collect();

        let face_indices = self.faces.iter().flat_map(
            |face| {
                let start_half_edge_index = face.seed_half_edge;
                let mut current_half_edge_index = start_half_edge_index;
                let mut indices = Vec::with_capacity(3);

                loop {
                    let current_half_edge = self.half_edge(current_half_edge_index);
                    indices.push(current_half_edge.target_vertex_index.0 as u32);
                    current_half_edge_index = current_half_edge.next_edge;
                    
                    if current_half_edge_index == start_half_edge_index {
                        break;
                    }
                }

                indices
            }
        ).collect();

        // TODO: potentially fill in normals from the half-edge mesh
        let normals = None;
        
        Mesh {
            vertex_coords: vertex_coords,
            face_indices: face_indices,
            normals: normals,
        }
    }


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