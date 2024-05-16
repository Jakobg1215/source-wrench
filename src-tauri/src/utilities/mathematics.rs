use std::f64::consts::{FRAC_PI_2, PI};
use std::ops::{Add, Index, IndexMut, Sub};

#[derive(Debug, Clone, Copy, Default)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl Sub for Vector2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::default()
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
            return Self::zero();
        }

        Self::new(self.x / mag, self.y / mag, self.z / mag)
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self::new(self.y * other.z - self.z * other.x, self.z * other.x, self.x * other.y - self.y * other.x)
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

#[derive(Debug, Clone, Copy, Default)]
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

    pub fn zero() -> Self {
        Self::default()
    }
}

/// Euler angles in radians. Roll, Pitch, Yaw
#[derive(Debug, Clone, Copy, Default)]
pub struct Angles {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Angles {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::default()
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

impl From<Quaternion> for Angles {
    fn from(value: Quaternion) -> Self {
        value.to_angles()
    }
}

impl Sub for Angles {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Quaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Quaternion {
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
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
}

impl From<Angles> for Quaternion {
    fn from(value: Angles) -> Self {
        value.to_quaternion()
    }
}

/// A 3 by 4 matrix.
#[derive(Debug, Clone, Copy, Default)]
pub struct Matrix {
    pub entries: [[f64; 4]; 3],
}

impl Matrix {
    pub fn zero() -> Self {
        Self::default()
    }

    pub fn new<T: Into<Angles>>(orientation: T, position: Vector3) -> Self {
        let mut new_matrix = Self::default();
        let angles = orientation.into();

        let cos_roll = angles.x.cos();
        let sin_roll = angles.x.sin();
        let cos_pitch = angles.y.cos();
        let sin_pitch = angles.y.sin();
        let cos_yaw = angles.z.cos();
        let sin_yaw = angles.z.sin();

        new_matrix[0][0] = cos_yaw * cos_pitch;
        new_matrix[1][0] = sin_yaw * cos_pitch;
        new_matrix[2][0] = -sin_pitch;

        new_matrix[0][1] = cos_yaw * sin_pitch * sin_roll - sin_yaw * cos_roll;
        new_matrix[1][1] = sin_yaw * sin_pitch * sin_roll + cos_yaw * cos_roll;
        new_matrix[2][1] = cos_pitch * sin_roll;

        new_matrix[0][2] = cos_yaw * sin_pitch * cos_roll + sin_yaw * sin_roll;
        new_matrix[1][2] = sin_yaw * sin_pitch * cos_roll - cos_yaw * sin_roll;
        new_matrix[2][2] = cos_pitch * cos_roll;

        new_matrix[0][3] = position.x;
        new_matrix[1][3] = position.y;
        new_matrix[2][3] = position.z;

        new_matrix
    }

    pub fn identity() -> Self {
        let mut new_matrix = Self::zero();
        new_matrix[0][0] = 1.0;
        new_matrix[1][1] = 1.0;
        new_matrix[2][2] = 1.0;
        new_matrix
    }
}

impl Matrix {
    pub fn to_angles(&self) -> Angles {
        let mut angles = Angles::zero();

        let singularity = (self[0][0] * self[0][0] + self[1][0] * self[1][0]).sqrt();

        if singularity > f64::EPSILON {
            angles.x = self[2][1].atan2(self[2][2]);
            angles.y = -self[2][0].atan2(singularity);
            angles.z = self[1][0].atan2(self[0][0]);
        } else {
            angles.x = 0.0;
            angles.y = -self[2][0].atan2(singularity);
            angles.z = -self[0][1].atan2(self[1][1]);
        }

        angles
    }

    pub fn to_vector(&self) -> Vector3 {
        Vector3::new(self[0][3], self[1][3], self[2][3])
    }

    pub fn transpose(&self) -> Self {
        let mut new_matrix = Self::zero();

        new_matrix[0][0] = self[0][0];
        new_matrix[0][1] = self[1][0];
        new_matrix[0][2] = self[2][0];

        new_matrix[1][0] = self[0][1];
        new_matrix[1][1] = self[1][1];
        new_matrix[1][2] = self[2][1];

        new_matrix[2][0] = self[0][2];
        new_matrix[2][1] = self[1][2];
        new_matrix[2][2] = self[2][2];

        let position = self.to_vector();

        new_matrix[0][3] = -position.dot(&Vector3::new(self[0][0], self[0][1], self[0][2]));
        new_matrix[1][3] = -position.dot(&Vector3::new(self[1][0], self[1][1], self[1][2]));
        new_matrix[2][3] = -position.dot(&Vector3::new(self[2][0], self[2][1], self[2][2]));

        new_matrix
    }

    pub fn concatenate(&self, other: &Self) -> Self {
        let mut new_matrix = Self::zero();

        for i in 0..3 {
            for j in 0..4 {
                new_matrix[i][j] = self[i][0] * other[0][j] + self[i][1] * other[1][j] + self[i][2] * other[2][j];
            }
        }

        new_matrix[0][3] += self[0][3];
        new_matrix[1][3] += self[1][3];
        new_matrix[2][3] += self[2][3];

        new_matrix
    }
}

impl Index<usize> for Matrix {
    type Output = [f64; 4];

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for Matrix {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}
