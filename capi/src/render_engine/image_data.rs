#[allow(non_snake_case)]
pub extern fn ImageData_Alloc() ->  *mut GGD_ImageData {
	Box::into_raw(Box::new(GGD_ImageData { }))
}

#[allow(non_snake_case)]
pub unsafe extern fn ImageData_Free(this: *mut GGD_ImageData) {
	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub extern fn ImageData_Prepare(this: *mut GGD_ImageData) {

}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GGD_ImageData { }
