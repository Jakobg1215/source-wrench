use std::f64::consts::{FRAC_PI_2, PI};
use std::ops::{Add, Div, Index, Mul, Sub};

#[derive(Clone, Copy, Debug, Default)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl Vector2 {
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

impl Sub for Vector2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn one() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }
}

impl Vector3 {
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn sum(&self) -> f64 {
        self.x + self.y + self.z
    }

    pub fn as_slice(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();

        if mag < f64::EPSILON {
            return Self::default();
        }

        Self::new(self.x / mag, self.y / mag, self.z / mag)
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self::new(self.y * other.z - self.z * other.x, self.z * other.x, self.x * other.y - self.y * other.x)
    }

    pub fn is_normalized(&self) -> bool {
        const EPSILON: f64 = 1e-15;
        let length_squared = self.x * self.x + self.y * self.y + self.z * self.z;
        (length_squared - 1.0).abs() < EPSILON
    }

    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul for Vector3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Div for Vector3 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl Index<usize> for Vector3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => unreachable!("Index out of bounds: the index is {}, but the length is 3", index),
        }
    }
}

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

#[derive(Clone, Copy, Debug, Default)]
pub struct Vector4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Vector4 {
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }
}

impl Vector4 {
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite() && self.w.is_finite()
    }
}

/// Euler angles in radians. Roll, Pitch, Yaw
#[derive(Clone, Copy, Debug, Default)]
pub struct Angles {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Angles {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

impl Angles {
    pub fn to_quaternion(&self) -> Quaternion {
        let half_cos_roll = (self.x / 2.0).cos();
        let half_sin_roll = (self.x / 2.0).sin();
        let half_cos_pitch = (self.y / 2.0).cos();
        let half_sin_pitch = (self.y / 2.0).sin();
        let half_cos_yaw = (self.z / 2.0).cos();
        let half_sin_yaw = (self.z / 2.0).sin();

        let x = half_sin_roll * half_cos_pitch * half_cos_yaw - half_cos_roll * half_sin_pitch * half_sin_yaw;
        let y = half_cos_roll * half_sin_pitch * half_cos_yaw + half_sin_roll * half_cos_pitch * half_sin_yaw;
        let z = half_cos_roll * half_cos_pitch * half_sin_yaw - half_sin_roll * half_sin_pitch * half_cos_yaw;
        let w = half_cos_roll * half_cos_pitch * half_cos_yaw + half_sin_roll * half_sin_pitch * half_sin_yaw;

        Quaternion::new(x, y, z, w)
    }

    pub fn to_matrix(&self) -> Matrix {
        let cos_roll = self.x.cos();
        let sin_roll = self.x.sin();
        let cos_pitch = self.y.cos();
        let sin_pitch = self.y.sin();
        let cos_yaw = self.z.cos();
        let sin_yaw = self.z.sin();

        Matrix {
            entries: [
                [
                    cos_yaw * cos_pitch,
                    cos_yaw * sin_pitch * sin_roll - sin_yaw * cos_roll,
                    cos_yaw * sin_pitch * cos_roll + sin_yaw * sin_roll,
                ],
                [
                    sin_yaw * cos_pitch,
                    sin_yaw * sin_pitch * sin_roll + cos_yaw * cos_roll,
                    sin_yaw * sin_pitch * cos_roll - cos_yaw * sin_roll,
                ],
                [-sin_pitch, cos_pitch * sin_roll, cos_pitch * cos_roll],
            ],
        }
    }

    pub fn to_degrees(&self) -> Self {
        let degrees_conversion = 180.0 / PI;
        Self::new(self.x * degrees_conversion, self.y * degrees_conversion, self.z * degrees_conversion)
    }

    pub fn to_radians(&self) -> Self {
        let radians_conversion = PI / 180.0;
        Self::new(self.x * radians_conversion, self.y * radians_conversion, self.z * radians_conversion)
    }

    pub fn sum(&self) -> f64 {
        self.x + self.y + self.z
    }
}

impl Sub for Angles {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Quaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            z: Default::default(),
            w: 1.0,
        }
    }
}

impl Quaternion {
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }
}

impl Quaternion {
    pub fn to_angles(&self) -> Angles {
        let sin_roll_cos_pitch = 2.0 * (self.w * self.x + self.y * self.z);
        let cos_roll_cos_pitch = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll_angle = sin_roll_cos_pitch.atan2(cos_roll_cos_pitch);

        let sin_pitch = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch_angle = if sin_pitch.abs() >= 1.0 {
            (sin_pitch / sin_pitch.abs()) * FRAC_PI_2
        } else {
            sin_pitch.asin()
        };

        let sin_yaw_cos_pitch = 2.0 * (self.w * self.z + self.x * self.y);
        let cos_yaw_cos_pitch = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw_angle = sin_yaw_cos_pitch.atan2(cos_yaw_cos_pitch);

        Angles::new(roll_angle, pitch_angle, yaw_angle)
    }

    pub fn to_matrix(&self) -> Matrix {
        Matrix {
            entries: [
                [
                    1.0 - 2.0 * self.y * self.y - 2.0 * self.z * self.z,
                    2.0 * self.x * self.y - 2.0 * self.w * self.z,
                    2.0 * self.x * self.z + 2.0 * self.w * self.y,
                ],
                [
                    2.0 * self.x * self.y + 2.0 * self.w * self.z,
                    1.0 - 2.0 * self.x * self.x - 2.0 * self.z * self.z,
                    2.0 * self.y * self.z - 2.0 * self.w * self.x,
                ],
                [
                    2.0 * self.x * self.z - 2.0 * self.w * self.y,
                    2.0 * self.y * self.z + 2.0 * self.w * self.x,
                    1.0 - 2.0 * self.x * self.x - 2.0 * self.y * self.y,
                ],
            ],
        }
    }
}

/// A 3 by 3 matrix.
#[derive(Clone, Copy, Debug, Default)]
pub struct Matrix {
    pub entries: [[f64; 3]; 3],
}

impl Matrix {
    pub fn identity() -> Self {
        Self {
            entries: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    pub fn to_angles(&self) -> Angles {
        let singularity = (self[0][0] * self[0][0] + self[1][0] * self[1][0]).sqrt();

        Angles {
            x: if singularity > f64::EPSILON { self[2][1].atan2(self[2][2]) } else { 0.0 },
            y: -self[2][0].atan2(singularity),
            z: if singularity > f64::EPSILON {
                self[1][0].atan2(self[0][0])
            } else {
                -self[0][1].atan2(self[1][1])
            },
        }
    }

    pub fn to_quaternion(&self) -> Quaternion {
        let trace = self.entries[0][0] + self.entries[1][1] + self.entries[2][2];
        if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            let w = 0.25 * s;
            let x = (self.entries[2][1] - self.entries[1][2]) / s;
            let y = (self.entries[0][2] - self.entries[2][0]) / s;
            let z = (self.entries[1][0] - self.entries[0][1]) / s;
            Quaternion { w, x, y, z }
        } else if self.entries[0][0] > self.entries[1][1] && self.entries[0][0] > self.entries[2][2] {
            let s = (1.0 + self.entries[0][0] - self.entries[1][1] - self.entries[2][2]).sqrt() * 2.0;
            let w = (self.entries[2][1] - self.entries[1][2]) / s;
            let x = 0.25 * s;
            let y = (self.entries[0][1] + self.entries[1][0]) / s;
            let z = (self.entries[0][2] + self.entries[2][0]) / s;
            Quaternion { w, x, y, z }
        } else if self.entries[1][1] > self.entries[2][2] {
            let s = (1.0 + self.entries[1][1] - self.entries[0][0] - self.entries[2][2]).sqrt() * 2.0;
            let w = (self.entries[0][2] - self.entries[2][0]) / s;
            let x = (self.entries[0][1] + self.entries[1][0]) / s;
            let y = 0.25 * s;
            let z = (self.entries[1][2] + self.entries[2][1]) / s;
            Quaternion { w, x, y, z }
        } else {
            let s = (1.0 + self.entries[2][2] - self.entries[0][0] - self.entries[1][1]).sqrt() * 2.0;
            let w = (self.entries[1][0] - self.entries[0][1]) / s;
            let x = (self.entries[0][2] + self.entries[2][0]) / s;
            let y = (self.entries[1][2] + self.entries[2][1]) / s;
            let z = 0.25 * s;
            Quaternion { w, x, y, z }
        }
    }

    pub fn transpose(&self) -> Self {
        Self {
            entries: [
                [self[0][0], self[1][0], self[2][0]],
                [self[0][1], self[1][1], self[2][1]],
                [self[0][2], self[1][2], self[2][2]],
            ],
        }
    }

    pub fn concatenate(&self, other: &Self) -> Self {
        Self {
            entries: [
                [
                    self[0][0] * other[0][0] + self[0][1] * other[1][0] + self[0][2] * other[2][0],
                    self[0][0] * other[0][1] + self[0][1] * other[1][1] + self[0][2] * other[2][1],
                    self[0][0] * other[0][2] + self[0][1] * other[1][2] + self[0][2] * other[2][2],
                ],
                [
                    self[1][0] * other[0][0] + self[1][1] * other[1][0] + self[1][2] * other[2][0],
                    self[1][0] * other[0][1] + self[1][1] * other[1][1] + self[1][2] * other[2][1],
                    self[1][0] * other[0][2] + self[1][1] * other[1][2] + self[1][2] * other[2][2],
                ],
                [
                    self[2][0] * other[0][0] + self[2][1] * other[1][0] + self[2][2] * other[2][0],
                    self[2][0] * other[0][1] + self[2][1] * other[1][1] + self[2][2] * other[2][1],
                    self[2][0] * other[0][2] + self[2][1] * other[1][2] + self[2][2] * other[2][2],
                ],
            ],
        }
    }
}

impl Index<usize> for Matrix {
    type Output = [f64; 3];

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
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
