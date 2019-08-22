use std::collections::HashMap;
use std::sync::{LockResult, MutexGuard};
use crate::mesh::MeshInner;
use std::sync::{Arc, Mutex};

pub struct MeshGroup {
	meshes: Mutex<HashMap<usize, Arc<Mutex<MeshInner>>>>,
}
impl MeshGroup {
	pub fn new() -> Arc<Self> {
		Arc::new(Self { meshes: Mutex::default() })
	}

	pub(crate) fn lock(&self) -> LockResult<MutexGuard<HashMap<usize, Arc<Mutex<MeshInner>>>>> {
		self.meshes.lock()
	}
}
