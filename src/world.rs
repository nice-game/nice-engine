use std::sync::Mutex;
use std::sync::Arc;
use crate::mesh::Mesh;

pub struct World {
	meshes: Mutex<Vec<Mesh>>,
}
impl World {
	pub(crate) fn new() -> Arc<Self> {
		Arc::new(Self { meshes: Mutex::default() })
	}

	pub fn add_mesh(&self, mesh: Mesh) {
		self.meshes.lock().unwrap().push(mesh);
	}

	pub(crate) fn meshes(&self) -> &Mutex<Vec<Mesh>> {
		&self.meshes
	}
}
