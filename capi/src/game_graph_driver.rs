use nice_engine::mesh::Mesh;
use crate::game_graph::*;
use libc::c_void;
use nice_engine::{camera::Camera, mesh_data::MeshData, mesh_group::MeshGroup, surface::Surface as NiceSurface};
#[cfg(unix)]
use std::os::raw::c_ulong;
use std::{
	os::raw::c_char,
	sync::{Arc, Mutex},
};

#[allow(non_camel_case_types)]
pub type GGD_Camera = Arc<Mutex<Camera>>;

#[allow(non_snake_case)]
#[repr(C)]
pub struct GGD_DriverContext {
	pub APIVersion: u64,
	pub GameVersion: u64,
	pub GameName: *const c_char,
	pub RegisterRenderEngine: extern fn(*const GGD_RenderEngine),
	pub RegisterPhysicsEngine: extern fn(*const GGD_PhysicsEngine),
}

#[allow(non_camel_case_types)]
pub struct GGD_FontData {}

#[allow(non_camel_case_types)]
pub struct GGD_ImageData {}

#[allow(non_camel_case_types)]
pub type GGD_MeshGroup = Arc<MeshGroup>;

#[allow(non_camel_case_types)]
pub type GGD_MeshData = Arc<MeshData>;

#[allow(non_camel_case_types)]
pub type GGD_MeshInstance = Mesh;

#[allow(non_snake_case)]
#[repr(C)]
pub struct GGD_PhysicsEngine {
	// TODO: implement physics engine
}

#[allow(non_snake_case, non_camel_case_types)]
#[repr(C)]
pub struct GGD_RenderEngine {
	pub Name: *const c_char,
	pub Priority: u64,
	pub GraphicsAPI: u64,
	pub Validate: Option<extern fn() -> i32>,
	pub Shutdown: Option<extern fn(*mut GGD_RenderEngine) -> i32>,

	pub Window_Alloc: unsafe extern fn(info: *mut GGD_WindowInfo) -> *mut GGD_Window,
	pub Window_Free: unsafe extern fn(*mut GGD_Window),
	pub Window_IsValid: extern fn(*mut GGD_Window) -> i32,
	pub Window_Resize: unsafe extern fn(*mut GGD_Window, w: u32, h: u32),
	pub Window_SetCamera: unsafe extern fn(dst: *mut GGD_Window, camera: *mut GGD_Camera),
	pub Window_SetOverlay: extern fn(dst: *mut GGD_Window, overlay: *mut GGD_ImageData),
	pub Window_Draw: unsafe extern fn(*mut GGD_Window),

	pub MeshData_Alloc_Polygon: unsafe extern fn(
		vertexBuffer: *const c_void,
		vertexCount: u32,
		vertexFormat: GGVertexFormat,
		indexBuffer: *const c_void,
		indexCount: u32,
		indexFormat: GGIndexFormat,
	) -> *mut GGD_MeshData,
	pub MeshData_Alloc_Polygon_Variant: Option<
		unsafe extern fn(
			vertexBuffer: *const c_void,
			vertexCount: u32,
			vertexFormat: GGVertexFormat,
			indexBuffer: *mut GGD_MeshData,
		) -> *mut GGD_MeshData,
	>,
	pub MeshData_Alloc_Field:
		Option<extern fn(buffer: *const c_void, x: u32, y: u32, z: u32, format: GGDistanceFormat) -> *mut GGD_MeshData>,
	pub MeshData_Free: unsafe extern fn(*mut GGD_MeshData),

	pub ImageData_Alloc: extern fn(usage: GGImageUsage, x: u32, y: u32, format: GGPixelFormat) -> *mut GGD_ImageData,
	pub ImageData_Free: unsafe extern fn(*mut GGD_ImageData),
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
	pub FontData_SetGlyph: extern fn(image: *mut GGD_FontData, codepoint: u32, img: *mut GGD_ImageData),

	pub MeshGroup_Alloc: extern fn() -> *mut GGD_MeshGroup,
	pub MeshGroup_Free: unsafe extern fn(*mut GGD_MeshGroup),
	pub MeshGroup_SetSky: extern fn(*mut GGD_MeshGroup, *mut GGD_ImageData),

	pub MeshInstance_Alloc: unsafe extern fn(*mut GGD_MeshGroup) -> *mut GGD_MeshInstance,
	pub MeshInstance_Free: unsafe extern fn(*mut GGD_MeshInstance),
	pub MeshInstance_SetMeshData: unsafe extern fn(*mut GGD_MeshInstance, mesh: *mut GGD_MeshData, index: u32),
	pub MeshInstance_SetImageData: extern fn(*mut GGD_MeshInstance, image: *mut GGD_ImageData, layer: i32),
	pub MeshInstance_SetAnimation: extern fn(*mut GGD_MeshInstance, firstIndex: u32, lastIndex: u32, frameRate: f32),
	pub MeshInstance_SetTransform: unsafe extern fn(*mut GGD_MeshInstance, pose: *const GGTransform),
	pub MeshInstance_SetBoneTransform: extern fn(*mut GGD_MeshInstance, bone: u32, pose: *const GGTransform),

	pub Camera_Alloc: extern fn() -> *mut GGD_Camera,
	pub Camera_Free: unsafe extern fn(*mut GGD_Camera),
	pub Camera_SetPerspective: unsafe extern fn(*mut GGD_Camera, aspect: f32, fovy: f32, zNear: f32, zFar: f32),
	pub Camera_SetOrthographic: Option<extern fn(*mut GGD_Camera, w: f32, h: f32, zNear: f32, zFar: f32)>,
	pub Camera_SetParabolic: Option<extern fn(*mut GGD_Camera, scale: f32)>,
	pub Camera_SetMeshGroup: extern fn(*mut GGD_Camera, *mut GGD_MeshGroup),
	pub Camera_SetTransform: extern fn(*mut GGD_Camera, *const GGTransform),
}

#[allow(non_camel_case_types)]
pub type GGD_Window = NiceSurface;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GGD_WindowInfo {
	/// GGPlatform
	pub platform: u64,
	pub sdlwindow: *mut c_void,
	pub wxcanvas: *mut c_void,
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

#[cfg(windows)]
type HINSTANCE = *mut c_void;
#[cfg(windows)]
type HWND = *mut c_void;
#[cfg(windows)]
type HDC = *mut c_void;
#[cfg(unix)]
type wl_display = c_void;
#[cfg(unix)]
type wl_surface = c_void;
#[cfg(unix)]
type wl_shell_surface = c_void;
#[cfg(unix)]
type Display = c_void;
#[cfg(unix)]
type Surface = c_ulong;
