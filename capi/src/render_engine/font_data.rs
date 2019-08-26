use crate::game_graph_driver::{GGD_FontData, GGD_ImageData};
use log::trace;

#[allow(non_snake_case)]
pub extern fn FontData_Alloc() -> *mut GGD_FontData {
	trace!("FontData_Alloc");

	Box::into_raw(Box::new(GGD_FontData {}))
}

#[allow(non_snake_case)]
pub unsafe extern fn FontData_Free(this: *mut GGD_FontData) {
	trace!("FontData_Free");

	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub extern fn FontData_SetGlyph(
	_image: *mut GGD_FontData,
	_codepoint: u32,
	_img: *mut GGD_ImageData,
	_basex: f32,
	_basey: f32,
) {
	trace!("FontData_SetGlyph");
}
