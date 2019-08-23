use cgmath::{prelude::*, Quaternion, Vector4};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Transform {
	pub pos: Vector4<f32>,
	pub rot: Quaternion<f32>,
}
impl Default for Transform {
	fn default() -> Self {
		Self { pos: Vector4::zero(), rot: Quaternion::one() }
	}
}
