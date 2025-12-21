use serde::{Deserialize, Serialize};

/// Flat, render/serialize-friendly mesh representation used throughout runtime.
#[derive(Serialize, Deserialize, Clone)]
pub struct Mesh {
    pub vertex_coords: Vec<f32>,
    pub face_indices: Vec<u32>,
    pub normals: Option<Vec<f32>>, // optional, computed or supplied by caller
}

impl Mesh {
    pub fn new() -> Self {
        Mesh {
            vertex_coords: Vec::new(),
            face_indices: Vec::new(),
            normals: None,
        }
    }

    #[inline]
    pub fn add_vertex(&mut self, x: f32, y: f32, z: f32) {
        self.vertex_coords.extend_from_slice(&[x, y, z]);
    }

    #[inline]
    pub fn add_triangle(&mut self, i0: u32, i1: u32, i2: u32) {
        self.face_indices.extend_from_slice(&[i0, i1, i2]);
    }

    #[inline]
    pub fn set_vertex(&mut self, i: usize, x: f32, y: f32, z: f32) {
        let base = i * 3;
        self.vertex_coords[base] = x;
        self.vertex_coords[base + 1] = y;
        self.vertex_coords[base + 2] = z;
    }

    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.vertex_coords.len() / 3
    }

    #[inline]
    pub fn face_count(&self) -> usize {
        self.face_indices.len() / 3
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

    /// Create a sphere mesh using UV sphere generation
    pub fn create_sphere(radius: f32, segments: u32, rings: u32) -> Mesh {
        let mut mesh = Mesh::new();
        
        // Generate vertices
        for ring in 0..=rings {
            let phi = std::f32::consts::PI * ring as f32 / rings as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();
            
            for segment in 0..=segments {
                let theta = 2.0 * std::f32::consts::PI * segment as f32 / segments as f32;
                let sin_theta = theta.sin();
                let cos_theta = theta.cos();
                
                let x = radius * sin_phi * cos_theta;
                let y = radius * cos_phi;
                let z = radius * sin_phi * sin_theta;
                
                mesh.add_vertex(x, y, z);
            }
        }
        
        // Generate faces
        for ring in 0..rings {
            for segment in 0..segments {
                let current = ring * (segments + 1) + segment;
                let next = current + segments + 1;
                
                // First triangle
                mesh.add_triangle(current, next, current + 1);
                // Second triangle  
                mesh.add_triangle(current + 1, next, next + 1);
            }
        }
        
        mesh
    }

}
