use crate::mesh_batch::MeshBatch;
use std::sync::Arc;

pub struct Camera {
	aspect: f32,
	fovx: f32,
	znear: f32,
	zfar: f32,
	mesh_batch: Option<Arc<MeshBatch>>,
}
impl Camera {
	pub fn new() -> Self {
		Camera { aspect: 0.0, fovx: 0.0, znear: 0.0, zfar: 0.0, mesh_batch: None }
	}

	pub fn set_perspective(&mut self, aspect: f32, fovx: f32, znear: f32, zfar: f32) {
		self.aspect = aspect;
		self.fovx = fovx;
		self.znear = znear;
		self.zfar = zfar;
	}

	pub fn set_mesh_batch(&mut self, mesh_batch: Option<Arc<MeshBatch>>) {
		self.mesh_batch = mesh_batch;
	}
}
