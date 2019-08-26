use crate::{
	ctx,
	game_graph::GGTransform,
	game_graph_driver::{GGD_Camera, GGD_MeshGroup},
};
use cgmath::{vec4, Quaternion};
use log::trace;
use nice_engine::camera::Camera;
use std::sync::{Arc, Mutex};

#[allow(non_snake_case)]
pub unsafe extern fn Camera_Alloc() -> *mut GGD_Camera {
	trace!("Camera_Alloc");
	Box::into_raw(Box::new(Arc::new(Mutex::new(Camera::new(ctx::get())))))
}

#[allow(non_snake_case)]
pub unsafe extern fn Camera_Free(this: *mut GGD_Camera) {
	trace!("Camera_Free");

	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub unsafe extern fn Camera_SetPerspective(this: *mut GGD_Camera, aspect: f32, fovx: f32, zNear: f32, zFar: f32) {
	trace!("Camera_SetPerspective");

	let this = &mut *this;

	this.lock().unwrap().set_perspective(aspect, fovx, zNear, zFar);
}

#[allow(non_snake_case)]
pub unsafe extern fn Camera_SetMeshGroup(this: *mut GGD_Camera, mesh_group: *mut GGD_MeshGroup) {
	trace!("Camera_SetMeshGroup");

	let this = &mut *this;
	let mesh_group = &mut *mesh_group;

	*this.lock().unwrap().mesh_group_mut() = mesh_group.clone();
}

#[allow(non_snake_case)]
pub unsafe extern fn Camera_SetTransform(this: *mut GGD_Camera, transform: *const GGTransform) {
	trace!("Camera_SetTransform");

	let this = &mut *this;
	let transform = &*transform;

	let mut lock = this.lock().unwrap();
	lock.transform_mut().pos =
		vec4(transform.Position.x, transform.Position.y, transform.Position.z, transform.Position.w);
	lock.transform_mut().rot =
		Quaternion::new(transform.Rotation.w, transform.Rotation.x, transform.Rotation.y, transform.Rotation.z);
}
