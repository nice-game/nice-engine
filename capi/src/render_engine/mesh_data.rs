use crate::{ game_graph::{ GGIndexFormat, GGVertexFormat }, game_graph_driver::GGD_MeshData };
use libc::c_void;

#[allow(non_snake_case)]
pub extern fn MeshData_Alloc() -> *mut GGD_MeshData {
	Box::into_raw(Box::new(GGD_MeshData { }))
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshData_Free(this: *mut GGD_MeshData) {
	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub extern fn MeshData_SetVertexData(_this: *mut GGD_MeshData, _buffer: *const c_void, _count: u32, _format: GGVertexFormat) {

}

#[allow(non_snake_case)]
pub extern fn MeshData_GetVertexData(_this: *mut GGD_MeshData, _buffer: *mut c_void, _count: *mut u32, _format: *mut GGVertexFormat) {

}

#[allow(non_snake_case)]
pub extern fn MeshData_SetIndexData(_this: *mut GGD_MeshData, _buffer: *const c_void, _count: u32, _format: GGIndexFormat) {

}

#[allow(non_snake_case)]
pub extern fn MeshData_GetIndexData(_this: *mut GGD_MeshData, _buffer: *mut c_void, _count: *mut u32, _format: *mut GGIndexFormat) {

}
