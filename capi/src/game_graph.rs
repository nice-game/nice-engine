use bitflags::bitflags;
use libc::c_void;
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

	PFMT_RGBA8,
	PFMT_RGBA8_SRGB,
	PFMT_RGBE8_SRGB,
	PFMT_A2_BGR10,
	PFMT_A2_BGR10_QRGB,
	PFMT_RGB9_E5_QRGB,
	PFMT_RGBA16F,
	PFMT_RG16F,
	PFMT_R16F,
	PFMT_RGBA32F,
	PFMT_RG32F,
	PFMT_R32F,

	PFMT_BC1,
	PFMT_BC1_SRGB,
	PFMT_BC2,
	PFMT_BC2_SRGB,
	PFMT_BC3,
	PFMT_BC3_SRGB,
	PFMT_BC4,
	PFMT_BC4_SIGNED,
	PFMT_BC5,
	PFMT_BC5_SIGNED,
	PFMT_BC6H,
	PFMT_BC6H_SIGNED,
	PFMT_BC7,
	PFMT_BC7_SRGB,

	PFMT_INVALID,
}
impl Into<Format> for GGPixelFormat {
	fn into(self) -> Format {
		match self {
			Self::PFMT_RGBA8 => R8G8B8A8Unorm,
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

// #[allow(non_camel_case_types)]
// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// #[repr(C)]
bitflags! {
	pub struct GGImageUsage: i32 {
		const IMG_USAGE_STATIC = 0;
		const IMG_USAGE_TARGET = 1;
		const IMG_USAGE_OVERLAY = 2;
		const IMG_USAGE_GLYPH = 4;
		const IMG_USAGE_SKYBOX = 8;
		const IMG_USAGE_EMISSIVE = 16;
	}
}

// #[allow(non_camel_case_types)]
// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// #[repr(C)]
// pub enum GGMaterialLayer {
// 	MATL_SURFACE_COLOR,
// 	MATL_SURFACE_FINISH,
// 	MATL_AMBIENT_OCCLUSION,
// 	MATL_LIGHTMAP0,
// 	MATL_LIGHTMAP1,
// 	MATL_LIGHTMAP2,
// 	MATL_LIGHTMAP3,
// }

#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum GGD_BufferStatus {
	GGD_BUFFER_NOP,
	GGD_BUFFER_READ,
	GGD_BUFFER_WRITE,
	GGD_BUFFER_CLOSED,
}

/// Must be !Sync because `size` may be modified by `read` and `status`, even though they accept const pointers.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GGD_BufferInfo {
	/// May be 0 if status is `GGD_BUFFER_CLOSED`
	///
	/// `size` will be modified by `resize`, and if `size` is 0, it may be modified by `read`, `write`, or `status`
	pub size: u64,

	/// Will block if status is `GGD_BUFFER_CLOSED`
	///
	/// # Returns
	/// A read-only buffer that will be valid until the next call to `read`, `write`, `resize`, or `status` with the
	/// same GGD_BufferInfo
	pub read: unsafe extern fn(*const GGD_BufferInfo, offset: u64, bytes: u64) -> *const c_void,

	/// Will block if status is not `GGD_BUFFER_WRITE`
	///
	/// # Returns
	/// A write-only buffer that will be valid until the next call to `read`, `write`, `resize`, or `status` with the
	/// same GGD_BufferInfo
	pub write: Option<unsafe extern fn(*mut GGD_BufferInfo, offset: u64, bytes: u64) -> *mut c_void>,

	/// Will block if status is not `GGD_BUFFER_WRITE`
	pub resize: Option<unsafe extern fn(*mut GGD_BufferInfo, bytes: u64)>,

	/// If `command` is not `GGD_BUFFER_NOP`, it is interpreted as a request to change state, but `status` will not
	/// block until the state is changed
	///
	/// # Returns
	/// The status at the time the function is called
	pub status: Option<unsafe extern fn(*const GGD_BufferInfo, command: i32) -> i32>,
}

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
