use crate::{direct_light::DirectLight};
use std::sync::{Arc, Mutex};

pub struct World {
	lights: Mutex<Vec<DirectLight>>,
}
impl World {
	pub(crate) fn new() -> Arc<Self> {
		Arc::new(Self { lights: Mutex::default() })
	}

	pub fn add_light(&self, light: DirectLight) {
		self.lights.lock().unwrap().push(light);
	}

	pub(crate) fn lights(&self) -> &Mutex<Vec<DirectLight>> {
		&self.lights
	}
}
