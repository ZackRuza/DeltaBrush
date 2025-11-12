use crate::Transform;

/// Trait for types that can be transformed
pub trait Transformable {
    /// Apply a transform to this object
    fn transform(&self, transform: &Transform) -> Self;
    
    /// Apply the inverse transform to this object
    fn inverse_transform(&self, transform: &Transform) -> Self;
}
