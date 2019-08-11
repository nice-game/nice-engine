use cgmath::{vec3, Vector3};

pub struct DirectLight {
	pub position: Vector3<f32>,
	pub color: Vector3<f32>,
	pub radius: f32,
}
impl DirectLight {
	pub fn new() -> Self {
		DirectLight { position: vec3(0.0, 0.0, 0.0), color: vec3(0.0, 0.0, 0.0), radius: 0.0 }
	}
}
