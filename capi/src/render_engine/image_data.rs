use crate::{ctx, game_graph::*, game_graph_driver::*};
use futures::{future::lazy, task::SpawnExt};
use half::f16;
use log::trace;
use nice_engine::{texture::{ImmutableTexture, TargetTexture}, resources::TextureResource, threads::FILE_THREAD};
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
	pixelBuffer: *const GGD_BufferInfo,
	_cacheBuffer: *mut GGD_BufferInfo,
) -> *mut GGD_ImageData {
	trace!("ImageData_Alloc");

	if usage.contains(GGImageUsage::IMG_USAGE_TARGET) {
		let (tex, tex_future) =
			TargetTexture::new::<Format>(ctx::get().queue().clone(), [x, y], format.into()).unwrap();
		tex_future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
		Box::into_raw(Box::new(GGD_ImageData::Initialized(Arc::new(tex))))
	} else {
		let ret = Box::into_raw(Box::new(GGD_ImageData::Uninitialized { usage, x, y, format }));
		if pixelBuffer != null() {
			ImageData_DrawPixelData(ret, pixelBuffer);
		}
		ret
	}
}

#[allow(non_snake_case)]
pub unsafe extern fn ImageData_Free(this: *mut GGD_ImageData) {
	trace!("ImageData_Free");

	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub unsafe extern fn ImageData_DrawPixelData(this: *mut GGD_ImageData, buffer: *const GGD_BufferInfo) {
	trace!("ImageData_SetPixelData");

	let this = &mut *this;
	let buffer = &*buffer;

	match *this {
		GGD_ImageData::Uninitialized { usage, x, y, format } => {
			if usage.contains(GGImageUsage::IMG_USAGE_TARGET) {
				unimplemented!();
			} else {
				let res = TextureResource::new(ctx::get().resources().white_pixel().clone());
				let res_clone = res.clone();

				FILE_THREAD
					.lock()
					.unwrap()
					.spawn(lazy(move |_| {
						let pixels = (buffer.read)(buffer, 0, buffer.size);

						let queue = ctx::get().queue().clone();
						let dims = [x, y];
						let len = x as usize * y as usize;

						let format = format.into();

						let (tex, tex_future): (_, Box<dyn GpuFuture>) = match format {
							R8G8B8A8Unorm | R8G8B8A8Srgb => {
								let buffer = slice::from_raw_parts(pixels as *const [u8; 4], len).iter().cloned();
								let (tex, fut) = ImmutableTexture::from_iter_vk(queue, buffer, dims, format).unwrap();
								(tex, Box::new(fut))
							},
							R32G32B32A32Sfloat => {
								let buffer = slice::from_raw_parts(pixels as *const [f32; 4], len).iter().cloned();
								let (tex, fut) = ImmutableTexture::from_iter_vk(queue, buffer, dims, format).unwrap();
								(tex, Box::new(fut))
							},
							R16G16B16A16Sfloat => {
								let buffer = slice::from_raw_parts(pixels as *const [f16; 4], len).iter().cloned();
								let (tex, fut) = ImmutableTexture::from_iter_vk(queue, buffer, dims, format).unwrap();
								(tex, Box::new(fut))
							},
							_ => panic!("{:?} not supported", format),
						};

						tex_future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

						res_clone.set_texture(Arc::new(tex));

						if let Some(status) = buffer.status {
							status(buffer, GGD_BufferStatus::GGD_BUFFER_CLOSED as _);
						}
					}))
					.unwrap();

				*this = GGD_ImageData::Initialized(res);
			}
		},
		GGD_ImageData::Initialized(_) => panic!("cannot write to initialized image"),
	}
}

// buffer can be null. x, y, and format are in/out params.
#[allow(non_snake_case)]
pub unsafe extern fn ImageData_ReadPixelData(_this: *mut GGD_ImageData, _buffer: *mut GGD_BufferInfo) {
	trace!("ImageData_GetPixelData");

	unimplemented!();
}

#[allow(non_snake_case)]
pub extern fn ImageData_DrawCamera(_this: *mut GGD_ImageData, _src: *mut GGD_Camera) {
	trace!("ImageData_DrawCamera");
}

#[allow(non_snake_case)]
pub extern fn ImageData_DrawImage(
	_this: *mut GGD_ImageData,
	_src: *mut GGD_ImageData,
	_x: f32,
	_y: f32,
	_w: f32,
	_h: f32,
) {
	trace!("ImageData_DrawImage");
}

#[allow(non_snake_case)]
pub extern fn ImageData_DrawText(
	_this: *mut GGD_ImageData,
	_src: *mut GGD_FontData,
	_x: f32,
	_y: f32,
	_origin: GGTextOrigin,
	_text: *const char,
) {
	trace!("ImageData_DrawText");
}
