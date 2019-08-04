use crate::{
	game_graph::{GGImageUsage, GGPixelFormat},
	game_graph_driver::{GGD_Camera, GGD_FontData, GGD_ImageData},
};
use libc::c_void;

#[allow(non_snake_case)]
pub extern fn ImageData_Alloc(_usage: GGImageUsage, _x: u32, _y: u32, _format: GGPixelFormat) -> *mut GGD_ImageData {
	Box::into_raw(Box::new(GGD_ImageData {}))
}

#[allow(non_snake_case)]
pub unsafe extern fn ImageData_Free(this: *mut GGD_ImageData) {
	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub extern fn ImageData_SetPixelData(
	_image: *mut GGD_ImageData,
	_buffer: *const c_void,
	_x: u32,
	_y: u32,
	_format: GGPixelFormat,
) {
}

#[allow(non_snake_case)]
pub extern fn ImageData_GetPixelData(
	_image: *mut GGD_ImageData,
	_buffer: *mut c_void,
	_x: *mut u32,
	_y: *mut u32,
	_format: *mut GGPixelFormat,
) {
}

#[allow(non_snake_case)]
pub extern fn ImageData_DrawCamera(_dst: *mut GGD_ImageData, _src: *mut GGD_Camera) {}

#[allow(non_snake_case)]
pub extern fn ImageData_DrawImage(
	_dst: *mut GGD_ImageData,
	_src: *mut GGD_ImageData,
	_x: f32,
	_y: f32,
	_w: f32,
	_h: f32,
) {
}

#[allow(non_snake_case)]
pub extern fn ImageData_DrawText(
	_dst: *mut GGD_ImageData,
	_src: *mut GGD_FontData,
	_x: f32,
	_y: f32,
	_text: *const char,
) {
}
