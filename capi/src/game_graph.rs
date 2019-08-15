use std::mem;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct GGPosition {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct GGRotation {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub w: f32,
}

#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct GGTransform {
	pub Position: GGPosition,
	pub Rotation: GGRotation,
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum GGPlatform {
	UNDEFINED = 0,
	WIN32 = 1,
	X11 = 2,
	WAYLAND = 3,
	OSX = 4,
}
impl GGPlatform {
	pub unsafe fn from_u64_unchecked(val: u64) -> Self {
		mem::transmute(val as u32)
	}
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum GGVertexFormat {
	VFMT_UNDEFINED = 0,
	VFMT_PNTL_32F,
	VFMT_PNTLB3_32F,
	VFMT_PNTLB7_32F,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum GGIndexFormat {
	IFMT_UNDEFINED = 0,
	IFMT_SOUP_16U,
	IFMT_SOUP_32U,
	IFMT_STRIP_16U,
	IFMT_STRIP_32U,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum GGDistanceFormat {
	DFMT_UNDEFINED = 0,
	DFMT_EXACT_DISTANCE_8,
	DFMT_EXACT_DISTANCE_32F,
	DFMT_BOUND_DISTANCE_8,
	DFMT_BOUND_DISTANCE_32F,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum GGMaterialLayer {
	MATL_ALBEDO = 0,
	MATL_NORMALS,
	MATL_TANGENTS,
	MATL_QTANGENTS,
	MATL_DISPLACEMENT,
	MATL_SMOOTHNESS,
	MATL_ROUGHNESS,
	MATL_METALNESS,
	MATL_EMISSIVE,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum GGPixelFormat {
	PFMT_UNDEFINED = 0,
	SRGB_DXT1,
	SRGB_8,

	RGB_DXT1,
	RGB_BC6H,
	RGB_BC6H_SIGNED,
	RGB_8,
	RGB_16F,
	RGB_32F,

	SRGBA_DXT1,
	SRGBA_DXT3,
	SRGBA_DXT5,
	SRGBA_BC7,
	SRGBA_8,

	RGBA_DXT1,
	RGBA_DXT3,
	RGBA_DXT5,
	RGBA_BC7,
	RGBA_8,
	RGBA_10_2,
	RGBA_16F,
	RGBA_32F,

	IVEC2_8,
	IVEC3_16,
	IVEC4_10_2,
	DEPTH_STENCIL_24_8,
	DEPTH_32,
	FVEC2,
	FVEC2_16,
	FVEC2_U8,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum GGDriverStatus {
	DRIVER_INVALID = 0,
	DRIVER_READY,
	DRIVER_ERROR,
	VERSION_INVALID,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum GGImageUsage {
	IMG_USAGE_UNDEFINED = 0,
	IMG_USAGE_STATIC,
	IMG_USAGE_TARGET,
	IMG_USAGE_OVERLAY,
	IMG_USAGE_GLYPH,
}
