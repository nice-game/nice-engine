use super::{ GGMaterialLayer, GGTransform, GGD_ImageData, GGD_MeshBatch, GGD_MeshData };

#[allow(non_camel_case_types)]
pub struct GGD_MeshInstance { }

#[allow(non_snake_case)]
pub extern fn MeshInstance_Alloc(_batch: *mut GGD_MeshBatch) -> *mut GGD_MeshInstance {
	Box::into_raw(Box::new(GGD_MeshInstance { }))
}

#[allow(non_snake_case)]
pub unsafe extern fn MeshInstance_Free(this: *mut GGD_MeshInstance) {
	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub extern fn MeshInstance_SetMeshData(_this: *mut GGD_MeshInstance, _mesh: *mut GGD_MeshData, _index: u32) {

}

#[allow(non_snake_case)]
pub extern fn MeshInstance_SetImageData(_this: *mut GGD_MeshInstance, _image: *mut GGD_ImageData, _layer: GGMaterialLayer) {

}

#[allow(non_snake_case)]
pub extern fn MeshInstance_SetAnimation(_this: *mut GGD_MeshInstance, _firstIndex: u32, _lastIndex: u32, _frameRate: f32) {

}

#[allow(non_snake_case)]
pub extern fn MeshInstance_SetTransform(_this: *mut GGD_MeshInstance, _pose: *mut GGTransform) {

}

#[allow(non_snake_case)]
pub extern fn MeshInstance_SetBoneTransform(_this: *mut GGD_MeshInstance, _bone: u32, _pose: *mut GGTransform) {

}
