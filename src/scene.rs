use wasm_bindgen::prelude::*;
use crate::{Mesh, Transform, Material, MeshEditor, ModelVariant};
use crate::scene_object::{SceneObject, WorldHitResponse};
use crate::{console_log, Vec3};
use crate::geometry::{Direction3, Point3, Ray3};
use serde::{Serialize, Deserialize};

// =================== CORE SCENE IMPLEMENTATION ===================

/// Core scene implementation - pure Rust, no JS dependencies
pub struct Scene {
    objects: Vec<SceneObject>,
    next_id: usize,
    dirty: bool,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            objects: Vec::new(),
            next_id: 0,
            dirty: false,
        }
    }

    pub fn add_cube(&mut self, size: f32, position: [f32; 3], material: Material) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let mesh = Mesh::create_cube(size);
        let object = SceneObject {
            id,
            model: ModelVariant::HalfEdge(MeshEditor::new(mesh)),
            transform: Transform {
                position,
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
            },
            material,
        };

        self.objects.push(object);
        self.dirty = true;
        id
    }

    pub fn add_sphere(&mut self, radius: f32, position: [f32; 3], material: Material) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let mesh = Mesh::create_sphere(radius, 16, 16);
        let object = SceneObject {
            id,
            model: ModelVariant::HalfEdge(MeshEditor::new(mesh)),
            transform: Transform {
                position,
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
            },
            material,
        };

        self.objects.push(object);
        self.dirty = true;
        id
    }

    pub fn remove_object(&mut self, id: usize) -> bool {
        if let Some(pos) = self.objects.iter().position(|obj| obj.id == id) {
            self.objects.remove(pos);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn update_transform(&mut self, id: usize, transform: Transform) -> bool {
        if let Some(obj) = self.objects.iter_mut().find(|obj| obj.id == id) {
            obj.transform = transform;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn raycast_closest_hit(&self, ray: Ray3) -> Option<WorldHitResponse> {
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

    // Getters
    pub fn is_dirty(&self) -> bool { self.dirty }
    pub fn clear_dirty(&mut self) { self.dirty = false; }
    pub fn object_count(&self) -> usize { self.objects.len() }
    pub fn objects(&self) -> &[SceneObject] { &self.objects }
    
    pub fn clear(&mut self) {
        self.objects.clear();
        self.dirty = true;
    }
}

// =================== JS INTERFACE LAYER ===================

/// JavaScript interface - handles conversions and WASM bindings
#[wasm_bindgen]
pub struct SceneAPI {
    core: Scene,
}

// Structs for passing information to the front end
#[derive(Serialize, Deserialize)]
struct HitPosition {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Serialize, Deserialize)]
struct HitData {
    position: HitPosition,
    object_id: usize,
}


// Public functions are exposed to the front end (JS) and handle conversions,
// private functions handle actual scene management
#[wasm_bindgen]
impl SceneAPI {
    #[wasm_bindgen(constructor)]
    pub fn new() -> SceneAPI {
        console_log!("Creating new Rust Scene");
        SceneAPI {
            core: Scene::new(),
        }
    }

    /// Add a cube to the scene
    pub fn add_cube(&mut self, size: f32, position: Vec<f32>) -> usize {
        let pos_array = [position[0], position[1], position[2]];
        let material = Material {
            color: [
                js_sys::Math::random() as f32,
                js_sys::Math::random() as f32,
                js_sys::Math::random() as f32,
            ],
            metalness: 0.3,
            roughness: 0.4,
        };

        let id = self.core.add_cube(size, pos_array, material);
        console_log!("Adding cube with id {} at position [{}, {}, {}]", id, position[0], position[1], position[2]);
        id
    }

    /// Add a sphere to the scene
    pub fn add_sphere(&mut self, radius: f32, position: Vec<f32>) -> usize {
        let pos_array = [position[0], position[1], position[2]];
        let material = Material {
            color: [
                js_sys::Math::random() as f32,
                js_sys::Math::random() as f32,
                js_sys::Math::random() as f32,
            ],
            metalness: 0.3,
            roughness: 0.4,
        };

        let id = self.core.add_sphere(radius, pos_array, material);
        console_log!("Adding sphere with id {} at position [{}, {}, {}]", id, position[0], position[1], position[2]);
        id
    }

    pub fn remove_object(&mut self, id: usize) -> bool {
        let success = self.core.remove_object(id);
        if success {
            console_log!("Removed object with id {}", id);
        } else {
            console_log!("Failed to remove object with id {}: not found", id);
        }
        success
    }

    pub fn update_transform(&mut self, id: usize, position: Vec<f32>, rotation: Vec<f32>, scale: Vec<f32>) {
        let transform = Transform {
            position: [position[0], position[1], position[2]],
            rotation: [rotation[0], rotation[1], rotation[2], rotation[3]],
            scale: [scale[0], scale[1], scale[2]],
        };

        if self.core.update_transform(id, transform) {
            console_log!("Updated transform for object {}", id);
        }
    }

    pub fn is_dirty(&self) -> bool { self.core.is_dirty() }
    pub fn clear_dirty(&mut self) { self.core.clear_dirty(); }
    pub fn object_count(&self) -> usize { self.core.object_count() }
    
    pub fn clear(&mut self) {
        console_log!("Clearing scene");
        self.core.clear();
    }

    pub fn get_scene_data(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self.core.objects()).unwrap()
    }

    pub fn raycast_closest_hit(&self, origin: Vec<f32>, direction: Vec<f32>) -> JsValue {
        if let (Ok(origin_vec3), Ok(direction_vec3)) = (Vec3::new_from_vec(origin), Vec3::new_from_vec(direction)) {
            let ray = Ray3::new(
                Point3 { vec3: origin_vec3 },
                Direction3 { vec3: direction_vec3 }
            );
            
            if let Some(world_hit) = self.core.raycast_closest_hit(ray) {
                // Return hit position and object ID for JS
                let hit_data = HitData {
                    position: HitPosition {
                        x: world_hit.hit_response.hit_position.vec3.x,
                        y: world_hit.hit_response.hit_position.vec3.y,
                        z: world_hit.hit_response.hit_position.vec3.z,
                    },
                    object_id: world_hit.object_id,
                };
                return serde_wasm_bindgen::to_value(&hit_data).unwrap();
            } else {
                // No response. Object was not hit.
                JsValue::NULL
            }
        } else {
            // TODO: Property handling if vectors aren't 3D. Throw error.
            JsValue::NULL
        }
    }
}