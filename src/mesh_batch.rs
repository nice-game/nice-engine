use crate::mesh::Mesh;
use std::{collections::HashSet, sync::{Arc, Mutex}};

pub struct MeshBatch {
	meshes: Mutex<HashSet<Arc<Mesh>>>,
}
impl MeshBatch {
	pub fn new() -> Arc<Self> {
		Arc::new(Self { meshes: Mutex::default() })
	}

	pub fn insert_mesh(&self, mesh: Arc<Mesh>) {
		self.meshes.lock().unwrap().insert(mesh);
	}

	pub fn meshes(&self) -> &Mutex<HashSet<Arc<Mesh>>> {
		&self.meshes
	}
}
