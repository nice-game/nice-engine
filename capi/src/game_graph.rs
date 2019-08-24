use std::mem;
use vulkano::format::Format::{self, *};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct GGPosition {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub w: f32,
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
pub enum GGPixelFormat {
	PFMT_UNDEFINED = 0,

	PFMT_RGBA8_UNORM,
	PFMT_RGBA8_SRGB,
	PFMT_A2_BGR10_UNORM,
	PFMT_A2_BGR10_QNORM,
	PFMT_RGB9_E5,
	PFMT_RGBA16F,
	PFMT_RG16F,
	PFMT_R16F,
	PFMT_RGBA32F,
	PFMT_RG32F,
	PFMT_R32F,

	PFMT_BC1_UNORM,
	PFMT_BC1_SRGB,
	PFMT_BC2_UNORM,
	PFMT_BC2_SRGB,
	PFMT_BC3_UNORM,
	PFMT_BC3_SRGB,
	PFMT_BC4_UNORM,
	PFMT_BC4_SNORM,
	PFMT_BC5_UNORM,
	PFMT_BC5_SNORM,
	PFMT_BC6H_UFLOAT,
	PFMT_BC6H_SFLOAT,
	PFMT_BC7_UNORM,
	PFMT_BC7_SRGB,

	PFMT_INVALID,
}
impl Into<Format> for GGPixelFormat {
	fn into(self) -> Format {
		match self {
			Self::PFMT_RGBA8_UNORM => R8G8B8A8Unorm,
			Self::PFMT_RGBA8_SRGB => R8G8B8A8Srgb,
			Self::PFMT_RGBA32F => R32G32B32A32Sfloat,
			Self::PFMT_RGBA16F => R16G16B16A16Sfloat,
			_ => panic!("{:?} not supported", self),
		}
	}
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum GGDriverStatus {
	GGD_STATUS_DRIVER_INVALID = 0,
	GGD_STATUS_DRIVER_READY = 1,
	GGD_STATUS_DRIVER_ERROR = 2,
	GGD_STATUS_VERSION_INVALID = 3,
	GGD_STATUS_SIGNATURE_INVALID = 4,
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

// #[allow(non_camel_case_types)]
// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// #[repr(C)]
// pub enum GGMaterialLayer {
// 	MATL_SURFACE_COLOR,
// 	MATL_SURFACE_FINISH,
// 	MATL_AMBIENT_OCCLUSION,
// 	MATL_LIGHTMAP_FLAT,
// 	MATL_LIGHTMAP_ANGLE0,
// 	MATL_LIGHTMAP_ANGLE1,
// 	MATL_LIGHTMAP_ANGLE2,
// }

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum GGTextOrigin {
	TEXT_BASELINE,
	TEXT_TOP_LEFT,
	TEXT_TOP,
	TEXT_TOP_RIGHT,
	TEXT_LEFT,
	TEXT_CENTER,
	TEXT_RIGHT,
	TEXT_BOTTOM_LEFT,
	TEXT_BOTTOM,
	TEXT_BOTTOM_RIGHT,
}
