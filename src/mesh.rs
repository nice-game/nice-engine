use crate::mesh_data::MeshData;
use std::{
	hash::{Hash, Hasher},
	sync::{Arc, Mutex},
};

pub struct Mesh {
	id: Arc<()>,
	mesh_data: Mutex<Option<Arc<MeshData>>>,
}
impl Mesh {
	pub fn new() -> Arc<Self> {
		Arc::new(Self { id: Arc::new(()), mesh_data: Mutex::default() })
	}

	pub fn mesh_data(&self) -> Option<Arc<MeshData>> {
		self.mesh_data.lock().unwrap().clone()
	}

	pub fn set_mesh_data(&self, mesh_data: Option<Arc<MeshData>>) {
		*self.mesh_data.lock().unwrap() = mesh_data;
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
