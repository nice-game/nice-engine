use crate::{ctx, game_graph::*, game_graph_driver::*};
use log::trace;
use nice_engine::mesh_group::MeshGroup;
use std::ptr::null_mut;

#[allow(non_snake_case)]
pub unsafe extern fn MeshGroup_Alloc(_cacheBuffer: *mut GGD_BufferInfo) -> *mut GGD_MeshGroup {
	trace!("MeshGroup_Alloc");

	Box::into_raw(Box::new(MeshGroup::new(ctx::get())))
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshGroup_Free(this: *mut GGD_MeshGroup) {
	trace!("MeshGroup_Free");

	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshGroup_SetSky(this: *mut GGD_MeshGroup, img: *mut GGD_ImageData) {
	trace!("MeshGroup_SetSky");

	let this = &mut *this;

	let img = if img == null_mut() { None } else { Some(&mut *img) };

	this.set_skybox(img.map(|img| img.tex().unwrap()));
}
