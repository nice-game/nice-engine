use crate::transform::Transform;
use cgmath::{prelude::*, vec4, Vector4};
use std::f32::consts::PI;

pub struct Camera {
	proj: Vector4<f32>,
	transform: Transform,
	pub exposure: f32,
}
impl Camera {
	pub fn new() -> Self {
		Camera { proj: Vector4::zero(), transform: Transform::default(), exposure: 1.0 }
	}

	pub fn set_perspective(&mut self, aspect: f32, fovx: f32, znear: f32, zfar: f32) {
		self.proj = projection(aspect, fovx, znear, zfar);
	}

	pub fn transform(&self) -> &Transform {
		&self.transform
	}

	pub fn transform_mut(&mut self) -> &mut Transform {
		&mut self.transform
	}

	pub(crate) fn projection(&self) -> Vector4<f32> {
		self.proj
	}
}

fn projection(aspect: f32, fovx: f32, znear: f32, zfar: f32) -> Vector4<f32> {
	let f = 1.0 / (fovx * (PI / 360.0)).tan();
	vec4(f / aspect, f, (zfar + znear) / (znear - zfar), zfar * znear / (znear - zfar))
}
