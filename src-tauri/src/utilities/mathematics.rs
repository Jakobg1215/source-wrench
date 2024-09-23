mod matrices;
mod rotations;
mod vectors;

pub use matrices::{Matrix3, Matrix4};
pub use rotations::{Angles, Quaternion};
pub use vectors::{Vector2, Vector3, Vector4};

#[derive(Clone, Copy, Debug, Default)]
pub struct BoundingBox {
    pub minimum: Vector3,
    pub maximum: Vector3,
}

impl BoundingBox {
    pub fn is_valid(&self) -> bool {
        self.minimum.x <= self.maximum.x && self.minimum.y <= self.maximum.y && self.minimum.z <= self.maximum.z
    }
}

pub fn clamp<T: PartialOrd>(value: T, minimum: T, maximum: T) -> T {
    if value < minimum {
        return minimum;
    }

    if value > maximum {
        return maximum;
    }

    value
}
