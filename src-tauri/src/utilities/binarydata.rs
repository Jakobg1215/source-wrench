use std::collections::HashMap;

use half::f16;

use super::mathematics::{clamp, Angles, Matrix, Quaternion, Vector2, Vector3, Vector4};

/// This is a utility to write binary data.
///
/// This is custom tailored to the MDL format.
#[derive(Default)]
pub struct DataWriter {
    data: Vec<u8>,
    string_table: HashMap<String, Vec<(usize, usize)>>,
}

impl DataWriter {
    pub fn write_unsigned_byte(&mut self, value: u8) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_unsigned_byte_array(&mut self, value: &Vec<u8>) {
        for byte in value {
            self.write_unsigned_byte(*byte);
        }
    }

    pub fn write_short(&mut self, value: i16) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_unsigned_short(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_short_array(&mut self, value: &Vec<i16>) {
        for short in value {
            self.write_short(*short);
        }
    }

    pub fn write_unsigned_short_array(&mut self, value: &Vec<u16>) {
        for short in value {
            self.write_unsigned_short(*short);
        }
    }

    pub fn write_int(&mut self, value: i32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_int_array(&mut self, value: &Vec<i32>) {
        for int in value {
            self.write_int(*int);
        }
    }

    pub fn write_long(&mut self, value: i64) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_float(&mut self, value: f32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_float_array(&mut self, value: &Vec<f32>) {
        for float in value {
            self.write_float(*float);
        }
    }

    pub fn write_index(&mut self) -> usize {
        let index = self.data.len();
        self.write_int(0);
        index
    }

    pub fn write_to_index(&mut self, index: usize, value: i32) {
        let bytes = value.to_le_bytes();

        self.data[index..index + bytes.len()].clone_from_slice(&bytes as &[u8]);
    }

    pub fn write_index_short(&mut self) -> usize {
        let index = self.data.len();
        self.write_short(0);
        index
    }

    pub fn write_to_index_short(&mut self, index: usize, value: i16) {
        let bytes = value.to_le_bytes();

        self.data[index..index + bytes.len()].clone_from_slice(&bytes as &[u8]);
    }

    pub fn write_vector2(&mut self, value: &Vector2) {
        self.write_float(value.x as f32);
        self.write_float(value.y as f32);
    }

    pub fn write_vector3(&mut self, value: &Vector3) {
        self.write_float(value.x as f32);
        self.write_float(value.y as f32);
        self.write_float(value.z as f32);
    }

    pub fn write_angles(&mut self, value: &Angles) {
        self.write_float(value.x as f32);
        self.write_float(value.y as f32);
        self.write_float(value.z as f32);
    }

    pub fn write_vector48(&mut self, value: &Vector3) {
        self.data.extend_from_slice(&f16::from_f64(value.x).to_le_bytes());
        self.data.extend_from_slice(&f16::from_f64(value.y).to_le_bytes());
        self.data.extend_from_slice(&f16::from_f64(value.z).to_le_bytes());
    }

    pub fn write_quaternion(&mut self, value: &Quaternion) {
        self.write_float(value.x as f32);
        self.write_float(value.y as f32);
        self.write_float(value.z as f32);
        self.write_float(value.w as f32);
    }

    pub fn write_quaternion64(&mut self, value: &Quaternion) {
        let x = clamp((value.x * 1048576.0) as i64 + 1048576, 0, 2097151);
        let y = clamp((value.y * 1048576.0) as i64 + 1048576, 0, 2097151);
        let z = clamp((value.z * 1048576.0) as i64 + 1048576, 0, 2097151);
        let w = (value.w < 0.0) as i64;
        self.write_long((x << 43) | (y << 22) | (z << 1) | w);
    }

    pub fn write_vector4(&mut self, value: &Vector4) {
        self.write_float(value.x as f32);
        self.write_float(value.y as f32);
        self.write_float(value.z as f32);
        self.write_float(value.w as f32);
    }

    pub fn write_matrix(&mut self, value: Matrix) {
        for i in 0..3 {
            for j in 0..4 {
                self.write_float(value[i][j] as f32);
            }
        }
    }

    pub fn write_string(&mut self, value: &str, length: usize) {
        let mut bytes = value.as_bytes().to_vec();
        bytes.resize(length, 0);
        self.data.extend_from_slice(&bytes);
    }

    pub fn add_string_to_table(&mut self, base: usize, value: &str) {
        let index = self.data.len();
        self.write_int(0);

        if !self.string_table.contains_key(value) {
            self.string_table.insert(value.to_string(), vec![(base, index)]);
            return;
        }

        self.string_table.get_mut(value).unwrap().push((base, index));
    }

    pub fn write_null_terminated_string(&mut self, value: &str) {
        self.data.extend_from_slice(value.as_bytes());
        self.data.push(0);
    }

    fn get_table_data(&mut self) -> Vec<(String, Vec<(usize, usize)>)> {
        let mut data = Vec::new();

        for (key, value) in self.string_table.drain() {
            data.push((key, value));
        }

        data
    }

    pub fn write_string_table(&mut self) {
        let mut table_data = self.get_table_data();

        table_data.sort_by(|(s1, _), (s2, _)| s2.len().cmp(&s1.len()));

        for (key, value) in table_data {
            let string_index = self.data.len();
            self.write_null_terminated_string(&key);

            for (base, index) in value {
                self.write_to_index(index, (string_index - base) as i32);
            }
        }
    }

    pub fn align(&mut self, alignment: usize) {
        let remainder = self.get_size() % alignment;

        if remainder == 0 {
            return;
        }

        let padding = alignment - remainder;

        self.data.resize(self.get_size() + padding, 0);
    }

    pub fn get_size(&self) -> usize {
        self.data.len()
    }

    pub fn get_data(&self) -> &[u8] {
        self.data.as_slice()
    }
}
