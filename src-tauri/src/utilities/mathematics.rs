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
        Self { x: 0.0, y: 0.0, z: 0.0 }
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
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

impl Angles {
    pub fn to_quaternion(&self) -> Quaternion {
        let half_cos_roll = self.x.cos() * 0.5;
        let half_sin_roll = self.x.sin() * 0.5;
        let half_cos_pitch = self.y.cos() * 0.5;
        let half_sin_pitch = self.y.sin() * 0.5;
        let half_cos_yaw = self.z.cos() * 0.5;
        let half_sin_yaw = self.z.sin() * 0.5;

        let x = half_sin_roll * half_cos_pitch * half_cos_yaw - half_cos_roll * half_sin_pitch * half_sin_yaw;
        let y = half_cos_roll * half_sin_pitch * half_cos_yaw + half_sin_roll * half_cos_pitch * half_sin_yaw;
        let z = half_cos_roll * half_cos_pitch * half_sin_yaw - half_sin_roll * half_sin_pitch * half_cos_yaw;
        let w = half_cos_roll * half_cos_pitch * half_cos_yaw + half_sin_roll * half_sin_pitch * half_sin_yaw;

        Quaternion::new(x, y, z, w)
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
            (sin_pitch / sin_pitch.abs()) * std::f64::consts::FRAC_PI_2
        } else {
            sin_pitch.asin()
        };

        let sin_yaw_cos_pitch = 2.0 * (self.w * self.z + self.x * self.y);
        let cos_yaw_cos_pitch = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw_angle = sin_yaw_cos_pitch.atan2(cos_yaw_cos_pitch);

        Angles::new(roll_angle, pitch_angle, yaw_angle)
    }
}
