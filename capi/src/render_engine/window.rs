#[cfg(windows)]
use crate::game_graph_driver::GGD_WindowInfo_WIN32;
#[cfg(unix)]
use crate::game_graph_driver::{GGD_WindowInfo_WAYLAND, GGD_WindowInfo_X11};
use crate::{
	ctx,
	game_graph::GGPlatform,
	game_graph_driver::{GGD_Camera, GGD_ImageData, GGD_Window, GGD_WindowInfo},
};
use log::trace;
use nice_engine::surface::Surface;
use std::ptr::null_mut;

#[allow(non_snake_case)]
pub unsafe extern fn Window_Alloc(info: *mut GGD_WindowInfo) -> *mut GGD_Window {
	trace!("Window_Alloc");

	let info_ref = &*info;

	let surface = match GGPlatform::from_u64_unchecked(info_ref.platform) {
		#[cfg(windows)]
		GGPlatform::WIN32 => {
			let info_ref = &*(info as *mut GGD_WindowInfo_WIN32);
			Surface::from_hwnd(ctx::get(), info_ref.hinstance, info_ref.hwnd)
		},
		#[cfg(unix)]
		GGPlatform::WAYLAND => {
			let info_ref = &*(info as *mut GGD_WindowInfo_WAYLAND);
			Surface::from_wayland(ctx::get(), info_ref.display, info_ref.surface)
		},
		#[cfg(unix)]
		GGPlatform::X11 => {
			let info_ref = &*(info as *mut GGD_WindowInfo_X11);
			Surface::from_xlib(ctx::get(), info_ref.display, info_ref.surface)
		},
		#[cfg(unix)]
		GGPlatform::OSX => {
			let info_ref = &*(info as *mut GGD_WindowInfo_X11);
			Surface::from_xlib(ctx::get(), info_ref.display, info_ref.surface)
		},
		_ => panic!("invalid platform"),
	};

	if let Ok(surface) = surface { Box::into_raw(Box::new(surface)) } else { null_mut() }
}

#[allow(non_snake_case)]
pub unsafe extern fn Window_Free(this: *mut GGD_Window) {
	trace!("Window_Free");

	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub extern fn Window_IsValid(_this: *mut GGD_Window) -> i32 {
	trace!("Window_IsValid");

	true as i32
}

#[allow(non_snake_case)]
pub unsafe extern fn Window_Resize(this: *mut GGD_Window, w: u32, h: u32) {
	trace!("Window_Resize");

	let this = &mut *this;

	this.resize(w, h);
}

#[allow(non_snake_case)]
pub unsafe extern fn Window_SetCamera(this: *mut GGD_Window, camera: *mut GGD_Camera) {
	trace!("Window_SetCamera");

	let this = &mut *this;
	let camera = &mut *camera;

	this.set_camera(camera.clone());
}

#[allow(non_snake_case)]
pub extern fn Window_SetOverlay(_this: *mut GGD_Window, _overlay: *mut GGD_ImageData) {
	trace!("Window_SetOverlay");
}

#[allow(non_snake_case)]
pub unsafe extern fn Window_Draw(this: *mut GGD_Window) {
	trace!("Window_Draw");

	let this = &mut *this;

	this.draw();
}
