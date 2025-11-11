use serde::{Deserialize, Serialize};

/// Flat, render/serialize-friendly mesh representation used throughout runtime.
#[derive(Serialize, Deserialize, Clone)]
pub struct Mesh {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub normals: Option<Vec<f32>>, // optional, computed or supplied by caller
}

impl Mesh {
    pub fn new() -> Self {
        Mesh {
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: None,
        }
    }

    #[inline]
    pub fn add_vertex(&mut self, x: f32, y: f32, z: f32) {
        self.vertices.extend_from_slice(&[x, y, z]);
    }

    #[inline]
    pub fn add_triangle(&mut self, i0: u32, i1: u32, i2: u32) {
        self.indices.extend_from_slice(&[i0, i1, i2]);
    }

    #[inline]
    pub fn set_vertex(&mut self, i: usize, x: f32, y: f32, z: f32) {
        let base = i * 3;
        self.vertices[base] = x;
        self.vertices[base + 1] = y;
        self.vertices[base + 2] = z;
    }

    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / 3
    }

    #[inline]
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
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
