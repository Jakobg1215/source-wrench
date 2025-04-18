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

    pub fn add_point(&mut self, point: Vector3) {
        if !self.is_valid() {
            self.minimum = point;
            self.maximum = point;
            return;
        }

        self.minimum.x = self.minimum.x.min(point.x);
        self.minimum.y = self.minimum.y.min(point.y);
        self.minimum.z = self.minimum.z.min(point.z);

        self.maximum.x = self.maximum.x.max(point.x);
        self.maximum.y = self.maximum.y.max(point.y);
        self.maximum.z = self.maximum.z.max(point.z);
    }

    pub fn center(&self) -> Vector3 {
        (self.minimum + self.maximum) * 0.5
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
