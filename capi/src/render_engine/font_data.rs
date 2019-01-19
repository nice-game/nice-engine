use super::GGD_ImageData;

#[allow(non_camel_case_types)]
pub struct GGD_FontData { }

#[allow(non_snake_case)]
pub extern fn FontData_Alloc() -> *mut GGD_FontData {
	Box::into_raw(Box::new(GGD_FontData { }))
}

#[allow(non_snake_case)]
pub unsafe extern fn FontData_Free(this: *mut GGD_FontData) {
	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub extern fn FontData_SetGlyph(_image: *mut GGD_FontData, _codepoint: u32, _img: *mut GGD_ImageData) {

}
