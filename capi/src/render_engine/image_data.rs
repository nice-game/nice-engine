use crate::{
	ctx,
	game_graph::{
		GGImageUsage::{self, *},
		GGPixelFormat::{self, *},
	},
	game_graph_driver::{GGD_Camera, GGD_FontData, GGD_ImageData},
};
use libc::c_void;
use nice_engine::texture::ImmutableTexture;
use std::slice;
use vulkano::{format::Format::*, sync::GpuFuture};

#[allow(non_snake_case)]
pub extern fn ImageData_Alloc(usage: GGImageUsage, _x: u32, _y: u32, _format: GGPixelFormat) -> *mut GGD_ImageData {
	match usage {
		IMG_USAGE_STATIC | IMG_USAGE_GLYPH => Box::into_raw(Box::new(GGD_ImageData::Uninitialized(usage))),
		_ => unimplemented!(),
	}
}

#[allow(non_snake_case)]
pub unsafe extern fn ImageData_Free(this: *mut GGD_ImageData) {
	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub unsafe extern fn ImageData_SetPixelData(
	this: *mut GGD_ImageData,
	buffer: *const c_void,
	x: u32,
	y: u32,
	format: GGPixelFormat,
) {
	let this = &mut *this;

	match this {
		GGD_ImageData::Uninitialized(usage) => match *usage {
			IMG_USAGE_STATIC | IMG_USAGE_GLYPH => {
				let queue = ctx::get().queue().clone();
				let dims = [x, y];
				let len = x as usize * y as usize;

				let format = match format {
					PFMT_RGBA8_UNORM => R8G8B8A8Unorm,
					PFMT_RGBA8_SRGB => R8G8B8A8Srgb,
					PFMT_RGBA32F => R32G32B32A32Sfloat,
					_ => panic!("{:?} not supported", format),
				};

				let (tex, tex_future): (_, Box<dyn GpuFuture>) = match format {
					R8G8B8A8Unorm | R8G8B8A8Srgb => {
						let buffer = slice::from_raw_parts(buffer as *const [u8; 4], len).iter().cloned();
						let (tex, fut) = ImmutableTexture::from_iter_vk(queue, buffer, dims, format).unwrap();
						(tex, Box::new(fut))
					},
					R32G32B32A32Sfloat => {
						let buffer = slice::from_raw_parts(buffer as *const [f32; 4], len).iter().cloned();
						let (tex, fut) = ImmutableTexture::from_iter_vk(queue, buffer, dims, format).unwrap();
						(tex, Box::new(fut))
					},
					_ => panic!("{:?} not supported", format),
				};

				*this = GGD_ImageData::Immutable(tex);

				tex_future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
			},
			_ => unimplemented!(),
		},
		GGD_ImageData::Immutable(_) => panic!("cannot initialize static or glyph images twice"),
	}
}

// buffer can be null. x, y, and format are in/out params.
#[allow(non_snake_case)]
pub extern fn ImageData_GetPixelData(
	_this: *mut GGD_ImageData,
	_buffer: *mut c_void,
	_x: *mut u32,
	_y: *mut u32,
	_format: *mut GGPixelFormat,
) {
	unimplemented!();
}

#[allow(non_snake_case)]
pub extern fn ImageData_DrawCamera(_dst: *mut GGD_ImageData, _src: *mut GGD_Camera) {}

#[allow(non_snake_case)]
pub extern fn ImageData_DrawImage(
	_dst: *mut GGD_ImageData,
	_src: *mut GGD_ImageData,
	_x: f32,
	_y: f32,
	_w: f32,
	_h: f32,
) {
}

#[allow(non_snake_case)]
pub extern fn ImageData_DrawText(
	_dst: *mut GGD_ImageData,
	_src: *mut GGD_FontData,
	_x: f32,
	_y: f32,
	_text: *const char,
) {
}
