use cgmath::{prelude::*, Quaternion, Vector3};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Transform {
	pub pos: Vector3<f32>,
	pub rot: Quaternion<f32>,
}
impl Default for Transform {
	fn default() -> Self {
		Self { pos: Vector3::zero(), rot: Quaternion::one() }
	}
}
