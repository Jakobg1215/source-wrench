use std::ops::Mul;

use serde::{Deserialize, Serialize};

use super::{Angles, Quaternion, Vector3};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Matrix3([[f64; 3]; 3]);

impl Matrix3 {
    pub fn new(entries: [[f64; 3]; 3]) -> Self {
        Self(entries)
    }

    pub fn to_angles(&self) -> Angles {
        Angles::new(
            self.0[2][1].atan2(self.0[2][2]),
            -self.0[2][0].atan2((self.0[0][0] * self.0[0][0] + self.0[1][0] * self.0[1][0]).sqrt()),
            self.0[1][0].atan2(self.0[0][0]),
        )
    }

    pub fn to_quaternion(&self) -> Quaternion {
        let trace = self.0[0][0] + self.0[1][1] + self.0[2][2];

        if trace > 0.0 {
            let scale = 0.5 / (trace + 1.0).sqrt();
            Quaternion::new(
                (self.0[2][1] - self.0[1][2]) * scale,
                (self.0[0][2] - self.0[2][0]) * scale,
                (self.0[1][0] - self.0[0][1]) * scale,
                0.25 / scale,
            )
        } else if self.0[0][0] > self.0[1][1] && self.0[0][0] > self.0[2][2] {
            let scale = 2.0 * (1.0 + self.0[0][0] - self.0[1][1] - self.0[2][2]).sqrt();
            Quaternion::new(
                0.25 * scale,
                (self.0[0][1] + self.0[1][0]) / scale,
                (self.0[0][2] + self.0[2][0]) / scale,
                (self.0[2][1] - self.0[1][2]) / scale,
            )
        } else if self.0[1][1] > self.0[2][2] {
            let scale = 2.0 * (1.0 + self.0[1][1] - self.0[0][0] - self.0[2][2]).sqrt();
            Quaternion::new(
                (self.0[0][1] + self.0[1][0]) / scale,
                0.25 * scale,
                (self.0[1][2] + self.0[2][1]) / scale,
                (self.0[0][2] - self.0[2][0]) / scale,
            )
        } else {
            let scale = 2.0 * (1.0 + self.0[2][2] - self.0[0][0] - self.0[1][1]).sqrt();
            Quaternion::new(
                (self.0[0][2] + self.0[2][0]) / scale,
                (self.0[1][2] + self.0[2][1]) / scale,
                0.25 * scale,
                (self.0[1][0] - self.0[0][1]) / scale,
            )
        }
    }

    pub fn rotate_vector(&self, vector: Vector3) -> Vector3 {
        Vector3 {
            x: vector.x * self.0[0][0] + vector.y * self.0[1][0] + vector.z * self.0[2][0],
            y: vector.x * self.0[0][1] + vector.y * self.0[1][1] + vector.z * self.0[2][1],
            z: vector.x * self.0[0][2] + vector.y * self.0[1][2] + vector.z * self.0[2][2],
        }
    }
}

impl Default for Matrix3 {
    fn default() -> Self {
        Self([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]])
    }
}

impl From<Angles> for Matrix3 {
    fn from(value: Angles) -> Self {
        value.to_matrix()
    }
}

impl From<Quaternion> for Matrix3 {
    fn from(value: Quaternion) -> Self {
        value.to_matrix()
    }
}

impl Mul for Matrix3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self([
            [
                self.0[0][0] * rhs.0[0][0] + self.0[0][1] * rhs.0[1][0] + self.0[0][2] * rhs.0[2][0],
                self.0[0][0] * rhs.0[0][1] + self.0[0][1] * rhs.0[1][1] + self.0[0][2] * rhs.0[2][1],
                self.0[0][0] * rhs.0[0][2] + self.0[0][1] * rhs.0[1][2] + self.0[0][2] * rhs.0[2][2],
            ],
            [
                self.0[1][0] * rhs.0[0][0] + self.0[1][1] * rhs.0[1][0] + self.0[1][2] * rhs.0[2][0],
                self.0[1][0] * rhs.0[0][1] + self.0[1][1] * rhs.0[1][1] + self.0[1][2] * rhs.0[2][1],
                self.0[1][0] * rhs.0[0][2] + self.0[1][1] * rhs.0[1][2] + self.0[1][2] * rhs.0[2][2],
            ],
            [
                self.0[2][0] * rhs.0[0][0] + self.0[2][1] * rhs.0[1][0] + self.0[2][2] * rhs.0[2][0],
                self.0[2][0] * rhs.0[0][1] + self.0[2][1] * rhs.0[1][1] + self.0[2][2] * rhs.0[2][1],
                self.0[2][0] * rhs.0[0][2] + self.0[2][1] * rhs.0[1][2] + self.0[2][2] * rhs.0[2][2],
            ],
        ])
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Matrix4([[f64; 4]; 4]);

impl Matrix4 {
    pub fn new(rotation: impl Into<Matrix3>, translation: Vector3) -> Self {
        let rotation_matrix = rotation.into();
        Self([
            [rotation_matrix.0[0][0], rotation_matrix.0[0][1], rotation_matrix.0[0][2], translation.x],
            [rotation_matrix.0[1][0], rotation_matrix.0[1][1], rotation_matrix.0[1][2], translation.y],
            [rotation_matrix.0[2][0], rotation_matrix.0[2][1], rotation_matrix.0[2][2], translation.z],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn entries(&self) -> [[f64; 4]; 4] {
        self.0
    }

    fn determinant(&self) -> f64 {
        self.0[0][0]
            * (self.0[1][1] * (self.0[2][2] * self.0[3][3] - self.0[2][3] * self.0[3][2])
                - self.0[1][2] * (self.0[2][1] * self.0[3][3] - self.0[2][3] * self.0[3][1])
                + self.0[1][3] * (self.0[2][1] * self.0[3][2] - self.0[2][2] * self.0[3][1]))
            - self.0[0][1]
                * (self.0[1][0] * (self.0[2][2] * self.0[3][3] - self.0[2][3] * self.0[3][2])
                    - self.0[1][2] * (self.0[2][0] * self.0[3][3] - self.0[2][3] * self.0[3][0])
                    + self.0[1][3] * (self.0[2][0] * self.0[3][2] - self.0[2][2] * self.0[3][0]))
            + self.0[0][2]
                * (self.0[1][0] * (self.0[2][1] * self.0[3][3] - self.0[2][3] * self.0[3][1])
                    - self.0[1][1] * (self.0[2][0] * self.0[3][3] - self.0[2][3] * self.0[3][0])
                    + self.0[1][3] * (self.0[2][0] * self.0[3][1] - self.0[2][1] * self.0[3][0]))
            - self.0[0][3]
                * (self.0[1][0] * (self.0[2][1] * self.0[3][2] - self.0[2][2] * self.0[3][1])
                    - self.0[1][1] * (self.0[2][0] * self.0[3][2] - self.0[2][2] * self.0[3][0])
                    + self.0[1][2] * (self.0[2][0] * self.0[3][1] - self.0[2][1] * self.0[3][0]))
    }

    pub fn inverse(&self) -> Self {
        let determinant = self.determinant();
        assert!(determinant != 0.0);
        let inverse_determinant = 1.0 / determinant;
        Self([
            [
                (self.0[1][1] * (self.0[2][2] * self.0[3][3] - self.0[2][3] * self.0[3][2])
                    - self.0[1][2] * (self.0[2][1] * self.0[3][3] - self.0[2][3] * self.0[3][1])
                    + self.0[1][3] * (self.0[2][1] * self.0[3][2] - self.0[2][2] * self.0[3][1]))
                    * inverse_determinant,
                -(self.0[0][1] * (self.0[2][2] * self.0[3][3] - self.0[2][3] * self.0[3][2])
                    - self.0[0][2] * (self.0[2][1] * self.0[3][3] - self.0[2][3] * self.0[3][1])
                    + self.0[0][3] * (self.0[2][1] * self.0[3][2] - self.0[2][2] * self.0[3][1]))
                    * inverse_determinant,
                (self.0[0][1] * (self.0[1][2] * self.0[3][3] - self.0[1][3] * self.0[3][2])
                    - self.0[0][2] * (self.0[1][1] * self.0[3][3] - self.0[1][3] * self.0[3][1])
                    + self.0[0][3] * (self.0[1][1] * self.0[3][2] - self.0[1][2] * self.0[3][1]))
                    * inverse_determinant,
                -(self.0[0][1] * (self.0[1][2] * self.0[2][3] - self.0[1][3] * self.0[2][2])
                    - self.0[0][2] * (self.0[1][1] * self.0[2][3] - self.0[1][3] * self.0[2][1])
                    + self.0[0][3] * (self.0[1][1] * self.0[2][2] - self.0[1][2] * self.0[2][1]))
                    * inverse_determinant,
            ],
            [
                -(self.0[1][0] * (self.0[2][2] * self.0[3][3] - self.0[2][3] * self.0[3][2])
                    - self.0[1][2] * (self.0[2][0] * self.0[3][3] - self.0[2][3] * self.0[3][0])
                    + self.0[1][3] * (self.0[2][0] * self.0[3][2] - self.0[2][2] * self.0[3][0]))
                    * inverse_determinant,
                (self.0[0][0] * (self.0[2][2] * self.0[3][3] - self.0[2][3] * self.0[3][2])
                    - self.0[0][2] * (self.0[2][0] * self.0[3][3] - self.0[2][3] * self.0[3][0])
                    + self.0[0][3] * (self.0[2][0] * self.0[3][2] - self.0[2][2] * self.0[3][0]))
                    * inverse_determinant,
                -(self.0[0][0] * (self.0[1][2] * self.0[3][3] - self.0[1][3] * self.0[3][2])
                    - self.0[0][2] * (self.0[1][0] * self.0[3][3] - self.0[1][3] * self.0[3][0])
                    + self.0[0][3] * (self.0[1][0] * self.0[3][2] - self.0[1][2] * self.0[3][0]))
                    * inverse_determinant,
                (self.0[0][0] * (self.0[1][2] * self.0[2][3] - self.0[1][3] * self.0[2][2])
                    - self.0[0][2] * (self.0[1][0] * self.0[2][3] - self.0[1][3] * self.0[2][0])
                    + self.0[0][3] * (self.0[1][0] * self.0[2][2] - self.0[1][2] * self.0[2][0]))
                    * inverse_determinant,
            ],
            [
                (self.0[1][0] * (self.0[2][1] * self.0[3][3] - self.0[2][3] * self.0[3][1])
                    - self.0[1][1] * (self.0[2][0] * self.0[3][3] - self.0[2][3] * self.0[3][0])
                    + self.0[1][3] * (self.0[2][0] * self.0[3][1] - self.0[2][1] * self.0[3][0]))
                    * inverse_determinant,
                -(self.0[0][0] * (self.0[2][1] * self.0[3][3] - self.0[2][3] * self.0[3][1])
                    - self.0[0][1] * (self.0[2][0] * self.0[3][3] - self.0[2][3] * self.0[3][0])
                    + self.0[0][3] * (self.0[2][0] * self.0[3][1] - self.0[2][1] * self.0[3][0]))
                    * inverse_determinant,
                (self.0[0][0] * (self.0[1][1] * self.0[3][3] - self.0[1][3] * self.0[3][1])
                    - self.0[0][1] * (self.0[1][0] * self.0[3][3] - self.0[1][3] * self.0[3][0])
                    + self.0[0][3] * (self.0[1][0] * self.0[3][1] - self.0[1][1] * self.0[3][0]))
                    * inverse_determinant,
                -(self.0[0][0] * (self.0[1][1] * self.0[2][3] - self.0[1][3] * self.0[2][1])
                    - self.0[0][1] * (self.0[1][0] * self.0[2][3] - self.0[1][3] * self.0[2][0])
                    + self.0[0][3] * (self.0[1][0] * self.0[2][1] - self.0[1][1] * self.0[2][0]))
                    * inverse_determinant,
            ],
            [
                -(self.0[1][0] * (self.0[2][1] * self.0[3][2] - self.0[2][2] * self.0[3][1])
                    - self.0[1][1] * (self.0[2][0] * self.0[3][2] - self.0[2][2] * self.0[3][0])
                    + self.0[1][2] * (self.0[2][0] * self.0[3][1] - self.0[2][1] * self.0[3][0]))
                    * inverse_determinant,
                (self.0[0][0] * (self.0[2][1] * self.0[3][2] - self.0[2][2] * self.0[3][1])
                    - self.0[0][1] * (self.0[2][0] * self.0[3][2] - self.0[2][2] * self.0[3][0])
                    + self.0[0][2] * (self.0[2][0] * self.0[3][1] - self.0[2][1] * self.0[3][0]))
                    * inverse_determinant,
                -(self.0[0][0] * (self.0[1][1] * self.0[3][2] - self.0[1][2] * self.0[3][1])
                    - self.0[0][1] * (self.0[1][0] * self.0[3][2] - self.0[1][2] * self.0[3][0])
                    + self.0[0][2] * (self.0[1][0] * self.0[3][1] - self.0[1][1] * self.0[3][0]))
                    * inverse_determinant,
                (self.0[0][0] * (self.0[1][1] * self.0[2][2] - self.0[1][2] * self.0[2][1])
                    - self.0[0][1] * (self.0[1][0] * self.0[2][2] - self.0[1][2] * self.0[2][0])
                    + self.0[0][2] * (self.0[1][0] * self.0[2][1] - self.0[1][1] * self.0[2][0]))
                    * inverse_determinant,
            ],
        ])
    }
}

impl Default for Matrix4 {
    fn default() -> Self {
        Self([[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]])
    }
}

impl Mul for Matrix4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self([
            [
                self.0[0][0] * rhs.0[0][0] + self.0[0][1] * rhs.0[1][0] + self.0[0][2] * rhs.0[2][0] + self.0[0][3] * rhs.0[3][0],
                self.0[0][0] * rhs.0[0][1] + self.0[0][1] * rhs.0[1][1] + self.0[0][2] * rhs.0[2][1] + self.0[0][3] * rhs.0[3][1],
                self.0[0][0] * rhs.0[0][2] + self.0[0][1] * rhs.0[1][2] + self.0[0][2] * rhs.0[2][2] + self.0[0][3] * rhs.0[3][2],
                self.0[0][0] * rhs.0[0][3] + self.0[0][1] * rhs.0[1][3] + self.0[0][2] * rhs.0[2][3] + self.0[0][3] * rhs.0[3][3],
            ],
            [
                self.0[1][0] * rhs.0[0][0] + self.0[1][1] * rhs.0[1][0] + self.0[1][2] * rhs.0[2][0] + self.0[1][3] * rhs.0[3][0],
                self.0[1][0] * rhs.0[0][1] + self.0[1][1] * rhs.0[1][1] + self.0[1][2] * rhs.0[2][1] + self.0[1][3] * rhs.0[3][1],
                self.0[1][0] * rhs.0[0][2] + self.0[1][1] * rhs.0[1][2] + self.0[1][2] * rhs.0[2][2] + self.0[1][3] * rhs.0[3][2],
                self.0[1][0] * rhs.0[0][3] + self.0[1][1] * rhs.0[1][3] + self.0[1][2] * rhs.0[2][3] + self.0[1][3] * rhs.0[3][3],
            ],
            [
                self.0[2][0] * rhs.0[0][0] + self.0[2][1] * rhs.0[1][0] + self.0[2][2] * rhs.0[2][0] + self.0[2][3] * rhs.0[3][0],
                self.0[2][0] * rhs.0[0][1] + self.0[2][1] * rhs.0[1][1] + self.0[2][2] * rhs.0[2][1] + self.0[2][3] * rhs.0[3][1],
                self.0[2][0] * rhs.0[0][2] + self.0[2][1] * rhs.0[1][2] + self.0[2][2] * rhs.0[2][2] + self.0[2][3] * rhs.0[3][2],
                self.0[2][0] * rhs.0[0][3] + self.0[2][1] * rhs.0[1][3] + self.0[2][2] * rhs.0[2][3] + self.0[2][3] * rhs.0[3][3],
            ],
            [
                self.0[3][0] * rhs.0[0][0] + self.0[3][1] * rhs.0[1][0] + self.0[3][2] * rhs.0[2][0] + self.0[3][3] * rhs.0[3][0],
                self.0[3][0] * rhs.0[0][1] + self.0[3][1] * rhs.0[1][1] + self.0[3][2] * rhs.0[2][1] + self.0[3][3] * rhs.0[3][1],
                self.0[3][0] * rhs.0[0][2] + self.0[3][1] * rhs.0[1][2] + self.0[3][2] * rhs.0[2][2] + self.0[3][3] * rhs.0[3][2],
                self.0[3][0] * rhs.0[0][3] + self.0[3][1] * rhs.0[1][3] + self.0[3][2] * rhs.0[2][3] + self.0[3][3] * rhs.0[3][3],
            ],
        ])
    }
}
