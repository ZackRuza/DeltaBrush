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

/// Scene object data structures
#[derive(Serialize, Deserialize, Clone)]
pub struct SceneObject {
    pub id: usize,
    pub mesh_data: MeshData,
    pub transform: Transform,
    pub material: Material,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 4], // quaternion (x, y, z, w)
    pub scale: [f32; 3],
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Material {
    pub color: [f32; 3],
    pub metalness: f32,
    pub roughness: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MeshData {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub normals: Option<Vec<f32>>,
}

/// Scene manager that maintains all 3D objects
#[wasm_bindgen]
pub struct Scene {
    objects: Vec<SceneObject>,
    next_id: usize,
    dirty: bool,
}

#[wasm_bindgen]
impl Scene {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Scene {
        console_log!("Creating new Rust Scene");
        Scene {
            objects: Vec::new(),
            next_id: 0,
            dirty: false,
        }
    }

    /// Add a cube to the scene
    pub fn add_cube(&mut self, size: f32, position: Vec<f32>) -> usize {
        let mesh = Mesh::create_cube(size);
        let id = self.next_id;
        self.next_id += 1;

        let object = SceneObject {
            id,
            mesh_data: MeshData {
                vertices: mesh.get_vertices_flat(),
                indices: mesh.get_indices(),
                normals: None,
            },
            transform: Transform {
                position: [position[0], position[1], position[2]],
                rotation: [0.0, 0.0, 0.0, 1.0], // identity quaternion
                scale: [1.0, 1.0, 1.0],
            },
            material: Material {
                color: [
                    js_sys::Math::random() as f32,
                    js_sys::Math::random() as f32,
                    js_sys::Math::random() as f32,
                ],
                metalness: 0.3,
                roughness: 0.4,
            },
        };

        console_log!("Adding cube with id {} at position [{}, {}, {}]", id, position[0], position[1], position[2]);
        self.objects.push(object);
        self.dirty = true;
        id
    }

    /// Remove an object from the scene
    pub fn remove_object(&mut self, id: usize) -> bool {
        if let Some(pos) = self.objects.iter().position(|obj| obj.id == id) {
            self.objects.remove(pos);
            self.dirty = true;
            console_log!("Removed object with id {}", id);
            true
        } else {
            console_log!("Failed to remove object with id {}: not found", id);
            false
        }
    }

    /// Update an object's transform
    pub fn update_transform(&mut self, id: usize, position: Vec<f32>, rotation: Vec<f32>, scale: Vec<f32>) {
        if let Some(obj) = self.objects.iter_mut().find(|obj| obj.id == id) {
            obj.transform.position = [position[0], position[1], position[2]];
            obj.transform.rotation = [rotation[0], rotation[1], rotation[2], rotation[3]];
            obj.transform.scale = [scale[0], scale[1], scale[2]];
            self.dirty = true;
            console_log!("Updated transform for object {}", id);
        }
    }

    /// Get all scene data as a JavaScript value
    pub fn get_scene_data(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.objects).unwrap()
    }

    /// Check if the scene has been modified
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear the dirty flag
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Clear all objects from the scene
    pub fn clear(&mut self) {
        console_log!("Clearing scene");
        self.objects.clear();
        self.dirty = true;
    }

    /// Get the number of objects in the scene
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("DeltaBrush Rust core initialized!");
}
