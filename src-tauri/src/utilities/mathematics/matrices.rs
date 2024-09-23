use std::ops::Mul;

use super::Vector3;

#[derive(Clone, Copy, Debug)]
pub struct Matrix3 {
    pub entries: [[f64; 3]; 3],
}

impl Matrix3 {
    pub fn identity() -> Self {
        Self {
            entries: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    pub fn clean(&self) -> Self {
        Self {
            entries: [
                [
                    if self.entries[0][0].abs() < f64::EPSILON { 0.0 } else { self.entries[0][0] },
                    if self.entries[0][1].abs() < f64::EPSILON { 0.0 } else { self.entries[0][1] },
                    if self.entries[0][2].abs() < f64::EPSILON { 0.0 } else { self.entries[0][2] },
                ],
                [
                    if self.entries[1][0].abs() < f64::EPSILON { 0.0 } else { self.entries[1][0] },
                    if self.entries[1][1].abs() < f64::EPSILON { 0.0 } else { self.entries[1][1] },
                    if self.entries[1][2].abs() < f64::EPSILON { 0.0 } else { self.entries[1][2] },
                ],
                [
                    if self.entries[2][0].abs() < f64::EPSILON { 0.0 } else { self.entries[2][0] },
                    if self.entries[2][1].abs() < f64::EPSILON { 0.0 } else { self.entries[2][1] },
                    if self.entries[2][2].abs() < f64::EPSILON { 0.0 } else { self.entries[2][2] },
                ],
            ],
        }
    }
}

impl Default for Matrix3 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul for Matrix3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self {
            entries: [
                [
                    self.entries[0][0] * rhs.entries[0][0] + self.entries[0][1] * rhs.entries[1][0] + self.entries[0][2] * rhs.entries[2][0],
                    self.entries[0][0] * rhs.entries[0][1] + self.entries[0][1] * rhs.entries[1][1] + self.entries[0][2] * rhs.entries[2][1],
                    self.entries[0][0] * rhs.entries[0][2] + self.entries[0][1] * rhs.entries[1][2] + self.entries[0][2] * rhs.entries[2][2],
                ],
                [
                    self.entries[1][0] * rhs.entries[0][0] + self.entries[1][1] * rhs.entries[1][0] + self.entries[1][2] * rhs.entries[2][0],
                    self.entries[1][0] * rhs.entries[0][1] + self.entries[1][1] * rhs.entries[1][1] + self.entries[1][2] * rhs.entries[2][1],
                    self.entries[1][0] * rhs.entries[0][2] + self.entries[1][1] * rhs.entries[1][2] + self.entries[1][2] * rhs.entries[2][2],
                ],
                [
                    self.entries[2][0] * rhs.entries[0][0] + self.entries[2][1] * rhs.entries[1][0] + self.entries[2][2] * rhs.entries[2][0],
                    self.entries[2][0] * rhs.entries[0][1] + self.entries[2][1] * rhs.entries[1][1] + self.entries[2][2] * rhs.entries[2][1],
                    self.entries[2][0] * rhs.entries[0][2] + self.entries[2][1] * rhs.entries[1][2] + self.entries[2][2] * rhs.entries[2][2],
                ],
            ],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Matrix4 {
    pub entries: [[f64; 4]; 4],
}

impl Matrix4 {
    pub fn identity() -> Self {
        Self {
            entries: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
        }
    }

    pub fn new(position: Vector3, rotation: Matrix3) -> Self {
        Self {
            entries: [
                [rotation.entries[0][0], rotation.entries[0][1], rotation.entries[0][2], position.x],
                [rotation.entries[1][0], rotation.entries[1][1], rotation.entries[1][2], position.y],
                [rotation.entries[2][0], rotation.entries[2][1], rotation.entries[2][2], position.z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn transpose(&self) -> Self {
        let translation = Vector3::new(self.entries[0][3], self.entries[1][3], self.entries[2][3]);
        let row_x = Vector3::new(self.entries[0][0], self.entries[1][0], self.entries[2][0]);
        let row_y = Vector3::new(self.entries[0][1], self.entries[1][1], self.entries[2][1]);
        let row_z = Vector3::new(self.entries[0][2], self.entries[1][2], self.entries[2][2]);

        Self {
            entries: [
                [self.entries[0][0], self.entries[1][0], self.entries[2][0], -translation.dot(row_x)],
                [self.entries[0][1], self.entries[1][1], self.entries[2][1], -translation.dot(row_y)],
                [self.entries[0][2], self.entries[1][2], self.entries[2][2], -translation.dot(row_z)],
                [self.entries[0][3], self.entries[1][3], self.entries[2][3], self.entries[3][3]],
            ],
        }
    }

    pub fn clean(&self) -> Self {
        Self {
            entries: [
                [
                    if self.entries[0][0].abs() < f64::EPSILON { 0.0 } else { self.entries[0][0] },
                    if self.entries[0][1].abs() < f64::EPSILON { 0.0 } else { self.entries[0][1] },
                    if self.entries[0][2].abs() < f64::EPSILON { 0.0 } else { self.entries[0][2] },
                    if self.entries[0][3].abs() < f64::EPSILON { 0.0 } else { self.entries[0][3] },
                ],
                [
                    if self.entries[1][0].abs() < f64::EPSILON { 0.0 } else { self.entries[1][0] },
                    if self.entries[1][1].abs() < f64::EPSILON { 0.0 } else { self.entries[1][1] },
                    if self.entries[1][2].abs() < f64::EPSILON { 0.0 } else { self.entries[1][2] },
                    if self.entries[1][3].abs() < f64::EPSILON { 0.0 } else { self.entries[1][3] },
                ],
                [
                    if self.entries[2][0].abs() < f64::EPSILON { 0.0 } else { self.entries[2][0] },
                    if self.entries[2][1].abs() < f64::EPSILON { 0.0 } else { self.entries[2][1] },
                    if self.entries[2][2].abs() < f64::EPSILON { 0.0 } else { self.entries[2][2] },
                    if self.entries[2][3].abs() < f64::EPSILON { 0.0 } else { self.entries[2][3] },
                ],
                [
                    if self.entries[3][0].abs() < f64::EPSILON { 0.0 } else { self.entries[3][0] },
                    if self.entries[3][1].abs() < f64::EPSILON { 0.0 } else { self.entries[3][1] },
                    if self.entries[3][2].abs() < f64::EPSILON { 0.0 } else { self.entries[3][2] },
                    if self.entries[3][3].abs() < f64::EPSILON { 0.0 } else { self.entries[3][3] },
                ],
            ],
        }
    }
}

impl Default for Matrix4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul for Matrix4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self {
            entries: [
                [
                    self.entries[0][0] * rhs.entries[0][0]
                        + self.entries[0][1] * rhs.entries[1][0]
                        + self.entries[0][2] * rhs.entries[2][0]
                        + self.entries[0][3] * rhs.entries[3][0],
                    self.entries[0][0] * rhs.entries[0][1]
                        + self.entries[0][1] * rhs.entries[1][1]
                        + self.entries[0][2] * rhs.entries[2][1]
                        + self.entries[0][3] * rhs.entries[3][1],
                    self.entries[0][0] * rhs.entries[0][2]
                        + self.entries[0][1] * rhs.entries[1][2]
                        + self.entries[0][2] * rhs.entries[2][2]
                        + self.entries[0][3] * rhs.entries[3][2],
                    self.entries[0][0] * rhs.entries[0][3]
                        + self.entries[0][1] * rhs.entries[1][3]
                        + self.entries[0][2] * rhs.entries[2][3]
                        + self.entries[0][3] * rhs.entries[3][3],
                ],
                [
                    self.entries[1][0] * rhs.entries[0][0]
                        + self.entries[1][1] * rhs.entries[1][0]
                        + self.entries[1][2] * rhs.entries[2][0]
                        + self.entries[1][3] * rhs.entries[3][0],
                    self.entries[1][0] * rhs.entries[0][1]
                        + self.entries[1][1] * rhs.entries[1][1]
                        + self.entries[1][2] * rhs.entries[2][1]
                        + self.entries[1][3] * rhs.entries[3][1],
                    self.entries[1][0] * rhs.entries[0][2]
                        + self.entries[1][1] * rhs.entries[1][2]
                        + self.entries[1][2] * rhs.entries[2][2]
                        + self.entries[1][3] * rhs.entries[3][2],
                    self.entries[1][0] * rhs.entries[0][3]
                        + self.entries[1][1] * rhs.entries[1][3]
                        + self.entries[1][2] * rhs.entries[2][3]
                        + self.entries[1][3] * rhs.entries[3][3],
                ],
                [
                    self.entries[2][0] * rhs.entries[0][0]
                        + self.entries[2][1] * rhs.entries[1][0]
                        + self.entries[2][2] * rhs.entries[2][0]
                        + self.entries[2][3] * rhs.entries[3][0],
                    self.entries[2][0] * rhs.entries[0][1]
                        + self.entries[2][1] * rhs.entries[1][1]
                        + self.entries[2][2] * rhs.entries[2][1]
                        + self.entries[2][3] * rhs.entries[3][1],
                    self.entries[2][0] * rhs.entries[0][2]
                        + self.entries[2][1] * rhs.entries[1][2]
                        + self.entries[2][2] * rhs.entries[2][2]
                        + self.entries[2][3] * rhs.entries[3][2],
                    self.entries[2][0] * rhs.entries[0][3]
                        + self.entries[2][1] * rhs.entries[1][3]
                        + self.entries[2][2] * rhs.entries[2][3]
                        + self.entries[2][3] * rhs.entries[3][3],
                ],
                [
                    self.entries[3][0] * rhs.entries[0][0]
                        + self.entries[3][1] * rhs.entries[1][0]
                        + self.entries[3][2] * rhs.entries[2][0]
                        + self.entries[3][3] * rhs.entries[3][0],
                    self.entries[3][0] * rhs.entries[0][1]
                        + self.entries[3][1] * rhs.entries[1][1]
                        + self.entries[3][2] * rhs.entries[2][1]
                        + self.entries[3][3] * rhs.entries[3][1],
                    self.entries[3][0] * rhs.entries[0][2]
                        + self.entries[3][1] * rhs.entries[1][2]
                        + self.entries[3][2] * rhs.entries[2][2]
                        + self.entries[3][3] * rhs.entries[3][2],
                    self.entries[3][0] * rhs.entries[0][3]
                        + self.entries[3][1] * rhs.entries[1][3]
                        + self.entries[3][2] * rhs.entries[2][3]
                        + self.entries[3][3] * rhs.entries[3][3],
                ],
            ],
        }
    }
}
