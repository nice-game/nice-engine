use super::{ GGTransform, GGD_MeshBatch };
use nice_engine::camera::Camera;

#[allow(non_camel_case_types)]
pub type GGD_Camera = Camera;

#[allow(non_snake_case)]
pub extern fn Camera_Alloc() -> *mut GGD_Camera {
	Box::into_raw(Box::new(Camera { }))
}

#[allow(non_snake_case)]
pub unsafe extern fn Camera_Free(this: *mut GGD_Camera) {
	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub extern fn Camera_SetPerspective(_this: *mut GGD_Camera, _aspect: f32, _fovx: f32, _zNear: f32, _zFar: f32) {

}

#[allow(non_snake_case)]
pub extern fn Camera_SetMeshBatch(_this: *mut GGD_Camera, _mesh_batch: *mut GGD_MeshBatch) {

}

#[allow(non_snake_case)]
pub extern fn Camera_SetTransform(_this: *mut GGD_Camera, _transform: *mut GGTransform) {

}
