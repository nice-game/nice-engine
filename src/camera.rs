pub struct Camera {
	aspect: f32,
	fovx: f32,
	znear: f32,
	zfar: f32,
}
impl Camera {
	pub fn new() -> Self {
		Camera { aspect: 0.0, fovx: 0.0, znear: 0.0, zfar: 0.0 }
	}

	pub fn set_perspective(&mut self, aspect: f32, fovx: f32, znear: f32, zfar: f32) {
		self.aspect = aspect;
		self.fovx = fovx;
		self.znear = znear;
		self.zfar = zfar;
	}
}
