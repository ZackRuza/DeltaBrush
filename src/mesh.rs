use wasm_bindgen::prelude::*;
use crate::algebra::Vec3;

#[wasm_bindgen]
pub struct Mesh {
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
}

#[wasm_bindgen]
impl Mesh {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Mesh {
        Mesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, x: f32, y: f32, z: f32) {
        self.vertices.push(Vec3::new(x, y, z));
    }

    pub fn add_triangle(&mut self, i0: u32, i1: u32, i2: u32) {
        self.indices.push(i0);
        self.indices.push(i1);
        self.indices.push(i2);
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn get_vertices_flat(&self) -> Vec<f32> {
        self.vertices
            .iter()
            .flat_map(|v| vec![v.x, v.y, v.z])
            .collect()
    }

    pub fn get_indices(&self) -> Vec<u32> {
        self.indices.clone()
    }

    /// Create a cube mesh
    pub fn create_cube(size: f32) -> Mesh {
        let mut mesh = Mesh::new();
        let half = size / 2.0;

        // Define 8 vertices of a cube
        mesh.add_vertex(-half, -half, -half); // 0
        mesh.add_vertex(half, -half, -half);  // 1
        mesh.add_vertex(half, half, -half);   // 2
        mesh.add_vertex(-half, half, -half);  // 3
        mesh.add_vertex(-half, -half, half);  // 4
        mesh.add_vertex(half, -half, half);   // 5
        mesh.add_vertex(half, half, half);    // 6
        mesh.add_vertex(-half, half, half);   // 7

        // Define 12 triangles (2 per face, 6 faces)
        // Front face
        mesh.add_triangle(0, 2, 1);
        mesh.add_triangle(0, 3, 2);
        // Back face
        mesh.add_triangle(5, 7, 4);
        mesh.add_triangle(5, 6, 7);
        // Top face
        mesh.add_triangle(3, 6, 2);
        mesh.add_triangle(3, 7, 6);
        // Bottom face
        mesh.add_triangle(4, 1, 5);
        mesh.add_triangle(4, 0, 1);
        // Right face
        mesh.add_triangle(1, 6, 5);
        mesh.add_triangle(1, 2, 6);
        // Left face
        mesh.add_triangle(4, 3, 0);
        mesh.add_triangle(4, 7, 3);

        mesh

    }
}
