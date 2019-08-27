use crate::Context;
use crate::{mesh_group::MeshGroup, transform::Transform};
use cgmath::{prelude::*, vec4, Vector4};
use std::{f32::consts::PI, sync::Arc};

pub struct Camera {
	proj: Vector4<f32>,
	transform: Transform,
	mesh_group: Arc<MeshGroup>,
	pub exposure: f32,
}
impl Camera {
	pub fn new(ctx: &Context) -> Self {
		Self { proj: Vector4::zero(), transform: Transform::default(), exposure: 1.0, mesh_group: MeshGroup::new(ctx) }
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

	pub fn mesh_group(&self) -> &Arc<MeshGroup> {
		&self.mesh_group
	}

	pub fn mesh_group_mut(&mut self) -> &mut Arc<MeshGroup> {
		&mut self.mesh_group
	}

	pub(crate) fn projection(&self) -> Vector4<f32> {
		self.proj
	}
}

fn projection(aspect: f32, fovx: f32, znear: f32, zfar: f32) -> Vector4<f32> {
	let f = 1.0 / (fovx * PI / 360.0).tan();
	vec4(f / aspect, f, (zfar + znear) / (znear - zfar), zfar * znear / (znear - zfar))
}
