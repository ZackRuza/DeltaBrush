use crate::{Transform, Transformable, Vec3};



#[derive(Debug, Clone, Copy)]
pub struct Point3 {
    pub position: Vec3,
}

impl Transformable for Point3 {
    // Performs rotation, scale, then translation
    fn transform(&self, transform: &Transform) -> Self {
        // Rotate THEN scale
        let mut transformed = self.position.transform(transform);

        // Translate
        let t = Vec3 { 
            x: transform.position[0], 
            y: transform.position[1], 
            z: transform.position[2] 
        };
        transformed = transformed + t;

        Point3 {
            position: transformed
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
        let transformed = self.position - t;

        // Inverse scale and inverse rotation and return
        Point3 {
            position: transformed.inverse_transform(transform)
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Direction3 {
    pub direction: Vec3
}

impl Transformable for Direction3 {
    fn transform(&self, transform: &Transform) -> Self {
        Direction3 {
            direction: self.direction.transform(transform)
        }
    }

    fn inverse_transform(&self, transform: &Transform) -> Self {
        Direction3 {
            direction: self.direction.inverse_transform(transform)
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Ray3 {
    pub origin: Point3,
    pub direction: Direction3,
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
