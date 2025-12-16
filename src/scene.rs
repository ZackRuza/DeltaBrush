use wasm_bindgen::prelude::*;
use crate::model::ModelVariant;
use crate::{HalfEdgeMesh, ModelWrapper, Transform};
use crate::scene_graph::{SceneGraphNode, SceneGraphChild, EdgeId};
use crate::RenderInstance;
use crate::render_instance::MeshId;
use crate::{console_log, Vec3};
use crate::geometry::{Direction3, Point3, Ray3, WorldHitResponse};
use serde::{Serialize, Deserialize};

// =================== CORE SCENE IMPLEMENTATION ===================

/// Core scene implementation - pure Rust, no JS dependencies
pub struct Scene {
    root: SceneGraphNode,
    dirty: bool,
    meshes: Vec<ModelVariant>,
    cached_render_instances: Vec<RenderInstance>,
    hierarchy_dirty: bool,
    selected_path: Option<Vec<EdgeId>>,  // Path of edge IDs
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            root: SceneGraphNode::new(),
            dirty: false,
            meshes: Vec::new(),
            cached_render_instances: Vec::new(),
            hierarchy_dirty: true,
            selected_path: None,  // Path of edge IDs
        }
    }

    /// Rebuild the flat cache when hierarchy changes
    fn rebuild_cache(&mut self) {
        if !self.hierarchy_dirty {
            return;
        }
        
        // Sync all render meshes first
        self.root.sync_render_mesh(&mut self.meshes);
        
        // Rebuild the flat cache
        let mut object_id = 0;
        self.cached_render_instances = self.root.flatten_to_render_instances(
            &Transform::identity(), 
            &mut object_id,
            &self.meshes,
            &[],  // Empty path for root
            self.selected_path.as_ref()
        );
        
        self.hierarchy_dirty = false;
        self.dirty = true;  // Mark for JS update
    }

    /// Add mesh to scene storage, returns mesh_id
    fn add_mesh(&mut self, model: ModelVariant) -> MeshId {
        let mesh_id = MeshId::new(self.meshes.len());
        self.meshes.push(model);
        mesh_id
    }

    pub fn add_cube(&mut self, size: f32, position: [f32; 3]) -> usize {
        let half_edge_mesh = HalfEdgeMesh::create_cube(size);
        let model = ModelVariant::HalfEdgeMesh(ModelWrapper::new(half_edge_mesh));
        let mesh_id = self.add_mesh(model);
        
        let mut node = SceneGraphNode::with_transform(Transform::from_position_rotation_scale(
            position,
            [0.0, 0.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
        ));
        node.add_child(SceneGraphChild::Model(mesh_id));
        
        let child_count = self.root.edges.len();
        self.root.add_child(SceneGraphChild::Node(Box::new(node)));
        self.hierarchy_dirty = true;
        
        // Return the index of the newly added child
        child_count
    }

    pub fn add_sphere(&mut self, radius: f32, position: [f32; 3]) -> usize {
        // TODO: Implement sphere as HalfEdgeMesh or add Mesh variant to ModelVariant
        // For now, create a cube as placeholder
        console_log!("Warning: Sphere not yet implemented, creating cube instead");
        self.add_cube(radius * 2.0, position)
    }

    pub fn add_plane(&mut self, size: f32, position: [f32; 3]) -> usize {
        let half_edge_mesh = HalfEdgeMesh::create_plane(size);
        let model = ModelVariant::HalfEdgeMesh(ModelWrapper::new(half_edge_mesh));
        let mesh_id = self.add_mesh(model);
        
        let mut node = SceneGraphNode::with_transform(Transform::from_position_rotation_scale(
            position,
            [0.0, 0.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
        ));
        node.add_child(SceneGraphChild::Model(mesh_id));
        
        let child_count = self.root.edges.len();
        self.root.add_child(SceneGraphChild::Node(Box::new(node)));
        self.hierarchy_dirty = true;
        
        // Return the index of the newly added child
        child_count
    }

    pub fn remove_object(&mut self, id: usize) -> bool {
        if id < self.root.edges.len() {
            self.root.edges.remove(id);
            self.hierarchy_dirty = true;
            true
        } else {
            false
        }
    }

    pub fn update_transform(&mut self, id: usize, transform: Transform) -> bool {
        if id < self.root.edges.len() {
            if let SceneGraphChild::Node(node) = &mut self.root.edges[id].child {
                node.transform = transform;
                self.dirty = true;
                return true;
            }
        }
        false
    }

    pub fn raycast_closest_hit(&self, ray: Ray3) -> Option<WorldHitResponse> {
        let identity_transform = Transform::identity();
        let mut object_id = 0;
        let mut current_path = Vec::new();
        self.root.raycast_closest_hit(ray, &identity_transform, &mut object_id, &self.meshes, &mut current_path)
    }

    // Getters
    pub fn is_dirty(&self) -> bool { 
        self.dirty || self.hierarchy_dirty
    }
    pub fn clear_dirty(&mut self) { self.dirty = false; }
    pub fn object_count(&self) -> usize { self.root.edges.len() }
    
    /// Get flattened render instances for JavaScript
    pub fn get_render_instances(&mut self) -> &Vec<RenderInstance> {
        self.rebuild_cache();
        &self.cached_render_instances
    }
    
    pub fn clear(&mut self) {
        self.root = SceneGraphNode::new();
        self.meshes.clear();
        self.cached_render_instances.clear();
        self.hierarchy_dirty = true;
        self.selected_path = None;
    }

    /// Get mesh data by ID for JavaScript
    pub fn get_mesh(&self, mesh_id: usize) -> Option<&crate::Mesh> {
        if mesh_id < self.meshes.len() {
            Some(self.meshes[mesh_id].get_mesh())
        } else {
            None
        }
    }
    
    /// Select an item by edge ID path
    pub fn select_by_edge_path(&mut self, path: Vec<EdgeId>) -> bool {
        if self.edge_path_is_valid(&path) {
            self.selected_path = Some(path);
            self.hierarchy_dirty = true;  // Need to rebuild to mark selected instances
            true
        } else {
            false
        }
    }
    
    /// Deselect current selection
    pub fn deselect(&mut self) {
        if self.selected_path.is_some() {
            self.selected_path = None;
            self.hierarchy_dirty = true;  // Need to rebuild to unmark instances
        }
    }
    
    /// Check if an edge ID path is valid in the hierarchy
    fn edge_path_is_valid(&self, path: &[EdgeId]) -> bool {
        if path.is_empty() {
            return false;
        }
        
        let mut current = &self.root;
        for (i, &edge_id) in path.iter().enumerate() {
            // Find edge with matching ID
            let edge = current.edges.iter().find(|e| e.edge_id == edge_id);
            match edge {
                Some(e) => match &e.child {
                    SceneGraphChild::Node(node) => {
                        current = node;
                    }
                    SceneGraphChild::Model(_) => {
                        // Models are leaf nodes, path should end here
                        return i == path.len() - 1;
                    }
                }
                None => return false,
            }
        }
        true
    }
    
    /// Get the currently selected path
    pub fn get_selected_path(&self) -> Option<&Vec<EdgeId>> {
        self.selected_path.as_ref()
    }
    
    /// Select parent of currently selected item
    pub fn select_parent(&mut self) -> bool {
        if let Some(path) = &self.selected_path {
            if path.len() > 1 {
                let parent_path = path[..path.len()-1].to_vec();
                self.selected_path = Some(parent_path);
                self.hierarchy_dirty = true;
                return true;
            }
        }
        false
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
    selection_path: Vec<String>,  // Edge IDs as strings for JavaScript
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

        let id = self.core.add_cube(size, pos_array);
        console_log!("Adding cube with id {} at position [{}, {}, {}]", id, position[0], position[1], position[2]);
        id
    }

    /// Add a sphere to the scene
    pub fn add_sphere(&mut self, radius: f32, position: Vec<f32>) -> usize {
        let pos_array = [position[0], position[1], position[2]];

        let id = self.core.add_sphere(radius, pos_array);
        console_log!("Adding sphere with id {} at position [{}, {}, {}]", id, position[0], position[1], position[2]);
        id
    }

    /// Add a plane to the scene
    pub fn add_plane(&mut self, size: f32, position: Vec<f32>) -> usize {
        let pos_array = [position[0], position[1], position[2]];

        let id = self.core.add_plane(size, pos_array);
        console_log!("Adding plane with id {} at position [{}, {}, {}]", id, position[0], position[1], position[2]);
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
        let transform = Transform::from_position_rotation_scale(
            [position[0], position[1], position[2]],
            [rotation[0], rotation[1], rotation[2], rotation[3]],
            [scale[0], scale[1], scale[2]],
        );

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

    pub fn get_scene_data(&mut self) -> JsValue {
        serde_wasm_bindgen::to_value(self.core.get_render_instances()).unwrap()
    }

    /// Get mesh data by ID for JavaScript
    pub fn get_mesh_data(&self, mesh_id: usize) -> JsValue {
        if let Some(mesh) = self.core.get_mesh(mesh_id) {
            serde_wasm_bindgen::to_value(mesh).unwrap()
        } else {
            JsValue::NULL
        }
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
                    selection_path: world_hit.selection_path.iter().map(|edge_id| edge_id.to_string()).collect(),
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
    
    pub fn select_by_edge_path(&mut self, path_strings: Vec<String>) -> bool {
        // Parse EdgeId strings
        let mut path = Vec::new();
        for s in path_strings {
            match EdgeId::from_string(&s) {
                Ok(edge_id) => path.push(edge_id),
                Err(_) => {
                    console_log!("Invalid EdgeId in path: {}", s);
                    return false;
                }
            }
        }
        self.core.select_by_edge_path(path)
    }
    
    pub fn deselect(&mut self) {
        self.core.deselect();
    }
    
    pub fn select_parent(&mut self) -> bool {
        self.core.select_parent()
    }
    
    pub fn get_selected_path(&self) -> JsValue {
        if let Some(path) = self.core.get_selected_path() {
            // Convert EdgeIds to strings for JavaScript
            let string_path: Vec<String> = path.iter().map(|edge_id| edge_id.to_string()).collect();
            serde_wasm_bindgen::to_value(&string_path).unwrap()
        } else {
            JsValue::NULL
        }
    }
}