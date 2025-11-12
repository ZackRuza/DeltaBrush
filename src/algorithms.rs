use crate::Point3;
use crate::{Vec3, geometry::{Ray3, Direction3, HitResponse}};

// The Möller–Trumbore intersection algorithm (using some exterior algebra)
// Returns the hit position vector and the distance from the origin to said vector
pub fn moller_trumbore_intersection(ray: Ray3, a: Vec3, b: Vec3, c: Vec3) -> Option<HitResponse> {
    let origin_vec3 = ray.origin.position;
    let direction_vec3 = ray.direction.direction;
    
    let edge1 = b - a;
    let edge2 = c - a;

    let ray_edge2_plane = direction_vec3 ^ edge2;
    let volume = ray_edge2_plane ^ edge1;
    if volume.xyz > -f32::EPSILON && volume.xyz < f32::EPSILON {
        return None; // The three vectors are not suitably linearly independent
    }

    let resize = 1.0 / volume.xyz;
    let s = origin_vec3 - a;
    let u = resize * (s ^ ray_edge2_plane).xyz;
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let s_edge1_plane = s ^ edge1;
    let v = resize * (direction_vec3 ^ s_edge1_plane).xyz;
    if v < 0.0 || v > 1.0 {
        return None;
    }

    // Calculate distance from origin to hit point
    let t = resize * (edge2 ^ s_edge1_plane).xyz;

    if t > f32::EPSILON {
        // Ray intersection
        let scaled_direction_vec3 = direction_vec3 * t;
        let intersection = origin_vec3 + scaled_direction_vec3;
        Some(
            HitResponse {
                hit_position: Point3 {
                    position: intersection
                },
                hit_direction: Direction3 {
                    direction: scaled_direction_vec3
                }})
    } else {
        // Line intersection but no ray intersection
        None
    }
}
