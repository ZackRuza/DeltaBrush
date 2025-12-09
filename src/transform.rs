use serde::{Serialize, Serializer};
use glam::{Mat4, Vec3 as GlamVec3, Quat};

#[derive(Clone)]
pub struct Transform {
    // Store the transformation as a 4x4 matrix
    matrix: Mat4,
}

// Custom serialization to output position, rotation, scale for JavaScript compatibility
impl Serialize for Transform {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        
        let (scale_vec3, rotation_quat, translation_vec3) = self.matrix.to_scale_rotation_translation();
        
        let translation = translation_vec3.to_array();
        let rotation = rotation_quat.normalize().to_array();
        let scale = scale_vec3.to_array();
        
        let mut state = serializer.serialize_struct("Transform", 3)?;
        state.serialize_field("translation", &translation)?;
        state.serialize_field("rotation", &rotation)?;
        state.serialize_field("scale", &scale)?;
        state.end()
    }
}

impl Transform {
    /// Create an identity transform
    pub fn identity() -> Self {
        Transform {
            matrix: Mat4::IDENTITY,
        }
    }

    /// Create a transform from position, rotation (quaternion), and scale
    pub fn from_position_rotation_scale(
        position: [f32; 3],
        rotation: [f32; 4], // quaternion [x, y, z, w]
        scale: [f32; 3],
    ) -> Self {
        let translation = GlamVec3::from_array(position);
        let quat = Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]).normalize();
        let scale_vec = GlamVec3::from_array(scale);
        
        Transform {
            matrix: Mat4::from_scale_rotation_translation(scale_vec, quat, translation),
        }
    }

    /// Create a transform from just position (identity rotation and scale)
    pub fn from_position(position: [f32; 3]) -> Self {
        Transform {
            matrix: Mat4::from_translation(GlamVec3::from_array(position)),
        }
    }

    /// Create a transform from just rotation
    pub fn from_rotation(rotation: [f32; 4]) -> Self {
        let quat = Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]).normalize();
        Transform {
            matrix: Mat4::from_quat(quat),
        }
    }

    /// Create a transform from just scale
    pub fn from_scale(scale: [f32; 3]) -> Self {
        Transform {
            matrix: Mat4::from_scale(GlamVec3::from_array(scale)),
        }
    }

    /// Get the underlying matrix
    pub fn matrix(&self) -> Mat4 {
        self.matrix
    }

    /// Get the inverse of this transform
    pub fn inverse(&self) -> Transform {
        Transform {
            matrix: self.matrix.inverse(),
        }
    }

    /// Compose this transform with a parent transform
    /// Returns parent * child (standard matrix multiplication order)
    pub fn compose_with_parent(&self, parent: &Transform) -> Transform {
        Transform {
            matrix: parent.matrix * self.matrix,
        }
    }

    /// Transform a point (applies translation)
    pub fn transform_point(&self, point: GlamVec3) -> GlamVec3 {
        self.matrix.transform_point3(point)
    }

    /// Transform a vector (no translation, just rotation and scale)
    pub fn transform_vector(&self, vector: GlamVec3) -> GlamVec3 {
        self.matrix.transform_vector3(vector)
    }
}
