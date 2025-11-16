use serde::{Deserialize, Serialize};

use crate::{Material, Mesh, Point3, Transform, Transformable, algorithms::moller_trumbore_intersection_exterior_algebra, geometry::Ray3, scene::WorldHitResponse};



#[derive(Serialize, Deserialize, Clone)]
pub struct SceneObject {
    pub id: usize,
    pub mesh: Mesh,
    pub transform: Transform,
    pub material: Material,
}

impl SceneObject {
    pub fn raycast_closest_hit(&self, ray: Ray3) -> Option<WorldHitResponse> {
        let vert_coords = &self.mesh.vertex_coords;
        let transform = &self.transform;
        let transformed_ray = ray.inverse_transform(transform);
        let mut closest: Option<WorldHitResponse> = None;

        // Go through each triangle and perform ray intersection
        let mut chunks = self.mesh.face_indices.chunks_exact(3);
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
