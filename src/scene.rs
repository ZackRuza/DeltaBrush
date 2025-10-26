use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use crate::geometry::Mesh;
use crate::{console_log, Vec3};

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneObject {
    pub id: usize,
    pub mesh_data: MeshData,
    pub transform: Transform,
    pub material: Material,
}

impl SceneObject {
    //TODO: Return Option for case where vec doesn't hit
    fn raycast_first_hit(&self, origin: Vec3, direction: Vec3) -> Option<HitResponse> {
        // Go through each triangle and perform ray intersection
        // TODO: implement this correctly.
        todo!()
    }
}

pub struct HitResponse {
    pub hit_position: Vec3,
    pub hit_distance: f32,
    // TODO: HitResponse can hold more information, such as element reference or element ID
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

// Public functions are exposed to the front end (JS) and handle conversions,
// private functions handle actual scene management
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


    // Functions for interacting witht the scene
    pub fn raycast_click(&self, origin: Vec<f32>, direction: Vec<f32>) -> JsValue {
        if let (Ok(origin_vec), Ok(direction_vec)) = (Vec3::new_from_vec(origin), Vec3::new_from_vec(direction)) {
            // NEXT: process return from raycast
            if let Some(response) = self.raycast_first_hit(origin_vec, direction_vec) {
                // Return the relevant hit data for JS
                return serde_wasm_bindgen::to_value(&response.hit_position).unwrap();
            } else {
                // TODO: Proper handling if no response
                // No response. Object was not hit.
            }
        } else {
            // TODO: Property handling if vectors aren't 3D. Throw error.
        }
        
        // TODO: return proper JS value
        JsValue::NULL
    }

    // Returns the position of the first hit
    fn raycast_first_hit(&self, origin: Vec3, direction: Vec3) -> Option<HitResponse> {
        let mut optional_hit: Option<HitResponse> = None;
        for scene_object in &self.objects {
            if let Some(response) = scene_object.raycast_first_hit(origin, direction) {
                let should_replace = match &optional_hit {
                    None => true,
                    Some(existing) => response.hit_distance < existing.hit_distance,
                };
                if should_replace {
                    optional_hit = Some(response);
                }
            }
        }
        optional_hit
    }
}
