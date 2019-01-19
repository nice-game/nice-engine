use crate::ctx;
use nice_engine::surface::Surface;
use std::{ mem, os::raw::c_void, ptr::null_mut };

#[allow(non_camel_case_types)]
pub type GGD_Window = Surface;

type HINSTANCE = *mut c_void;
type HWND = *mut c_void;
type HDC = *mut c_void;

#[allow(non_snake_case)]
pub unsafe extern fn Window_Alloc(info: *mut GGD_WindowInfo) -> *mut GGD_Window {
	let info_ref = &*info;

	let surface = match GGPlatform::from_u64_unchecked(info_ref.platform) {
		#[cfg(windows)]
		GGPlatform::PLAT_WIN32 => {
			let info_ref = &*(info as *mut GGD_WindowInfo_WIN32);
			Surface::from_hwnd(ctx::get(), info_ref.hinstance, info_ref.hwnd)
		},
		#[cfg(unix)]
		GGPlatform::PLAT_WAYLAND => {
			let info_ref = &*(info as *mut GGD_WindowInfo_WAYLAND);
			Surface::from_wayland(ctx::get(), info_ref.display, info_ref.surface)
		},
		#[cfg(unix)]
		GGPlatform::PLAT_X11 => {
			let info_ref = &*(info as *mut GGD_WindowInfo_X11);
			Surface::from_xlib(ctx::get(), info_ref.display, info_ref.surface)
		},
		#[cfg(unix)]
		GGPlatform::PLAT_OSX => {
			let info_ref = &*(info as *mut GGD_WindowInfo_X11);
			Surface::from_xlib(ctx::get(), info_ref.display, info_ref.surface)
		},
		_ => panic!("invalid platform"),
	};

	if let Ok(surface) = surface {
		Box::into_raw(Box::new(surface))
	} else {
		null_mut()
	}
}

#[allow(non_snake_case)]
pub unsafe extern fn Window_Free(this: *mut GGD_Window) {
	Box::from_raw(this);
}

#[allow(non_snake_case)]
pub extern fn Window_IsValid(_this: *mut GGD_Window) -> i32 {
	true as i32
}

#[allow(non_snake_case)]
pub unsafe extern fn Window_Resize(this: *mut GGD_Window, w: u32, h: u32) {
	let this_ref = &mut *this;

	this_ref.resize(w, h);
}

// pub extern fn Window_Draw(this: *mut GGD_Window, out: *mut GGD_ImageData) {

// }

#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum GGPlatform {
	PLAT_UNDEFINED = 0,
	PLAT_WIN32 = 1,
	PLAT_X11 = 2,
	PLAT_WAYLAND = 3,
	PLAT_OSX = 4,
}
impl GGPlatform {
	unsafe fn from_u64_unchecked(val: u64) -> Self {
		mem::transmute(val as u32)
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GGD_WindowInfo {
	/// GGPlatform
	platform: u64,
	sdlsurface: *mut c_void,
}

#[cfg(unix)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GGD_WindowInfo_WAYLAND {
	info: GGD_WindowInfo,
	display: *mut wl_display,
	surface: *mut wl_surface,
	wmsurface: *mut wl_shell_surface,
}

#[cfg(windows)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GGD_WindowInfo_WIN32 {
	info: GGD_WindowInfo,
	hinstance: HINSTANCE,
	hwnd: HWND,
	hdc: HDC,
}

#[cfg(unix)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GGD_WindowInfo_X11 {
	info: GGD_WindowInfo,
	display: *mut Display,
	surface: Surface,
}
