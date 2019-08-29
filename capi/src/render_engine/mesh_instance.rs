use crate::{
	ctx,
	game_graph::GGTransform,
	game_graph_driver::{GGD_ImageData, GGD_MeshData, GGD_MeshGroup, GGD_MeshInstance},
};
use cgmath::{vec4, Quaternion};
use log::trace;
use nice_engine::transform::Transform;

#[allow(non_snake_case)]
pub unsafe extern fn MeshInstance_Alloc(group: *mut GGD_MeshGroup) -> *mut GGD_MeshInstance {
	trace!("MeshInstance_Alloc");

	let group = &mut *group;

	Box::into_raw(Box::new(GGD_MeshInstance::new(ctx::get(), group.clone())))
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshInstance_Free(this: *mut GGD_MeshInstance) {
	trace!("MeshInstance_Free");

	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshInstance_SetMeshData(this: *mut GGD_MeshInstance, mesh: *mut GGD_MeshData, _index: u32) {
	trace!("MeshInstance_SetMeshData");

	let this = &mut *this;
	let mesh = &mut *mesh;

	this.inner().set_mesh_data(Some(mesh.clone()));
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshInstance_SetMeshSubset(this: *mut GGD_MeshInstance, offset: u32, count: u32) {
	trace!("MeshInstance_SetMeshSubset");

	let this = &mut *this;

	this.inner().set_range(offset as usize..(offset + count) as usize);
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshInstance_SetImageData(this: *mut GGD_MeshInstance, image: *mut GGD_ImageData, layer: i32) {
	trace!("MeshInstance_SetImageData");

	let this = &mut *this;
	let image = &mut *image;

	println!("{}", layer);

	this.inner().set_tex(layer as usize, image.tex().unwrap().clone());
}

#[allow(non_snake_case)]
pub extern fn MeshInstance_SetAnimation(
	_this: *mut GGD_MeshInstance,
	_firstIndex: u32,
	_lastIndex: u32,
	_frameRate: f32,
) {
	trace!("MeshInstance_SetAnimation");
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshInstance_SetTransform(this: *mut GGD_MeshInstance, transform: *const GGTransform) {
	trace!("MeshInstance_SetTransform");

	let this = &mut *this;
	let transform = &*transform;

	let transform = Transform {
		pos: vec4(transform.Position.x, transform.Position.y, transform.Position.z, transform.Position.w),
		rot: Quaternion::new(transform.Rotation.w, transform.Rotation.x, transform.Rotation.y, transform.Rotation.z),
	};

	this.inner().set_transform(transform);
}

#[allow(non_snake_case)]
pub extern fn MeshInstance_SetBoneTransform(_this: *mut GGD_MeshInstance, _bone: u32, _pose: *const GGTransform) {
	trace!("MeshInstance_SetBoneTransform");
}
