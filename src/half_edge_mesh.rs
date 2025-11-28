use std::collections::HashMap;
use crate::{Mesh, ToMesh, geometry::Point3};

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
    /// Create a cube half-edge mesh directly with quad faces
    /// 8 vertices, 24 half-edges (4 per face), 6 quad faces
    pub fn create_cube(size: f32) -> Self {
        let half = size / 2.0;
        
        // 8 vertices
        let vertices = vec![
            Vertex { position: Point3::new(-half, -half, -half), seed_half_edge: Some(HalfEdgeIndex(0)) },  // 0
            Vertex { position: Point3::new( half, -half, -half), seed_half_edge: Some(HalfEdgeIndex(4)) },  // 1
            Vertex { position: Point3::new( half,  half, -half), seed_half_edge: Some(HalfEdgeIndex(8)) },  // 2
            Vertex { position: Point3::new(-half,  half, -half), seed_half_edge: Some(HalfEdgeIndex(12)) }, // 3
            Vertex { position: Point3::new(-half, -half,  half), seed_half_edge: Some(HalfEdgeIndex(16)) }, // 4
            Vertex { position: Point3::new( half, -half,  half), seed_half_edge: Some(HalfEdgeIndex(20)) }, // 5
            Vertex { position: Point3::new( half,  half,  half), seed_half_edge: Some(HalfEdgeIndex(5)) },  // 6
            Vertex { position: Point3::new(-half,  half,  half), seed_half_edge: Some(HalfEdgeIndex(9)) },  // 7
        ];
        
        // 6 quad faces (24 half-edges total, 4 per face)
        let faces = vec![
            Face { seed_half_edge: HalfEdgeIndex(0) },  // Front face (-Z)
            Face { seed_half_edge: HalfEdgeIndex(4) },  // Right face (+X)
            Face { seed_half_edge: HalfEdgeIndex(8) },  // Back face (+Z)
            Face { seed_half_edge: HalfEdgeIndex(12) }, // Left face (-X)
            Face { seed_half_edge: HalfEdgeIndex(16) }, // Bottom face (-Y)
            Face { seed_half_edge: HalfEdgeIndex(20) }, // Top face (+Y)
        ];
        
        let half_edges = vec![
            // Face 0: Front face (-Z): 0 -> 1 -> 2 -> 3
            HalfEdge { target_vertex_index: VertexIndex(1), twin_index: Some(HalfEdgeIndex(7)),  next_edge: HalfEdgeIndex(1),  prev_edge: HalfEdgeIndex(3),  face_index: Some(FaceIndex(0)) }, // 0
            HalfEdge { target_vertex_index: VertexIndex(2), twin_index: Some(HalfEdgeIndex(11)), next_edge: HalfEdgeIndex(2),  prev_edge: HalfEdgeIndex(0),  face_index: Some(FaceIndex(0)) }, // 1
            HalfEdge { target_vertex_index: VertexIndex(3), twin_index: Some(HalfEdgeIndex(15)), next_edge: HalfEdgeIndex(3),  prev_edge: HalfEdgeIndex(1),  face_index: Some(FaceIndex(0)) }, // 2
            HalfEdge { target_vertex_index: VertexIndex(0), twin_index: Some(HalfEdgeIndex(19)), next_edge: HalfEdgeIndex(0),  prev_edge: HalfEdgeIndex(2),  face_index: Some(FaceIndex(0)) }, // 3
            
            // Face 1: Right face (+X): 1 -> 5 -> 6 -> 2
            HalfEdge { target_vertex_index: VertexIndex(5), twin_index: Some(HalfEdgeIndex(17)), next_edge: HalfEdgeIndex(5),  prev_edge: HalfEdgeIndex(7),  face_index: Some(FaceIndex(1)) }, // 4
            HalfEdge { target_vertex_index: VertexIndex(6), twin_index: Some(HalfEdgeIndex(21)), next_edge: HalfEdgeIndex(6),  prev_edge: HalfEdgeIndex(4),  face_index: Some(FaceIndex(1)) }, // 5
            HalfEdge { target_vertex_index: VertexIndex(2), twin_index: Some(HalfEdgeIndex(9)),  next_edge: HalfEdgeIndex(7),  prev_edge: HalfEdgeIndex(5),  face_index: Some(FaceIndex(1)) }, // 6
            HalfEdge { target_vertex_index: VertexIndex(1), twin_index: Some(HalfEdgeIndex(0)),  next_edge: HalfEdgeIndex(4),  prev_edge: HalfEdgeIndex(6),  face_index: Some(FaceIndex(1)) }, // 7
            
            // Face 2: Back face (+Z): 5 -> 4 -> 7 -> 6
            HalfEdge { target_vertex_index: VertexIndex(4), twin_index: Some(HalfEdgeIndex(18)), next_edge: HalfEdgeIndex(9),  prev_edge: HalfEdgeIndex(11), face_index: Some(FaceIndex(2)) }, // 8
            HalfEdge { target_vertex_index: VertexIndex(7), twin_index: Some(HalfEdgeIndex(22)), next_edge: HalfEdgeIndex(10), prev_edge: HalfEdgeIndex(8),  face_index: Some(FaceIndex(2)) }, // 9
            HalfEdge { target_vertex_index: VertexIndex(6), twin_index: Some(HalfEdgeIndex(6)),  next_edge: HalfEdgeIndex(11), prev_edge: HalfEdgeIndex(9),  face_index: Some(FaceIndex(2)) }, // 10
            HalfEdge { target_vertex_index: VertexIndex(5), twin_index: Some(HalfEdgeIndex(1)),  next_edge: HalfEdgeIndex(8),  prev_edge: HalfEdgeIndex(10), face_index: Some(FaceIndex(2)) }, // 11
            
            // Face 3: Left face (-X): 4 -> 0 -> 3 -> 7
            HalfEdge { target_vertex_index: VertexIndex(0), twin_index: Some(HalfEdgeIndex(16)), next_edge: HalfEdgeIndex(13), prev_edge: HalfEdgeIndex(15), face_index: Some(FaceIndex(3)) }, // 12
            HalfEdge { target_vertex_index: VertexIndex(3), twin_index: Some(HalfEdgeIndex(23)), next_edge: HalfEdgeIndex(14), prev_edge: HalfEdgeIndex(12), face_index: Some(FaceIndex(3)) }, // 13
            HalfEdge { target_vertex_index: VertexIndex(7), twin_index: Some(HalfEdgeIndex(10)), next_edge: HalfEdgeIndex(15), prev_edge: HalfEdgeIndex(13), face_index: Some(FaceIndex(3)) }, // 14
            HalfEdge { target_vertex_index: VertexIndex(4), twin_index: Some(HalfEdgeIndex(2)),  next_edge: HalfEdgeIndex(12), prev_edge: HalfEdgeIndex(14), face_index: Some(FaceIndex(3)) }, // 15
            
            // Face 4: Bottom face (-Y): 0 -> 4 -> 5 -> 1
            HalfEdge { target_vertex_index: VertexIndex(4), twin_index: Some(HalfEdgeIndex(12)), next_edge: HalfEdgeIndex(17), prev_edge: HalfEdgeIndex(19), face_index: Some(FaceIndex(4)) }, // 16
            HalfEdge { target_vertex_index: VertexIndex(5), twin_index: Some(HalfEdgeIndex(4)),  next_edge: HalfEdgeIndex(18), prev_edge: HalfEdgeIndex(16), face_index: Some(FaceIndex(4)) }, // 17
            HalfEdge { target_vertex_index: VertexIndex(1), twin_index: Some(HalfEdgeIndex(8)),  next_edge: HalfEdgeIndex(19), prev_edge: HalfEdgeIndex(17), face_index: Some(FaceIndex(4)) }, // 18
            HalfEdge { target_vertex_index: VertexIndex(0), twin_index: Some(HalfEdgeIndex(3)),  next_edge: HalfEdgeIndex(16), prev_edge: HalfEdgeIndex(18), face_index: Some(FaceIndex(4)) }, // 19
            
            // Face 5: Top face (+Y): 3 -> 2 -> 6 -> 7
            HalfEdge { target_vertex_index: VertexIndex(2), twin_index: Some(HalfEdgeIndex(14)), next_edge: HalfEdgeIndex(21), prev_edge: HalfEdgeIndex(23), face_index: Some(FaceIndex(5)) }, // 20
            HalfEdge { target_vertex_index: VertexIndex(6), twin_index: Some(HalfEdgeIndex(5)),  next_edge: HalfEdgeIndex(22), prev_edge: HalfEdgeIndex(20), face_index: Some(FaceIndex(5)) }, // 21
            HalfEdge { target_vertex_index: VertexIndex(7), twin_index: Some(HalfEdgeIndex(9)),  next_edge: HalfEdgeIndex(23), prev_edge: HalfEdgeIndex(21), face_index: Some(FaceIndex(5)) }, // 22
            HalfEdge { target_vertex_index: VertexIndex(3), twin_index: Some(HalfEdgeIndex(13)), next_edge: HalfEdgeIndex(20), prev_edge: HalfEdgeIndex(22), face_index: Some(FaceIndex(5)) }, // 23
        ];
        
        HalfEdgeMesh {
            vertices,
            half_edges,
            faces,
        }
    }

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



impl ToMesh for HalfEdgeMesh {
    fn to_mesh(&self) -> Mesh {

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

                // TODO: We know that a face will have at least 3 vertices. But,
                //       maybe we can imprive efficiency if we know capacity beforehand
                let mut indices = Vec::with_capacity(3);

                // Have triangular faces made from a single source vertex on the face
                
                // The first half-edge simply points to our source vertexs
                let pointing_half_edge = self.half_edge(face.seed_half_edge);
                // Source vertex
                let source_vertex_index = pointing_half_edge.target_vertex_index;

                // The next half edge points to the first half-edge in our sequence of half-edges
                // which represent the exterior edges of the sequence of triangles that make
                // up the face. We skip this half-edge.
                let mut current_half_edge_index = pointing_half_edge.next_edge;
                let mut prev_vertex_index = self.half_edge(current_half_edge_index).target_vertex_index;

                loop {
                    // Move to the next half edge
                    current_half_edge_index = self.half_edge(current_half_edge_index).next_edge;
                    
                    // Exit if we've looped back to the beginning
                    if current_half_edge_index.0 == face.seed_half_edge.0 {
                        break;
                    }

                    // Find next vertex (it won't be source!)
                    let next_vertex_index = self.half_edge(current_half_edge_index).target_vertex_index;

                    // Create a triangle with (source, next_vertex, prev_vertex)
                    indices.push(source_vertex_index.0 as u32);
                    indices.push(next_vertex_index.0 as u32);
                    indices.push(prev_vertex_index.0 as u32);
                    

                    prev_vertex_index = next_vertex_index;
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
}