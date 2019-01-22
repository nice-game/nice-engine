mod camera;
mod font_data;
mod image_data;
mod mesh_batch;
mod mesh_instance;
mod mesh_data;
mod window;

use crate::{ game_graph_driver::GGD_RenderEngine };
use self::{ camera::*, font_data::*, image_data::*, mesh_batch::*, mesh_data::*, mesh_instance::*, window::* };

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
