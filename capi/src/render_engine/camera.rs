use crate::{
	game_graph::GGTransform,
	game_graph_driver::{GGD_Camera, GGD_MeshBatch},
};
use nice_engine::camera::Camera;

#[allow(non_snake_case)]
pub extern fn Camera_Alloc() -> *mut GGD_Camera {
	Box::into_raw(Box::new(Camera::new()))
}

#[allow(non_snake_case)]
pub unsafe extern fn Camera_Free(this: *mut GGD_Camera) {
	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub unsafe extern fn Camera_SetPerspective(this: *mut GGD_Camera, aspect: f32, fovx: f32, zNear: f32, zFar: f32) {
	let this_ref = &mut *this;
	this_ref.set_perspective(aspect, fovx, zNear, zFar);
}

#[allow(non_snake_case)]
pub extern fn Camera_SetMeshBatch(_this: *mut GGD_Camera, _mesh_batch: *mut GGD_MeshBatch) {}

#[allow(non_snake_case)]
pub extern fn Camera_SetTransform(_this: *mut GGD_Camera, _transform: *mut GGTransform) {}
