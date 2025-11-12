use serde::{Deserialize, Serialize};

use crate::{Material, Mesh, Transform, Transformable, Vec3, algorithms::moller_trumbore_intersection, geometry::Ray3, scene::WorldHitResponse};



#[derive(Serialize, Deserialize, Clone)]
pub struct SceneObject {
    pub id: usize,
    pub mesh: Mesh,
    pub transform: Transform,
    pub material: Material,
}

impl SceneObject {
    pub fn raycast_first_hit(&self, ray: Ray3) -> Option<WorldHitResponse> {
        let verts = &self.mesh.vertex_coords;
        let mut closest: Option<WorldHitResponse> = None;

        let transformed_ray = ray.transform(&self.transform);

        // Go through each triangle and perform ray intersection
        let mut chunks = self.mesh.face_indices.chunks_exact(3);
        for tri in &mut chunks {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            let v = |i: usize| Vec3::new(verts[3 * i], verts[3 * i + 1], verts[3 * i + 2]);
            
            if let Some(this_hit) = moller_trumbore_intersection(transformed_ray, v(i0), v(i1), v(i2)) {
                let this_world_distance = this_hit.hit_direction.inverse_transform(&self.transform).length();
                match &closest {
                    None => {
                        closest = Some(WorldHitResponse { hit_response: this_hit, distance: this_world_distance });
                    },
                    Some(existing) => {
                        if this_world_distance < existing.distance {
                            closest = Some(WorldHitResponse { hit_response: this_hit, distance: this_world_distance });
                        }
                    },
                };
            }
        }

        if !chunks.remainder().is_empty() {
            crate::console_log!("Mesh indices not a multiple of 3. Trailing mesh indices ignored.");
        }

        closest
    }
}
