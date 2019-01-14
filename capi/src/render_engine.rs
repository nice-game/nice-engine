mod window;

use self::window::{ GGD_WindowInfo, GGD_Window, Window_Alloc, Window_Free, Window_IsValid, Window_Resize }; // Window_Draw
use std::os::raw::c_char;

pub const RENDER_ENGINE: GGD_RenderEngine = GGD_RenderEngine {
	Name: "nIce Engine".as_ptr() as _,
	Priority: 5,
	Validate: None,
	Shutdown: None,

	Window_Alloc: Window_Alloc,
	Window_Free: Window_Free,
	Window_IsValid: Window_IsValid,
	Window_Resize: Window_Resize,
	// Window_Draw: Window_Draw,

	// MeshData_Alloc: extern fn () -> *mut GGD_MeshData,
	// MeshData_Free: extern fn (*mut GGD_MeshData),
	// MeshData_Prepare: extern fn (*mut GGD_MeshData),
	// MeshData_SetCacheData: None,
	// MeshData_GetCacheData: None,
	// MeshData_SetDistanceData: None,
	// MeshData_GetDistanceData: None,
	// MeshData_SetVertexData: extern fn (*mut GGD_MeshData, buffer: *const c_void, count: u32, format: GGVertexFormat),
	// MeshData_GetVertexData: extern fn (*mut GGD_MeshData, buffer: *mut c_void, count: *mut u32, format: *mut GGVertexFormat),
	// MeshData_SetIndexData: extern fn (*mut GGD_MeshData, buffer: *const c_void, count: u32, format: GGIndexFormat),
	// MeshData_GetIndexData: extern fn (*mut GGD_MeshData, buffer: *mut c_void, count: *mut u32, format: *mut GGIndexFormat),
	// MeshData_UseIndexData: None,

	// ImageData_Alloc: extern fn () ->  *mut GGD_ImageData,
	// ImageData_Free: extern fn (*mut GGD_ImageData),
	// ImageData_Prepare: extern fn (*mut GGD_ImageData),
	// ImageData_SetCacheData: None,
	// ImageData_GetCacheData: None,
	// ImageData_SetPixelData: extern fn (image: *mut GGD_ImageData, buffer: *const c_void, x: u32, y: u32, z: u32, format: GGPixelFormat),
	// ImageData_GetPixelData: extern fn (image: *mut GGD_ImageData, buffer: *mut c_void, x: *mut u32, y: *mut u32, z: *mut u32, format: *mut GGPixelFormat),
	// ImageData_Blur: extern fn (dst: *mut GGD_ImageData, src: *mut GGD_ImageData, radius: f32),

	// MeshBatch_Alloc: extern fn () -> *mut GGD_MeshBatch,
	// MeshBatch_Free: extern fn (*mut GGD_MeshBatch),
	// MeshBatch_SetCacheData: None,
	// MeshBatch_GetCacheData: None,

	// MeshInstance_Alloc: extern fn (*mut GGD_MeshBatch) -> *mut GGD_MeshInstance,
	// MeshInstance_Free: extern fn (*mut GGD_MeshInstance),
	// MeshInstance_SetCacheData: None,
	// MeshInstance_GetCacheData: None,
	// MeshInstance_SetMeshData: extern fn (*mut GGD_MeshInstance, mesh: *mut GGD_MeshData, index: u32),
	// MeshInstance_SetImageData: extern fn (*mut GGD_MeshInstance, image: *mut GGD_ImageData, layer: GGMaterialLayer),
	// MeshInstance_SetAnimation: extern fn (*mut GGD_MeshInstance, firstIndex: u32, lastIndex: u32, frameRate: f32),
	// MeshInstance_SetTransform: extern fn (*mut GGD_MeshInstance, pose: *mut GGTransform),
	// MeshInstance_SetBoneTransform: extern fn (*mut GGD_MeshInstance, bone: u32, pose: *mut GGTransform),

	// Camera_Alloc: extern fn () -> *mut GGD_Camera,
	// Camera_Free: extern fn (*mut GGD_Camera),
	// Camera_SetPerspective: extern fn (*mut GGD_Camera, aspect: f32, fovx: f32, zNear: f32, zFar: f32),
	// Camera_SetOrthographic: extern fn (*mut GGD_Camera, w: f32, h: f32, zNear: f32, zFar: f32),
	// Camera_SetParabolic: extern fn (*mut GGD_Camera, scale: f32),
	// Camera_SetMeshBatch: extern fn (*mut GGD_Camera, *mut GGD_MeshBatch),
	// Camera_SetTransform: extern fn (*mut GGD_Camera, *mut GGTransform),
	// Camera_Draw: extern fn (*mut GGD_Camera, output: *mut GGD_ImageData),
};

#[allow(non_snake_case)]
#[repr(C)]
pub struct GGD_RenderEngine {
	Name: *const c_char,
	Priority: u64,
	Validate: Option<extern fn () -> i32>,
	Shutdown: Option<extern fn (*mut GGD_RenderEngine) -> i32>,

	Window_Alloc: unsafe extern fn (info: *mut GGD_WindowInfo) -> *mut GGD_Window,
	Window_Free: unsafe extern fn (*mut GGD_Window),
	Window_IsValid: extern fn (*mut GGD_Window) -> i32,
	Window_Resize: extern fn (*mut GGD_Window, w: u32, h: u32),
	// Window_Draw: extern fn (*mut GGD_Window, *mut GGD_ImageData),

	// MeshData_Alloc: extern fn () -> *mut GGD_MeshData,
	// MeshData_Free: extern fn (*mut GGD_MeshData),
	// MeshData_Prepare: extern fn (*mut GGD_MeshData),
	// MeshData_SetCacheData: Option<extern fn (*mut GGD_MeshData, buffer: *const c_void, size: u32) -> i32>,
	// MeshData_GetCacheData: Option<extern fn (*mut GGD_MeshData, buffer: *mut c_void, size: *mut u32) -> i32>,
	// MeshData_SetDistanceData: Option<extern fn (*mut GGD_MeshData, buffer: *const c_void, x: u32, y: u32, z: u32, format: GGDistanceFormat)>,
	// MeshData_GetDistanceData: Option<extern fn (*mut GGD_MeshData, buffer: *mut c_void, x: u32, y: u32, z: u32, format: *mut GGDistanceFormat)>,
	// MeshData_SetVertexData: extern fn (*mut GGD_MeshData, buffer: *const c_void, count: u32, format: GGVertexFormat),
	// MeshData_GetVertexData: extern fn (*mut GGD_MeshData, buffer: *mut c_void, count: *mut u32, format: *mut GGVertexFormat),
	// MeshData_SetIndexData: extern fn (*mut GGD_MeshData, buffer: *const c_void, count: u32, format: GGIndexFormat),
	// MeshData_GetIndexData: extern fn (*mut GGD_MeshData, buffer: *mut c_void, count: *mut u32, format: *mut GGIndexFormat),
	// MeshData_UseIndexData: Option<extern fn (*mut GGD_MeshData, src: *mut GGD_MeshData)>,

	// ImageData_Alloc: extern fn () ->  *mut GGD_ImageData,
	// ImageData_Free: extern fn (*mut GGD_ImageData),
	// ImageData_Prepare: extern fn (*mut GGD_ImageData),
	// ImageData_SetCacheData: Option<extern fn (image: *mut GGD_ImageData, buffer: *const c_void, size: u32) -> i32>,
	// ImageData_GetCacheData: Option<extern fn (image: *mut GGD_ImageData, buffer: *mut c_void, size: *mut u32) -> i32>,
	// ImageData_SetPixelData: extern fn (image: *mut GGD_ImageData, buffer: *const c_void, x: u32, y: u32, z: u32, format: GGPixelFormat),
	// ImageData_GetPixelData: extern fn (image: *mut GGD_ImageData, buffer: *mut c_void, x: *mut u32, y: *mut u32, z: *mut u32, format: *mut GGPixelFormat),
	// ImageData_Blur: extern fn (dst: *mut GGD_ImageData, src: *mut GGD_ImageData, radius: f32),

	// MeshBatch_Alloc: extern fn () -> *mut GGD_MeshBatch,
	// MeshBatch_Free: extern fn (*mut GGD_MeshBatch),
	// MeshBatch_SetCacheData: Option<extern fn (*mut GGD_MeshBatch, buffer: *const c_void, size: u32) -> i32>,
	// MeshBatch_GetCacheData: Option<extern fn (*mut GGD_MeshBatch, buffer: *mut c_void, size: *mut u32) -> i32>,

	// MeshInstance_Alloc: extern fn (*mut GGD_MeshBatch) -> *mut GGD_MeshInstance,
	// MeshInstance_Free: extern fn (*mut GGD_MeshInstance),
	// MeshInstance_SetCacheData: Option<extern fn (*mut GGD_MeshInstance, buffer: *const c_void, size: u32) -> i32>,
	// MeshInstance_GetCacheData: Option<extern fn (*mut GGD_MeshInstance, buffer: *mut c_void, size: *mut u32) -> i32>,
	// MeshInstance_SetMeshData: extern fn (*mut GGD_MeshInstance, mesh: *mut GGD_MeshData, index: u32),
	// MeshInstance_SetImageData: extern fn (*mut GGD_MeshInstance, image: *mut GGD_ImageData, layer: GGMaterialLayer),
	// MeshInstance_SetAnimation: extern fn (*mut GGD_MeshInstance, firstIndex: u32, lastIndex: u32, frameRate: f32),
	// MeshInstance_SetTransform: extern fn (*mut GGD_MeshInstance, pose: *mut GGTransform),
	// MeshInstance_SetBoneTransform: extern fn (*mut GGD_MeshInstance, bone: u32, pose: *mut GGTransform),

	// Camera_Alloc: extern fn () -> *mut GGD_Camera,
	// Camera_Free: extern fn (*mut GGD_Camera),
	// Camera_SetPerspective: extern fn (*mut GGD_Camera, aspect: f32, fovx: f32, zNear: f32, zFar: f32),
	// Camera_SetOrthographic: extern fn (*mut GGD_Camera, w: f32, h: f32, zNear: f32, zFar: f32),
	// Camera_SetParabolic: extern fn (*mut GGD_Camera, scale: f32),
	// Camera_SetMeshBatch: extern fn (*mut GGD_Camera, *mut GGD_MeshBatch),
	// Camera_SetTransform: extern fn (*mut GGD_Camera, *mut GGTransform),
	// Camera_Draw: extern fn (*mut GGD_Camera, output: *mut GGD_ImageData),
}
