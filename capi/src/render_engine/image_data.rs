use crate::{
	ctx,
	game_graph::{
		GGImageUsage::{self, *},
		GGPixelFormat, GGTextOrigin,
	},
	game_graph_driver::{GGD_Camera, GGD_FontData, GGD_ImageData},
};
use half::f16;
use libc::c_void;
use nice_engine::texture::{ImmutableTexture, TargetTexture};
use std::{ptr::null, slice, sync::Arc};
use vulkano::{
	format::Format::{self, *},
	sync::GpuFuture,
};

#[allow(non_snake_case)]
pub unsafe extern fn ImageData_Alloc(
	usage: GGImageUsage,
	x: u32,
	y: u32,
	format: GGPixelFormat,
	buffer: *const c_void,
) -> *mut GGD_ImageData {
	match usage {
		IMG_USAGE_STATIC | IMG_USAGE_GLYPH => {
			let ret = Box::into_raw(Box::new(GGD_ImageData::Uninitialized { usage, x, y, format }));
			if buffer != null() {
				ImageData_SetPixelData(ret, buffer, 0);
			}
			ret
		},
		IMG_USAGE_TARGET => {
			let tex = TargetTexture::new::<Format>(ctx::get().device().clone(), [x, y], format.into()).unwrap();
			Box::into_raw(Box::new(GGD_ImageData::Initialized(Arc::new(tex))))
		},
		_ => unimplemented!(),
	}
}

#[allow(non_snake_case)]
pub unsafe extern fn ImageData_Free(this: *mut GGD_ImageData) {
	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub unsafe extern fn ImageData_SetPixelData(this: *mut GGD_ImageData, buffer: *const c_void, mipmap: i32) -> i32 {
	let this = &mut *this;

	if mipmap != 0 {
		return 1;
	}

	match *this {
		GGD_ImageData::Uninitialized { usage, x, y, format } => match usage {
			IMG_USAGE_STATIC | IMG_USAGE_GLYPH => {
				let queue = ctx::get().queue().clone();
				let dims = [x, y];
				let len = x as usize * y as usize;

				let format = format.into();

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
					R16G16B16A16Sfloat => {
						let buffer = slice::from_raw_parts(buffer as *const [f16; 4], len).iter().cloned();
						let (tex, fut) = ImmutableTexture::from_iter_vk(queue, buffer, dims, format).unwrap();
						(tex, Box::new(fut))
					},
					_ => panic!("{:?} not supported", format),
				};

				*this = GGD_ImageData::Initialized(Arc::new(tex));

				tex_future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
			},
			_ => unimplemented!(),
		},
		GGD_ImageData::Initialized(_) => panic!("cannot write to initialized image"),
	}

	1
}

// buffer can be null. x, y, and format are in/out params.
#[allow(non_snake_case)]
pub extern fn ImageData_GetPixelData(_this: *mut GGD_ImageData, _buffer: *mut c_void, _mipmap: i32) -> i32 {
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
	_origin: GGTextOrigin,
	_text: *const char,
) {
}
