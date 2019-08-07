use std::sync::Mutex;
use std::sync::Arc;
use crate::mesh::Mesh;
use crate::direct_light::DirectLight;

pub struct World {
	meshes: Mutex<Vec<Mesh>>,
	lights: Mutex<Vec<DirectLight>>,
}
impl World {
	pub(crate) fn new() -> Arc<Self> {
		Arc::new(Self { meshes: Mutex::default(), lights: Mutex::default() })
	}

	pub fn add_mesh(&self, mesh: Mesh) {
		self.meshes.lock().unwrap().push(mesh);
	}

	pub(crate) fn meshes(&self) -> &Mutex<Vec<Mesh>> {
		&self.meshes
	}

	pub fn add_light(&self, light: DirectLight) {
		self.lights.lock().unwrap().push(light);
	}

	pub(crate) fn lights(&self) -> &Mutex<Vec<DirectLight>> {
		&self.lights
	}
}
