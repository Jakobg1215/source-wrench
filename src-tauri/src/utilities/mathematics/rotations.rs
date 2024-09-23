use std::{
    f64::consts::{FRAC_PI_2, PI},
    ops::{Add, Sub},
};

use super::Matrix3;

/// Euler angles in radians. Roll, Pitch, Yaw
#[derive(Clone, Copy, Debug, Default)]
pub struct Angles {
    pub roll: f64,
    pub pitch: f64,
    pub yaw: f64,
}

impl Angles {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { roll: x, pitch: y, yaw: z }
    }
}

impl Angles {
    pub fn to_quaternion(&self) -> Quaternion {
        let half_cos_roll = (self.roll / 2.0).cos();
        let half_sin_roll = (self.roll / 2.0).sin();
        let half_cos_pitch = (self.pitch / 2.0).cos();
        let half_sin_pitch = (self.pitch / 2.0).sin();
        let half_cos_yaw = (self.yaw / 2.0).cos();
        let half_sin_yaw = (self.yaw / 2.0).sin();

        let x = half_sin_roll * half_cos_pitch * half_cos_yaw - half_cos_roll * half_sin_pitch * half_sin_yaw;
        let y = half_cos_roll * half_sin_pitch * half_cos_yaw + half_sin_roll * half_cos_pitch * half_sin_yaw;
        let z = half_cos_roll * half_cos_pitch * half_sin_yaw - half_sin_roll * half_sin_pitch * half_cos_yaw;
        let w = half_cos_roll * half_cos_pitch * half_cos_yaw + half_sin_roll * half_sin_pitch * half_sin_yaw;

        Quaternion::new(x, y, z, w)
    }

    pub fn to_matrix(&self) -> Matrix3 {
        let cos_roll = self.roll.cos();
        let sin_roll = self.roll.sin();
        let cos_pitch = self.pitch.cos();
        let sin_pitch = self.pitch.sin();
        let cos_yaw = self.yaw.cos();
        let sin_yaw = self.yaw.sin();

        Matrix3 {
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
        Self::new(self.roll * degrees_conversion, self.pitch * degrees_conversion, self.yaw * degrees_conversion)
    }

    pub fn to_radians(&self) -> Self {
        let radians_conversion = PI / 180.0;
        Self::new(self.roll * radians_conversion, self.pitch * radians_conversion, self.yaw * radians_conversion)
    }

    pub fn sum(&self) -> f64 {
        self.roll + self.pitch + self.yaw
    }

    pub fn normalize(&self) -> Self {
        let x = self.roll % (2.0 * PI);
        let y = self.pitch % (2.0 * PI);
        let z = self.yaw % (2.0 * PI);

        Self::new(
            if x > PI { x - 2.0 * PI } else { x },
            if y > PI { y - 2.0 * PI } else { y },
            if z > PI { z - 2.0 * PI } else { z },
        )
    }
}

impl Add for Angles {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.roll + rhs.roll, self.pitch + rhs.pitch, self.yaw + rhs.yaw)
    }
}

impl Sub for Angles {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.roll - rhs.roll, self.pitch - rhs.pitch, self.yaw - rhs.yaw)
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

    pub fn to_matrix(&self) -> Matrix3 {
        Matrix3 {
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

    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();

        if mag < f64::EPSILON {
            return Self::default();
        }

        Self::new(self.x / mag, self.y / mag, self.z / mag, self.w / mag)
    }
}
