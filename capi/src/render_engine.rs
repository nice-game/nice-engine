mod camera;
mod font_data;
mod image_data;
mod mesh_batch;
mod mesh_data;
mod mesh_instance;
mod window;

use self::{camera::*, font_data::*, image_data::*, mesh_batch::*, mesh_data::*, mesh_instance::*, window::*};
use crate::game_graph_driver::GGD_RenderEngine;

pub const RENDER_ENGINE: GGD_RenderEngine = GGD_RenderEngine {
	Name: "nIce Engine".as_ptr() as _,
	Priority: 10,
	GraphicsAPI: 100,
	Validate: None,
	Shutdown: None,

	Window_Alloc,
	Window_Free,
	Window_IsValid,
	Window_Resize,
	Window_SetCamera,
	Window_SetOverlay,
	Window_Draw,

	MeshData_Alloc_Polygon,
	MeshData_Alloc_Polygon_Variant: None,
	MeshData_Alloc_Field: None,
	MeshData_Free,

	ImageData_Alloc,
	ImageData_Free,
	ImageData_SetPixelData,
	ImageData_GetPixelData,
	ImageData_DrawCamera,
	ImageData_DrawImage,
	ImageData_DrawText,

	FontData_Alloc,
	FontData_Free,
	FontData_SetGlyph,

	MeshGroup_Alloc,
	MeshGroup_Free,

	MeshInstance_Alloc,
	MeshInstance_Free,
	MeshInstance_SetCacheData: None,
	MeshInstance_GetCacheData: None,
	MeshInstance_SetMeshData,
	MeshInstance_SetImageData,
	MeshInstance_SetAnimation,
	MeshInstance_SetTransform,
	MeshInstance_SetBoneTransform,

	Camera_Alloc,
	Camera_Free,
	Camera_SetPerspective,
	Camera_SetOrthographic: None,
	Camera_SetParabolic: None,
	Camera_SetMeshGroup,
	Camera_SetTransform,
};
