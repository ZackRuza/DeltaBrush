use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// A 3D vector
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[wasm_bindgen]
impl Vec3 {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Vec3 {
        let len = self.length();
        if len > 0.0 {
            Vec3 {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            *self
        }
    }

    pub fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

/// A 3D mesh
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
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        // Back face
        mesh.add_triangle(5, 4, 7);
        mesh.add_triangle(5, 7, 6);
        // Top face
        mesh.add_triangle(3, 2, 6);
        mesh.add_triangle(3, 6, 7);
        // Bottom face
        mesh.add_triangle(4, 5, 1);
        mesh.add_triangle(4, 1, 0);
        // Right face
        mesh.add_triangle(1, 5, 6);
        mesh.add_triangle(1, 6, 2);
        // Left face
        mesh.add_triangle(4, 0, 3);
        mesh.add_triangle(4, 3, 7);

        mesh
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("DeltaBrush Rust core initialized!");
}
