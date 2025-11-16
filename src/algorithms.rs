use crate::Point3;
use crate::algebra::{Dual, InnerProduct};
use crate::{Vec3, geometry::{Ray3, Direction3, HitResponse}};

// The Möller–Trumbore intersection algorithm, implementation using some exterior algebra
pub fn moller_trumbore_intersection_exterior_algebra(ray: Ray3, a: Point3, b: Point3, c: Point3) -> Option<HitResponse> {
    let origin_vec3 = ray.origin.vec3;
    let direction_vec3 = ray.direction().vec3;
    
    
    let edge1 = (b - a).vec3;
    let edge2 = (c - a).vec3;

    let ray_edge2_plane = direction_vec3 ^ edge2;
    let det = edge1.inner(ray_edge2_plane.dual());
    if det > -f32::EPSILON && det < f32::EPSILON {
        return None; // The three vectors are not suitably linearly independent
    }

    let resize = 1.0 / det;
    let s = origin_vec3 - a.vec3;
    // TODO: This may be optimizable
    let u = resize * s.inner(ray_edge2_plane.dual());
    
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let s_edge1_plane = s ^ edge1;
    // TODO: This may be optimizable
    let v = resize * direction_vec3.inner(s_edge1_plane.dual());
    if v < 0.0 || u + v > 1.0 {
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
                    vec3: intersection
                },
                hit_direction: Direction3 {
                    vec3: scaled_direction_vec3
                }})
    } else {
        // Line intersection but no ray intersection
        None
    }
}




// Moller Trumbore Intersection algorithm. Largely based on the Wikipedia implementation.
#[allow(dead_code)]
pub fn moller_trumbore_intersection(ray: Ray3, a: Point3, b: Point3, c: Point3) -> Option<HitResponse> {
    let origin_vec3 = ray.origin.vec3;
    let direction_vec3 = ray.direction().vec3;
    
    // TODO: Not that cross and dot here take references, compared to the wikipedia implementation which
    //       takes in the object directly (it seems)
    
    let edge1 = (b - a).vec3;
    let edge2 = (c - a).vec3;

    let ray_cross_edge2 = direction_vec3.cross(&edge2);
    let det = edge1.dot(&ray_cross_edge2);
    if det > -f32::EPSILON && det < f32::EPSILON {
        return None;
    }

    let inv_det = 1.0 / det;
    let s = origin_vec3 - a.vec3;
    let u = inv_det * s.dot(&ray_cross_edge2);
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let s_cross_edge1 = s.cross(&edge1);
    let v = inv_det * direction_vec3.dot(&s_cross_edge1);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    // Calculate distance from origin to hit point
    let t = inv_det * edge2.dot(&s_cross_edge1);

    if t > f32::EPSILON {
        // Ray intersection
        let scaled_direction_vec3 = direction_vec3 * t;
        let intersection = origin_vec3 + scaled_direction_vec3;
        Some(
            HitResponse {
                hit_position: Point3 {
                    vec3: intersection
                },
                hit_direction: Direction3 {
                    vec3: scaled_direction_vec3
                }})
    } else {
        // Line intersection but no ray intersection
        None
    }
}