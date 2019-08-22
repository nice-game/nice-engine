use crate::{
	ctx,
	game_graph::GGTransform,
	game_graph_driver::{GGD_ImageData, GGD_MeshData, GGD_MeshGroup, GGD_MeshInstance},
};
use cgmath::{vec3, Quaternion};

#[allow(non_snake_case)]
pub unsafe extern fn MeshInstance_Alloc(group: *mut GGD_MeshGroup) -> *mut GGD_MeshInstance {
	let group = &mut *group;

	Box::into_raw(Box::new(GGD_MeshInstance::new(ctx::get(), group)))
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshInstance_Free(this: *mut GGD_MeshInstance) {
	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshInstance_SetMeshData(this: *mut GGD_MeshInstance, mesh: *mut GGD_MeshData, _index: u32) {
	let this = &mut *this;
	let mesh = &mut *mesh;

	this.lock().unwrap().set_mesh_data(Some(mesh.clone()));
}

#[allow(non_snake_case)]
pub extern fn MeshInstance_SetImageData(_this: *mut GGD_MeshInstance, _image: *mut GGD_ImageData, _layer: i32) {}

#[allow(non_snake_case)]
pub extern fn MeshInstance_SetAnimation(
	_this: *mut GGD_MeshInstance,
	_firstIndex: u32,
	_lastIndex: u32,
	_frameRate: f32,
) {
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshInstance_SetTransform(this: *mut GGD_MeshInstance, transform: *const GGTransform) {
	let this = &mut *this;
	let transform = &*transform;

	let mut lock = this.lock().unwrap();
	lock.transform_mut().pos = vec3(transform.Position.x, transform.Position.y, transform.Position.z);
	lock.transform_mut().rot =
		Quaternion::new(transform.Rotation.w, transform.Rotation.x, transform.Rotation.y, transform.Rotation.z);
}

#[allow(non_snake_case)]
pub extern fn MeshInstance_SetBoneTransform(_this: *mut GGD_MeshInstance, _bone: u32, _pose: *const GGTransform) {}
