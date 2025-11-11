use serde::{Deserialize, Serialize};

use crate::{Material, Mesh, Transform, Vec3, algorithms::moller_trumbore_intersection, scene::HitResponse};



#[derive(Serialize, Deserialize, Clone)]
pub struct SceneObject {
    pub id: usize,
    pub mesh_data: Mesh,
    pub transform: Transform,
    pub material: Material,
}

impl SceneObject {
    pub fn raycast_first_hit(&self, origin: Vec3, direction: Vec3) -> Option<HitResponse> {
        let verts = &self.mesh_data.vertex_coords;
        let mut closest: Option<HitResponse> = None;

        // Go through each triangle and perform ray intersection
        let mut chunks = self.mesh_data.face_indices.chunks_exact(3);
        for tri in &mut chunks {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            let v = |i: usize| Vec3::new(verts[3 * i], verts[3 * i + 1], verts[3 * i + 2]);
            
            if let Some(this_hit) = moller_trumbore_intersection(origin, direction, v(i0), v(i1), v(i2)) {
                let should_replace = match &closest {
                    None => true,
                    Some(existing_closest) => this_hit.hit_distance < existing_closest.hit_distance,
                };

                if should_replace {
                    closest = Some(this_hit);
                }
            }
        }

        if !chunks.remainder().is_empty() {
            crate::console_log!("Mesh indices not a multiple of 3. Trailing mesh indices ignored.");
        }

        closest
    }
}
