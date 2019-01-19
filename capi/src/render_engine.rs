mod camera;
mod font_data;
mod image_data;
mod mesh_batch;
mod mesh_instance;
mod mesh_data;
mod window;

use crate::game_graph::*;
use self::{ camera::*, font_data::*, image_data::*, mesh_batch::*, mesh_data::*, mesh_instance::*, window::* };
use libc::c_void;
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
	Window_Draw: Window_Draw,

	MeshData_Alloc: MeshData_Alloc,
	MeshData_Free: MeshData_Free,
	MeshData_Prepare: None,
	MeshData_SetCacheData: None,
	MeshData_GetCacheData: None,
	MeshData_SetDistanceData: None,
	MeshData_GetDistanceData: None,
	MeshData_SetVertexData: MeshData_SetVertexData,
	MeshData_GetVertexData: MeshData_GetVertexData,
	MeshData_SetIndexData: MeshData_SetIndexData,
	MeshData_GetIndexData: MeshData_GetIndexData,
	MeshData_UseIndexData: None,

	ImageData_Alloc: ImageData_Alloc,
	ImageData_Free: ImageData_Free,
	ImageData_Prepare: None,
	ImageData_SetCacheData: None,
	ImageData_GetCacheData: None,
	ImageData_SetPixelData: ImageData_SetPixelData,
	ImageData_GetPixelData: ImageData_GetPixelData,
	ImageData_DrawCamera: ImageData_DrawCamera,
	ImageData_DrawImage: ImageData_DrawImage,
	ImageData_DrawText: ImageData_DrawText,

	FontData_Alloc: FontData_Alloc,
	FontData_Free: FontData_Free,
	FontData_Prepare: None,
	FontData_SetCacheData: None,
	FontData_GetCacheData: None,
	FontData_SetGlyph: FontData_SetGlyph,
	FontData_SetTTFData: None,

	MeshBatch_Alloc: MeshBatch_Alloc,
	MeshBatch_Free: MeshBatch_Free,
	MeshBatch_Prepare: None,
	MeshBatch_SetCacheData: None,
	MeshBatch_GetCacheData: None,

	MeshInstance_Alloc: MeshInstance_Alloc,
	MeshInstance_Free: MeshInstance_Free,
	MeshInstance_SetCacheData: None,
	MeshInstance_GetCacheData: None,
	MeshInstance_SetMeshData: MeshInstance_SetMeshData,
	MeshInstance_SetImageData: MeshInstance_SetImageData,
	MeshInstance_SetAnimation: MeshInstance_SetAnimation,
	MeshInstance_SetTransform: MeshInstance_SetTransform,
	MeshInstance_SetBoneTransform: MeshInstance_SetBoneTransform,

	Camera_Alloc: Camera_Alloc,
	Camera_Free: Camera_Free,
	Camera_SetPerspective: Camera_SetPerspective,
	Camera_SetOrthographic: None,
	Camera_SetParabolic: None,
	Camera_SetMeshBatch: Camera_SetMeshBatch,
	Camera_SetTransform: Camera_SetTransform,
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
	Window_Resize: unsafe extern fn (*mut GGD_Window, w: u32, h: u32),
	Window_Draw: extern fn (*mut GGD_Window, src: *mut GGD_Camera, overlay: *mut GGD_ImageData),

	MeshData_Alloc: extern fn () -> *mut GGD_MeshData,
	MeshData_Free: unsafe extern fn (*mut GGD_MeshData),
	MeshData_Prepare: Option<extern fn (*mut GGD_MeshData)>,
	MeshData_SetCacheData: Option<extern fn (*mut GGD_MeshData, buffer: *const c_void, size: u32) -> i32>,
	MeshData_GetCacheData: Option<extern fn (*mut GGD_MeshData, buffer: *mut c_void, size: *mut u32) -> i32>,
	MeshData_SetDistanceData: Option<extern fn (*mut GGD_MeshData, buffer: *const c_void, x: u32, y: u32, z: u32, format: GGDistanceFormat)>,
	MeshData_GetDistanceData: Option<extern fn (*mut GGD_MeshData, buffer: *mut c_void, x: u32, y: u32, z: u32, format: *mut GGDistanceFormat)>,
	MeshData_SetVertexData: extern fn (*mut GGD_MeshData, buffer: *const c_void, count: u32, format: GGVertexFormat),
	MeshData_GetVertexData: extern fn (*mut GGD_MeshData, buffer: *mut c_void, count: *mut u32, format: *mut GGVertexFormat),
	MeshData_SetIndexData: extern fn (*mut GGD_MeshData, buffer: *const c_void, count: u32, format: GGIndexFormat),
	MeshData_GetIndexData: extern fn (*mut GGD_MeshData, buffer: *mut c_void, count: *mut u32, format: *mut GGIndexFormat),
	MeshData_UseIndexData: Option<extern fn (*mut GGD_MeshData, src: *mut GGD_MeshData)>,

	ImageData_Alloc: extern fn (usage: GGImageUsage, x: u32, y: u32, format: GGPixelFormat) -> *mut GGD_ImageData,
	ImageData_Free: unsafe extern fn (*mut GGD_ImageData),
	ImageData_Prepare: Option<extern fn (*mut GGD_ImageData)>,
	ImageData_SetCacheData: Option<extern fn (image: *mut GGD_ImageData, buffer: *const c_void, size: u32) -> i32>,
	ImageData_GetCacheData: Option<extern fn (image: *mut GGD_ImageData, buffer: *mut c_void, size: *mut u32) -> i32>,
	ImageData_SetPixelData: extern fn (image: *mut GGD_ImageData, buffer: *const c_void, x: u32, y: u32, format: GGPixelFormat),
	ImageData_GetPixelData: extern fn (image: *mut GGD_ImageData, buffer: *mut c_void, x: *mut u32, y: *mut u32, format: *mut GGPixelFormat),
	ImageData_DrawCamera: extern fn (dst: *mut GGD_ImageData, src: *mut GGD_Camera),
	ImageData_DrawImage: extern fn (dst: *mut GGD_ImageData, src: *mut GGD_ImageData, x: f32, y: f32, w: f32, h: f32),
	ImageData_DrawText: extern fn (dst: *mut GGD_ImageData, src: *mut GGD_FontData, x: f32, y: f32, text: *const char),

	FontData_Alloc: extern fn () -> *mut GGD_FontData,
	FontData_Free: unsafe extern fn (*mut GGD_FontData),
	FontData_Prepare: Option<extern fn (*mut GGD_FontData)>,
	FontData_SetCacheData: Option<extern fn (image: *mut GGD_FontData, buffer: *const c_void, size: u32) -> i32>,
	FontData_GetCacheData: Option<extern fn (image: *mut GGD_FontData, buffer: *mut c_void, size: *mut u32) -> i32>,
	FontData_SetGlyph: extern fn (image: *mut GGD_FontData, codepoint: u32, img: *mut GGD_ImageData),
	FontData_SetTTFData: Option<extern fn (image: *mut GGD_FontData, buffer: *const c_void, bytes: u32, fontsize: u32, r: f32, g: f32, b: f32)>,

	MeshBatch_Alloc: extern fn () -> *mut GGD_MeshBatch,
	MeshBatch_Free: unsafe extern fn (*mut GGD_MeshBatch),
	MeshBatch_Prepare: Option<extern fn (*mut GGD_MeshBatch)>,
	MeshBatch_SetCacheData: Option<extern fn (*mut GGD_MeshBatch, buffer: *const c_void, size: u32) -> i32>,
	MeshBatch_GetCacheData: Option<extern fn (*mut GGD_MeshBatch, buffer: *mut c_void, size: *mut u32) -> i32>,

	MeshInstance_Alloc: extern fn (*mut GGD_MeshBatch) -> *mut GGD_MeshInstance,
	MeshInstance_Free: unsafe extern fn (*mut GGD_MeshInstance),
	MeshInstance_SetCacheData: Option<extern fn (*mut GGD_MeshInstance, buffer: *const c_void, size: u32) -> i32>,
	MeshInstance_GetCacheData: Option<extern fn (*mut GGD_MeshInstance, buffer: *mut c_void, size: *mut u32) -> i32>,
	MeshInstance_SetMeshData: extern fn (*mut GGD_MeshInstance, mesh: *mut GGD_MeshData, index: u32),
	MeshInstance_SetImageData: extern fn (*mut GGD_MeshInstance, image: *mut GGD_ImageData, layer: GGMaterialLayer),
	MeshInstance_SetAnimation: extern fn (*mut GGD_MeshInstance, firstIndex: u32, lastIndex: u32, frameRate: f32),
	MeshInstance_SetTransform: extern fn (*mut GGD_MeshInstance, pose: *mut GGTransform),
	MeshInstance_SetBoneTransform: extern fn (*mut GGD_MeshInstance, bone: u32, pose: *mut GGTransform),

	Camera_Alloc: extern fn () -> *mut GGD_Camera,
	Camera_Free: unsafe extern fn (*mut GGD_Camera),
	Camera_SetPerspective: extern fn (*mut GGD_Camera, aspect: f32, fovx: f32, zNear: f32, zFar: f32),
	Camera_SetOrthographic: Option<extern fn (*mut GGD_Camera, w: f32, h: f32, zNear: f32, zFar: f32)>,
	Camera_SetParabolic: Option<extern fn (*mut GGD_Camera, scale: f32)>,
	Camera_SetMeshBatch: extern fn (*mut GGD_Camera, *mut GGD_MeshBatch),
	Camera_SetTransform: extern fn (*mut GGD_Camera, *mut GGTransform),
}
