#[allow(non_camel_case_types)]
pub struct GGD_MeshBatch { }

#[allow(non_snake_case)]
pub extern fn MeshBatch_Alloc() -> *mut GGD_MeshBatch {
	Box::into_raw(Box::new(GGD_MeshBatch { }))
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshBatch_Free(this: *mut GGD_MeshBatch) {
	Box::from_raw(this);
}
