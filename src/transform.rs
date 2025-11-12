use serde::{Deserialize, Serialize};
use crate::Vec3;

#[derive(Serialize, Deserialize, Clone)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 4], // quaternion (x, y, z, w)
    pub scale: [f32; 3],
}

impl Transform {
    #[inline]
    pub(crate) fn normalize_quat(q: [f32; 4]) -> [f32; 4] {
        let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
        let mag2 = x * x + y * y + z * z + w * w;
        if mag2 > 0.0 {
            let inv = mag2.sqrt().recip();
            [x * inv, y * inv, z * inv, w * inv]
        } else {
            // Identity rotation
            [0.0, 0.0, 0.0, 1.0]
        }
    }

    #[inline]
    pub(crate) fn rotate_vec3_by_quat(v: Vec3, q: [f32; 4]) -> Vec3 {
        // Rotates v by unit quaternion q using the efficient formula:
        // v' = v + 2w(q_xyz x v) + 2(q_xyz x (q_xyz x v))
        let (qx, qy, qz, qw) = (q[0], q[1], q[2], q[3]);
        let qv = Vec3 { x: qx, y: qy, z: qz };
        let uv = qv.cross(&v);
        let uuv = qv.cross(&uv);
        v + (2.0 * qw) * uv + 2.0 * uuv
    }
}
