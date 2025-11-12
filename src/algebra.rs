use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{BitXor, Sub, Mul, Add};

// Vector
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// Bivector
#[derive(Debug, Clone, Copy)]
pub struct Bivec3 {
    pub xy: f32,
    pub xz: f32,
    pub yz: f32
}

// Trivector
#[derive(Debug, Clone, Copy)]
pub struct Trivec3 {
    pub xyz: f32
}

// Multiplication by scalars
impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, scalar: f32) -> Vec3 {
        Vec3 { x: self.x * scalar, y: self.y * scalar, z: self.z * scalar }
    }
}
impl Mul<f32> for Bivec3 {
    type Output = Bivec3;
    fn mul(self, scalar: f32) -> Bivec3 {
        Bivec3 { xy: self.xy * scalar, xz: self.xz * scalar, yz: self.yz * scalar }
    }
}
impl Mul<f32> for Trivec3 {
    type Output = Trivec3;
    fn mul(self, scalar: f32) -> Trivec3 {
        Trivec3 { xyz: self.xyz * scalar }
    }
}
// f32 * Vec3
impl Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, vec: Vec3) -> Vec3 {
        Vec3 { x: vec.x * self, y: vec.y * self, z: vec.z * self }
    }
}
// f32 * Bivec3
impl Mul<Bivec3> for f32 {
    type Output = Bivec3;
    fn mul(self, biv: Bivec3) -> Bivec3 {
        Bivec3 { xy: biv.xy * self, xz: biv.xz * self, yz: biv.yz * self }
    }
}
// f32 * Trivec3
impl Mul<Trivec3> for f32 {
    type Output = Trivec3;
    fn mul(self, triv: Trivec3) -> Trivec3 {
        Trivec3 { xyz: triv.xyz * self }
    }
}
impl Add<Vec3> for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}
impl Add<Bivec3> for Bivec3 {
    type Output = Bivec3;
    fn add(self, rhs: Bivec3) -> Bivec3 {
        Bivec3 { xy: self.xy + rhs.xy, xz: self.xz + rhs.xz, yz: self.yz + rhs.yz }
    }
}
impl Add<Trivec3> for Trivec3 {
    type Output = Trivec3;
    fn add(self, rhs: Trivec3) -> Trivec3 {
        Trivec3 { xyz: self.xyz + rhs.xyz }
    }
}
impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
}
impl Sub for Bivec3 {
    type Output = Bivec3;
    fn sub(self, other: Bivec3) -> Bivec3 {
        Bivec3 { xy: self.xy - other.xy, xz: self.xz - other.xz, yz: self.yz - other.yz }
    }
}
impl Sub for Trivec3 {
    type Output = Trivec3;
    fn sub(self, other: Trivec3) -> Trivec3 {
        Trivec3 { xyz: self.xyz - other.xyz }
    }
}
// Vec3 ^ Vec3 -> Bivec3
impl BitXor for Vec3 {
    type Output = Bivec3;
    fn bitxor(self, other: Vec3) -> Bivec3 {
        Bivec3 {
            xy: self.x * other.y - self.y * other.x,
            xz: self.x * other.z - self.z * other.x,
            yz: self.y * other.z - self.z * other.y,
        }
    }
}
// Vec3 ^ Bivec3 -> Trivec3 and Bivec3 ^ Vec3 -> Trivec3
impl BitXor<Bivec3> for Vec3 {
    type Output = Trivec3;
    fn bitxor(self, other: Bivec3) -> Trivec3 {
        Trivec3 { xyz: self.x * other.yz - self.y * other.xz + self.z * other.xy }
    }
}
impl BitXor<Vec3> for Bivec3 {
    type Output = Trivec3;
    fn bitxor(self, other: Vec3) -> Trivec3 {
        Trivec3 { xyz: self.xy * other.z - self.xz * other.y + self.yz * other.x }
    }
}

#[wasm_bindgen]
impl Vec3 {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    pub fn normalize(&self) -> Vec3 {
        let len = self.length();
        if len > 0.0 {
            Vec3 { x: self.x / len, y: self.y / len, z: self.z / len }
        } else {
            *self
        }
    }
    pub fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

// Functions not visible to WASM interface
impl Vec3 {
    pub fn new_from_array(values: [f32; 3]) -> Vec3 {
        Vec3 { x: values[0], y: values[1], z: values[2] }
    }
    pub fn new_from_vec(values: Vec<f32>) -> Result<Vec3, String> {
        if values.len() != 3 {
            Err(format!("expected 3 elements for Vec3, got {}", values.len()))
        } else {
            Ok(Vec3 { x: values[0], y: values[1], z: values[2] })
        }
    }
}

// Implement Transformable for Vec3
impl crate::Transformable for Vec3 {
    /// Apply transform to a vector: scale THEN rotate
    fn transform(&self, transform: &crate::Transform) -> Self {
        // Scale
        let scaled = Vec3 { 
            x: self.x * transform.scale[0],
            y: self.y * transform.scale[1],
            z: self.z * transform.scale[2],
        };

        // Rotate adn return
        let q = crate::Transform::normalize_quat(transform.rotation);
        crate::Transform::rotate_vec3_by_quat(scaled, q)
    }

    /// Apply inverse transform: translate^-1 -> rotate^-1 -> scale^-1
    fn inverse_transform(&self, transform: &crate::Transform) -> Self {
        // Inverse rotation
        let q = crate::Transform::normalize_quat(transform.rotation);
        let q_conj = [-q[0], -q[1], -q[2], q[3]];
        let unrotated = crate::Transform::rotate_vec3_by_quat(*self, q_conj);
        
        // Undo scale (component-wise) and return
        // Sets to 0 if scale is 0
        let inv_x = if transform.scale[0] != 0.0 { 1.0 / transform.scale[0] } else { 0.0 };
        let inv_y = if transform.scale[1] != 0.0 { 1.0 / transform.scale[1] } else { 0.0 };
        let inv_z = if transform.scale[2] != 0.0 { 1.0 / transform.scale[2] } else { 0.0 };
        Vec3 { x: unrotated.x * inv_x, y: unrotated.y * inv_y, z: unrotated.z * inv_z }
    }
}
