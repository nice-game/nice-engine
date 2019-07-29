use crate::game_graph::*;
use libc::c_void;
use nice_engine::{camera::Camera, mesh_data::MeshData, surface::Surface};
use std::os::raw::c_char;

#[allow(non_camel_case_types)]
pub type GGD_Camera = Camera;

#[allow(non_snake_case)]
#[repr(C)]
pub struct GGD_DriverContext {
	pub APIVersion: u64,
	pub GameVersion: u64,
	pub GameName: *const c_char,
	pub RegisterRenderEngine: extern fn(*mut GGD_RenderEngine),
	pub RegisterPhysicsEngine: extern fn(*mut GGD_PhysicsEngine),
}

#[allow(non_camel_case_types)]
pub struct GGD_FontData {}

#[allow(non_camel_case_types)]
pub struct GGD_ImageData {}

#[allow(non_camel_case_types)]
pub struct GGD_MeshBatch {}

#[allow(non_camel_case_types)]
pub type GGD_MeshData = MeshData;

#[allow(non_camel_case_types)]
pub struct GGD_MeshInstance {}

#[allow(non_snake_case)]
#[repr(C)]
pub struct GGD_PhysicsEngine {
	// TODO: implement physics engine
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct GGD_RenderEngine {
	pub Name: *const c_char,
	pub Priority: u64,
	pub Validate: Option<extern fn() -> i32>,
	pub Shutdown: Option<extern fn(*mut GGD_RenderEngine) -> i32>,

	pub Window_Alloc: unsafe extern fn(info: *mut GGD_WindowInfo) -> *mut GGD_Window,
	pub Window_Free: unsafe extern fn(*mut GGD_Window),
	pub Window_IsValid: extern fn(*mut GGD_Window) -> i32,
	pub Window_Resize: unsafe extern fn(*mut GGD_Window, w: u32, h: u32),
	pub Window_Draw: extern fn(*mut GGD_Window, src: *mut GGD_Camera, overlay: *mut GGD_ImageData),

	pub MeshData_Alloc: unsafe extern fn() -> *mut GGD_MeshData,
	pub MeshData_Free: unsafe extern fn(*mut GGD_MeshData),
	pub MeshData_Prepare: Option<extern fn(*mut GGD_MeshData)>,
	pub MeshData_SetCacheData: Option<extern fn(*mut GGD_MeshData, buffer: *const c_void, size: u32) -> i32>,
	pub MeshData_GetCacheData: Option<extern fn(*mut GGD_MeshData, buffer: *mut c_void, size: *mut u32) -> i32>,
	pub MeshData_SetDistanceData:
		Option<extern fn(*mut GGD_MeshData, buffer: *const c_void, x: u32, y: u32, z: u32, format: GGDistanceFormat)>,
	pub MeshData_GetDistanceData: Option<
		extern fn(*mut GGD_MeshData, buffer: *mut c_void, x: u32, y: u32, z: u32, format: *mut GGDistanceFormat),
	>,
	pub MeshData_SetVertexData:
		unsafe extern fn(*mut GGD_MeshData, buffer: *const c_void, count: u32, format: GGVertexFormat),
	pub MeshData_GetVertexData:
		extern fn(*mut GGD_MeshData, buffer: *mut c_void, count: *mut u32, format: *mut GGVertexFormat),
	pub MeshData_SetIndexData:
		unsafe extern fn(*mut GGD_MeshData, buffer: *const c_void, count: u32, format: GGIndexFormat),
	pub MeshData_GetIndexData:
		extern fn(*mut GGD_MeshData, buffer: *mut c_void, count: *mut u32, format: *mut GGIndexFormat),
	pub MeshData_UseIndexData: Option<extern fn(*mut GGD_MeshData, src: *mut GGD_MeshData)>,

	pub ImageData_Alloc: extern fn(usage: GGImageUsage, x: u32, y: u32, format: GGPixelFormat) -> *mut GGD_ImageData,
	pub ImageData_Free: unsafe extern fn(*mut GGD_ImageData),
	pub ImageData_Prepare: Option<extern fn(*mut GGD_ImageData)>,
	pub ImageData_SetCacheData: Option<extern fn(image: *mut GGD_ImageData, buffer: *const c_void, size: u32) -> i32>,
	pub ImageData_GetCacheData:
		Option<extern fn(image: *mut GGD_ImageData, buffer: *mut c_void, size: *mut u32) -> i32>,
	pub ImageData_SetPixelData:
		extern fn(image: *mut GGD_ImageData, buffer: *const c_void, x: u32, y: u32, format: GGPixelFormat),
	pub ImageData_GetPixelData:
		extern fn(image: *mut GGD_ImageData, buffer: *mut c_void, x: *mut u32, y: *mut u32, format: *mut GGPixelFormat),
	pub ImageData_DrawCamera: extern fn(dst: *mut GGD_ImageData, src: *mut GGD_Camera),
	pub ImageData_DrawImage:
		extern fn(dst: *mut GGD_ImageData, src: *mut GGD_ImageData, x: f32, y: f32, w: f32, h: f32),
	pub ImageData_DrawText:
		extern fn(dst: *mut GGD_ImageData, src: *mut GGD_FontData, x: f32, y: f32, text: *const char),

	pub FontData_Alloc: extern fn() -> *mut GGD_FontData,
	pub FontData_Free: unsafe extern fn(*mut GGD_FontData),
	pub FontData_Prepare: Option<extern fn(*mut GGD_FontData)>,
	pub FontData_SetCacheData: Option<extern fn(image: *mut GGD_FontData, buffer: *const c_void, size: u32) -> i32>,
	pub FontData_GetCacheData: Option<extern fn(image: *mut GGD_FontData, buffer: *mut c_void, size: *mut u32) -> i32>,
	pub FontData_SetGlyph: extern fn(image: *mut GGD_FontData, codepoint: u32, img: *mut GGD_ImageData),
	pub FontData_SetTTFData: Option<
		extern fn(image: *mut GGD_FontData, buffer: *const c_void, bytes: u32, fontsize: u32, r: f32, g: f32, b: f32),
	>,

	pub MeshBatch_Alloc: extern fn() -> *mut GGD_MeshBatch,
	pub MeshBatch_Free: unsafe extern fn(*mut GGD_MeshBatch),
	pub MeshBatch_Prepare: Option<extern fn(*mut GGD_MeshBatch)>,
	pub MeshBatch_SetCacheData: Option<extern fn(*mut GGD_MeshBatch, buffer: *const c_void, size: u32) -> i32>,
	pub MeshBatch_GetCacheData: Option<extern fn(*mut GGD_MeshBatch, buffer: *mut c_void, size: *mut u32) -> i32>,

	pub MeshInstance_Alloc: extern fn(*mut GGD_MeshBatch) -> *mut GGD_MeshInstance,
	pub MeshInstance_Free: unsafe extern fn(*mut GGD_MeshInstance),
	pub MeshInstance_SetCacheData: Option<extern fn(*mut GGD_MeshInstance, buffer: *const c_void, size: u32) -> i32>,
	pub MeshInstance_GetCacheData: Option<extern fn(*mut GGD_MeshInstance, buffer: *mut c_void, size: *mut u32) -> i32>,
	pub MeshInstance_SetMeshData: extern fn(*mut GGD_MeshInstance, mesh: *mut GGD_MeshData, index: u32),
	pub MeshInstance_SetImageData: extern fn(*mut GGD_MeshInstance, image: *mut GGD_ImageData, layer: GGMaterialLayer),
	pub MeshInstance_SetAnimation: extern fn(*mut GGD_MeshInstance, firstIndex: u32, lastIndex: u32, frameRate: f32),
	pub MeshInstance_SetTransform: extern fn(*mut GGD_MeshInstance, pose: *mut GGTransform),
	pub MeshInstance_SetBoneTransform: extern fn(*mut GGD_MeshInstance, bone: u32, pose: *mut GGTransform),

	pub Camera_Alloc: extern fn() -> *mut GGD_Camera,
	pub Camera_Free: unsafe extern fn(*mut GGD_Camera),
	pub Camera_SetPerspective: unsafe extern fn(*mut GGD_Camera, aspect: f32, fovx: f32, zNear: f32, zFar: f32),
	pub Camera_SetOrthographic: Option<extern fn(*mut GGD_Camera, w: f32, h: f32, zNear: f32, zFar: f32)>,
	pub Camera_SetParabolic: Option<extern fn(*mut GGD_Camera, scale: f32)>,
	pub Camera_SetMeshBatch: extern fn(*mut GGD_Camera, *mut GGD_MeshBatch),
	pub Camera_SetTransform: extern fn(*mut GGD_Camera, *mut GGTransform),
}

#[allow(non_camel_case_types)]
pub type GGD_Window = Surface;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GGD_WindowInfo {
	/// GGPlatform
	pub platform: u64,
	pub sdlsurface: *mut c_void,
}

#[cfg(unix)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GGD_WindowInfo_WAYLAND {
	pub info: GGD_WindowInfo,
	pub display: *mut wl_display,
	pub surface: *mut wl_surface,
	pub wmsurface: *mut wl_shell_surface,
}

#[cfg(windows)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GGD_WindowInfo_WIN32 {
	pub info: GGD_WindowInfo,
	pub hinstance: HINSTANCE,
	pub hwnd: HWND,
	pub hdc: HDC,
}

#[cfg(unix)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GGD_WindowInfo_X11 {
	pub info: GGD_WindowInfo,
	pub display: *mut Display,
	pub surface: Surface,
}

type HINSTANCE = *mut c_void;
type HWND = *mut c_void;
type HDC = *mut c_void;
