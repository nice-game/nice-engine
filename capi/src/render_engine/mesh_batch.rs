use crate::game_graph_driver::GGD_MeshGroup;

#[allow(non_snake_case)]
pub extern fn MeshGroup_Alloc() -> *mut GGD_MeshGroup {
	Box::into_raw(Box::new(GGD_MeshGroup {}))
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshGroup_Free(this: *mut GGD_MeshGroup) {
	Box::from_raw(this);
}
