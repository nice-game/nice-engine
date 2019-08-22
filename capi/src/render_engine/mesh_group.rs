use crate::game_graph_driver::{GGD_ImageData, GGD_MeshGroup};
use nice_engine::mesh_group::MeshGroup;

#[allow(non_snake_case)]
pub extern fn MeshGroup_Alloc() -> *mut GGD_MeshGroup {
	Box::into_raw(Box::new(MeshGroup::new()))
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshGroup_Free(this: *mut GGD_MeshGroup) {
	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub extern fn MeshGroup_SetSky(_this: *mut GGD_MeshGroup, _img: *mut GGD_ImageData) {}
