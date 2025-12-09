use crate::{Point3, RenderInstance, Transform, Transformable, algorithms::moller_trumbore_intersection_exterior_algebra, geometry::{Ray3, WorldHitResponse}, model::ModelVariant};
use crate::render_instance::MeshId;


/// A child in the scene graph can be either another node or a model
#[derive(Clone)]
pub enum SceneGraphChild {
    Node(Box<SceneGraphNode>),
    Model(MeshId),  // mesh_id reference to central storage
}

/// A node in the scene graph hierarchy
/// TODO: instead of strictly holding transform, nodes should
///       be able to hold any properties that will be passed
///       down to the children
#[derive(Clone)]
pub struct SceneGraphNode {
    pub transform: Transform,
    pub children: Vec<SceneGraphChild>,
}

impl SceneGraphNode {
    /// Create a new scene graph node with identity transform
    pub fn new() -> Self {
        SceneGraphNode {
            transform: Transform::identity(),
            children: Vec::new(),
        }
    }

    /// Create a new scene graph node with a specific transform
    pub fn with_transform(transform: Transform) -> Self {
        SceneGraphNode {
            transform,
            children: Vec::new(),
        }
    }

    /// Add a child to this node
    pub fn add_child(&mut self, child: SceneGraphChild) {
        self.children.push(child);
    }

    /// Sync all render meshes in the subtree
    pub fn sync_render_mesh(&mut self, meshes: &mut [ModelVariant]) {
        for child in &mut self.children {
            match child {
                SceneGraphChild::Node(node) => {
                    node.sync_render_mesh(meshes);
                }
                SceneGraphChild::Model(mesh_id) => {
                    meshes[mesh_id.as_usize()].sync_render_mesh();
                }
            }
        }
    }

    /// Flatten the scene graph into a list of renderable instances
    /// This is what JavaScript needs for rendering
    pub fn flatten_to_render_instances(&self, parent_transform: &Transform, object_id: &mut usize, meshes: &[ModelVariant]) -> Vec<RenderInstance> {
        let world_transform = self.transform.compose_with_parent(parent_transform);
        let mut instances = Vec::new();

        for child in &self.children {
            match child {
                SceneGraphChild::Node(child_node) => {
                    // Recursively flatten child nodes
                    instances.extend(child_node.flatten_to_render_instances(&world_transform, object_id, meshes));
                }
                SceneGraphChild::Model(mesh_id) => {
                    // Add this model as a render instance
                    instances.push(RenderInstance {
                        mesh_id: *mesh_id,
                        transform: world_transform.clone(),
                        id: *object_id,
                    });
                    *object_id += 1;
                }
            }
        }

        instances
    }

    /// Perform raycast against this node and all children
    /// Returns the closest hit in world coordinates
    pub fn raycast_closest_hit(&self, ray: Ray3, parent_transform: &Transform, object_id: &mut usize, meshes: &[ModelVariant]) -> Option<WorldHitResponse> {
        // Compose this node's transform with the parent's
        let world_transform = self.transform.compose_with_parent(parent_transform);
        
        let mut closest: Option<WorldHitResponse> = None;

        // Check all children
        for child in &self.children {
            match child {
                SceneGraphChild::Node(child_node) => {
                    // Recursively check child nodes
                    if let Some(hit) = child_node.raycast_closest_hit(ray, &world_transform, object_id, meshes) {
                        let should_replace = match &closest {
                            None => true,
                            Some(existing) => hit.distance < existing.distance,
                        };
                        if should_replace {
                            closest = Some(hit);
                        }
                    }
                }
                SceneGraphChild::Model(mesh_id) => {
                    // Check ray intersection with this model
                    if let Some(hit) = Self::raycast_model(ray, &meshes[mesh_id.as_usize()], &world_transform, *object_id) {
                        let should_replace = match &closest {
                            None => true,
                            Some(existing) => hit.distance < existing.distance,
                        };
                        if should_replace {
                            closest = Some(hit);
                        }
                    }
                    *object_id += 1;
                }
            }
        }

        closest
    }

    /// Raycast against a single model with a given world transform
    fn raycast_model(ray: Ray3, model: &ModelVariant, world_transform: &Transform, object_id: usize) -> Option<WorldHitResponse> {
        let mesh = model.get_mesh();
        let transformed_ray = ray.inverse_transform(world_transform);
        let mut closest: Option<WorldHitResponse> = None;

        // Go through each triangle and perform ray intersection
        let vert_coords = &mesh.vertex_coords;
        let mut chunks = mesh.face_indices.chunks_exact(3);
        for tri in &mut chunks {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            let p = |i: usize| Point3::new(vert_coords[3 * i], vert_coords[3 * i + 1], vert_coords[3 * i + 2]);
            
            if let Some(this_hit)
                = moller_trumbore_intersection_exterior_algebra(transformed_ray, p(i0), p(i1), p(i2)) {
                
                // The hit response was in local coordinates. Transform to world coordinates.
                let world_hit = this_hit.transform(world_transform);

                let this_world_distance = world_hit.hit_direction.length();
                let should_update = match &closest {
                    None => true,
                    Some(existing) =>
                        this_world_distance < existing.distance,
                };

                if should_update {
                    closest = Some(WorldHitResponse {
                        hit_response: world_hit,
                        distance: this_world_distance,
                        object_id,
                    });
                }
            }
        }

        if !chunks.remainder().is_empty() {
            crate::console_log!("Mesh indices not a multiple of 3. Trailing mesh indices ignored.");
        }

        closest
    }
}