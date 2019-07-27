use crate::{mesh_batch::MeshBatch, mesh_data::MeshData};
use std::sync::Arc;

pub struct Mesh {
	mesh_data: Option<Arc<MeshData>>,
}
impl Mesh {
	pub fn new(_mesh_batch: Arc<MeshBatch>) -> Self {
		Self { mesh_data: None }
	}

	pub fn mesh_data(&self) -> Option<&Arc<MeshData>> {
		self.mesh_data.as_ref()
	}

	pub fn set_mesh_data(&mut self, mesh_data: Option<Arc<MeshData>>) {
		self.mesh_data = mesh_data;
	}
}
