use crate::{mesh_data::MeshData, transform::Transform};
use std::{
	hash::{Hash, Hasher},
	sync::Arc,
};

pub struct Mesh {
	id: Arc<()>,
	mesh_data: Option<Arc<MeshData>>,
	transform: Transform,
}
impl Mesh {
	pub fn new() -> Self {
		Self { id: Arc::new(()), mesh_data: None, transform: Transform::default() }
	}

	pub fn mesh_data(&self) -> &Option<Arc<MeshData>> {
		&self.mesh_data
	}

	pub fn set_mesh_data(&mut self, mesh_data: Option<Arc<MeshData>>) {
		self.mesh_data = mesh_data;
	}

	pub fn transform(&self) -> &Transform {
		&self.transform
	}

	pub fn transform_mut(&mut self) -> &mut Transform {
		&mut self.transform
	}
}
impl Hash for Mesh {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.id.hash(state);
	}
}
impl PartialEq for Mesh {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}
impl Eq for Mesh {}
