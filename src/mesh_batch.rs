use crate::mesh::Mesh;
use std::{collections::HashSet, sync::Arc};

pub struct MeshBatch {
	meshes: HashSet<Arc<Mesh>>,
}
impl MeshBatch {
	pub fn new() -> Self {
		Self { meshes: HashSet::new() }
	}

	pub fn insert_mesh(&mut self, mesh: Arc<Mesh>) {
		self.meshes.insert(mesh);
	}

	pub fn meshes(&self) -> &HashSet<Arc<Mesh>> {
		&self.meshes
	}
}
