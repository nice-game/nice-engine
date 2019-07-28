use crate::{mesh_data::MeshData, transform::Transform};
use std::{
	hash::{Hash, Hasher},
	sync::{Arc, Mutex},
};

pub struct Mesh {
	id: Arc<()>,
	inner: Mutex<MeshInner>,
}
impl Mesh {
	pub fn new() -> Arc<Self> {
		Arc::new(Self { id: Arc::new(()), inner: Mutex::new(MeshInner::new()) })
	}

	pub fn mesh_data(&self) -> Option<Arc<MeshData>> {
		self.inner.lock().unwrap().mesh_data.clone()
	}

	pub fn set_mesh_data(&self, mesh_data: Option<Arc<MeshData>>) {
		self.inner.lock().unwrap().mesh_data = mesh_data;
	}

	pub fn transform(&self) -> Transform {
		self.inner.lock().unwrap().transform.clone()
	}

	pub fn set_transform(&self, transform: Transform) {
		self.inner.lock().unwrap().transform = transform;
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

struct MeshInner {
	mesh_data: Option<Arc<MeshData>>,
	transform: Transform,
}
impl MeshInner {
	pub fn new() -> Self {
		Self { mesh_data: None, transform: Transform::default() }
	}
}
