use std::{collections::VecDeque, future::Future};
use crate::{HalfEdgeMesh, VertexIndex};

// Trait for asynchronous visits on type T
pub trait AsyncVisitor<T> {
    fn visit<'a>(&'a mut self, mesh: &'a HalfEdgeMesh, element: T) -> impl Future<Output = ()> + 'a;
}


// For each type of visit we want to do, we create a struct.
// Each struct corresponds to a type of action we want to perform on a element
struct PrintVisitor;

impl AsyncVisitor<VertexIndex> for PrintVisitor {
    fn visit<'a>(&'a mut self, mesh: &'a HalfEdgeMesh, vertex_idx: VertexIndex) -> impl Future<Output = ()> + 'a {
        async move {
            let vertex = mesh.vertex(vertex_idx);
            println!("Visited vertex {} at position ({}, {}, {})", 
                     vertex_idx.0,
                     vertex.position.vec3.x,
                     vertex.position.vec3.y,
                     vertex.position.vec3.z);
        }
    }
}

// BFS traversal starting from a vertex, using half-edge mesh structure
pub async fn half_edge_mesh_bfs<V>(
    mesh: &HalfEdgeMesh,
    start: VertexIndex,
    visitor: &mut V
)
where
    V: AsyncVisitor<VertexIndex>,
{
    use std::collections::HashSet;
    
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back(start);
    visited.insert(start);

    while let Some(vertex_idx) = queue.pop_front() {
        // Async call to visitor with mesh and vertex index
        visitor.visit(mesh, vertex_idx).await;

        // Find neighbors by walking around the vertex via half-edges
        if let Some(seed_he) = mesh.vertex(vertex_idx).seed_half_edge {
            let mut current_he = seed_he;
            
            loop {
                let he = mesh.half_edge(current_he);
                let neighbor = he.target_vertex_index;
                
                // Add neighbor to queue if not visited
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
                
                // Move to next half-edge around this vertex
                if let Some(twin) = he.twin_index {
                    current_he = mesh.half_edge(twin).next_edge;
                    
                    // Stop when we've completed the loop
                    if current_he == seed_he {
                        break;
                    }
                } else {
                    // Hit a boundary edge
                    break;
                }
            }
        }
    }
}