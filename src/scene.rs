use wasm_bindgen::prelude::*;
use crate::{Mesh, Transform, Material};
use crate::scene_object::SceneObject;
use crate::{console_log, Vec3};
use crate::geometry::{Direction3, HitResponse, Point3, Ray3};

// World hit reponse holds the hit response in world coordinates, as well as the
// distance
#[derive(Clone)]
pub struct WorldHitResponse {
    pub hit_response: HitResponse,
    pub distance: f32,
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
        let id = self.next_id;
        self.next_id += 1;

        let object = SceneObject {
            id,
            mesh: Mesh::create_cube(size),
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


    // Functions for interacting with the scene
    pub fn raycast_click(&self, origin: Vec<f32>, direction: Vec<f32>) -> JsValue {
        if let (Ok(origin_vec3), Ok(direction_vec3)) = (Vec3::new_from_vec(origin), Vec3::new_from_vec(direction)) {
            let ray = Ray3 {
                origin: Point3 { position: origin_vec3 },
                direction: Direction3 { direction: direction_vec3 }
            };
            
            if let Some(world_hit) = self.raycast_closest_hit(ray) {
                // Return the relevant hit position for JS
                return serde_wasm_bindgen::to_value(&world_hit.hit_response.hit_position.position).unwrap();
            } else {
                // No response. Object was not hit.
                JsValue::NULL
            }
        } else {
            // TODO: Property handling if vectors aren't 3D. Throw error.
            JsValue::NULL
        }
    }

    // Returns the position of the closest hit to the ray origin
    fn raycast_closest_hit(&self, ray: Ray3) -> Option<WorldHitResponse> {
        let mut optional_hit: Option<WorldHitResponse> = None;

        for scene_object in &self.objects {
            if let Some(world_hit) = scene_object.raycast_closest_hit(ray) {
                let should_replace = match &optional_hit {
                    None => true,
                    Some(existing) => world_hit.distance < existing.distance,
                };
                if should_replace {
                    optional_hit = Some(world_hit);
                }
            }
        }
        optional_hit
    }
}
