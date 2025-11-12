use crate::{Transform, Transformable, Vec3};



#[derive(Debug, Clone, Copy)]
pub struct Point3 {
    pub vec3: Vec3,
}

impl Point3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Point3 {
            vec3: Vec3::new(x, y, z)
        }
    }
}

// Subtraction two points yields direction
impl std::ops::Sub for Point3 {
    type Output = Direction3;
    fn sub(self, rhs: Point3) -> Direction3 {
        Direction3 { vec3: Vec3 { 
            x: self.vec3.x - rhs.vec3.x,
            y: self.vec3.y - rhs.vec3.y,
            z: self.vec3.z - rhs.vec3.z,
        }}
    }
}



impl Transformable for Point3 {
    // Performs rotation, scale, then translation
    fn transform(&self, transform: &Transform) -> Self {
        // Rotate THEN scale
        let mut transformed = self.vec3.transform(transform);

        // Translate
        let t = Vec3 { 
            x: transform.position[0], 
            y: transform.position[1], 
            z: transform.position[2] 
        };
        transformed = transformed + t;

        Point3 {
            vec3: transformed
        }
    }

    // Inverts via inverse translation, inverse scale, and then inverse rotation
    fn inverse_transform(&self, transform: &Transform) -> Self {
        // Undo the translation
        let t = Vec3 { 
            x: transform.position[0], 
            y: transform.position[1], 
            z: transform.position[2] 
        };
        let transformed = self.vec3 - t;

        // Inverse scale and inverse rotation and return
        Point3 {
            vec3: transformed.inverse_transform(transform)
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Direction3 {
    pub vec3: Vec3
}

impl Transformable for Direction3 {
    fn transform(&self, transform: &Transform) -> Self {
        Direction3 {
            vec3: self.vec3.transform(transform)
        }
    }

    fn inverse_transform(&self, transform: &Transform) -> Self {
        Direction3 {
            vec3: self.vec3.inverse_transform(transform)
        }
    }
}

impl Direction3 {
    pub fn length(&self) -> f32 {
        return self.vec3.length()
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Ray3 {
    pub origin: Point3,
    // Direction must be access through a getter, where it is normalized if necessary
    direction: Direction3,
}

impl Ray3 {
    pub fn new(origin: Point3, direction: Direction3) -> Self {
        Ray3 {
            origin,
            direction
        }
    }

    // Getter for direction that normalizes if necessary
    pub fn direction(&self) -> Direction3 {
        if !self.direction.vec3.is_normalized() {
            // Normalize the direction if it's not already normalized
            let normalized = self.direction.vec3.normalize();
            Direction3 { vec3: normalized }
        } else {
            self.direction
        }
    }
}

impl Transformable for Ray3 {
    fn transform(&self, transform: &Transform) -> Self {
        Ray3 {
            origin: self.origin.transform(transform),
            direction: self.direction.transform(transform)
        }
    }

    fn inverse_transform(&self, transform: &Transform) -> Self {
        Ray3 {
            origin: self.origin.inverse_transform(transform),
            direction: self.direction.inverse_transform(transform)
        }
    }
}



#[derive(Clone)]
pub struct HitResponse {
    pub hit_position: Point3,
    pub hit_direction: Direction3,
}

impl Transformable for HitResponse {
    fn transform(&self, transform: &Transform) -> Self {
        HitResponse {
            hit_position: self.hit_position.transform(transform),
            hit_direction: self.hit_direction.transform(transform)
        }
    }

    fn inverse_transform(&self, transform: &Transform) -> Self {
        HitResponse {
            hit_position: self.hit_position.inverse_transform(transform),
            hit_direction: self.hit_direction.inverse_transform(transform)
        }
    }
}