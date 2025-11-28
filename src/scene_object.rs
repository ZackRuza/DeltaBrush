use serde::{Serialize, Serializer};
use crate::{Material, Mesh, Point3, Transform, Transformable, algorithms::moller_trumbore_intersection_exterior_algebra, geometry::{HitResponse, Ray3}, model::ModelVariant};


#[derive(Clone)]
pub struct SceneObject {
    pub id: usize,
    pub model: ModelVariant,
    pub transform: Transform,
    pub material: Material,
}

/// World hit response holds the hit response in world coordinates, as well as the
/// distance and object ID
#[derive(Clone)]
pub struct WorldHitResponse {
    pub hit_response: HitResponse,
    pub distance: f32,
    pub object_id: usize,
}

impl SceneObject {
    /// Get the current renderable mesh
    pub fn get_mesh(&self) -> &Mesh {
        self.model.get_mesh()
    }

    /// Sync the render mesh with the underlying representation
    pub fn sync_render_mesh(&mut self) {
        self.model.sync_render_mesh();
    }

    pub fn raycast_closest_hit(&self, ray: Ray3) -> Option<WorldHitResponse> {
        let mesh = self.get_mesh();
        let transform = &self.transform;
        let transformed_ray = ray.inverse_transform(transform);
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
                let world_hit = this_hit.transform(transform);

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
                        object_id: self.id,
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

// Custom serialization - JavaScript always sees the current renderable mesh
impl Serialize for SceneObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SceneObject", 4)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("mesh", self.get_mesh())?;
        state.serialize_field("transform", &self.transform)?;
        state.serialize_field("material", &self.material)?;
        state.end()
    }
}