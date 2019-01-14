use nice_engine::window::Window;
use std::{ mem, os::raw::c_void };

#[allow(non_camel_case_types)]
pub type GGD_Window = Window;

type HINSTANCE = *mut c_void;
type HWND = *mut c_void;
type HDC = *mut c_void;

#[allow(non_snake_case)]
pub unsafe extern fn Window_Alloc(info: *mut GGD_WindowInfo) -> *mut GGD_Window {
	let info_ref = &*info;

	#[cfg(windows)]
	let window = match GGPlatform::from_u64_unchecked(info_ref.platform) {
		GGPlatform::PLAT_WIN32 => {
			let info_ref = &*(info as *mut GGD_WindowInfo_WIN32);
			Window::from_hwnd(info_ref.hinstance, info_ref.hwnd)
		},
		_ => panic!("invalid platform"),
	};

	#[cfg(unix)]
	let window = match GGPlatform::from_u64_unchecked(info_ref.platform) {
		GGPlatform::PLAT_WAYLAND => {
			let info_ref = &*(info as *mut GGD_WindowInfo_WAYLAND);
			Window::from_wayland(info_ref.display, info_ref.surface)
		},
		GGPlatform::PLAT_X11 => {
			let info_ref = &*(info as *mut GGD_WindowInfo_X11);
			Window::from_xlib(info_ref.display, info_ref.window)
		},
		GGPlatform::PLAT_OSX => {
			let info_ref = &*(info as *mut GGD_WindowInfo_X11);
			Window::from_xlib(info_ref.display, info_ref.window)
		},
		_ => panic!("invalid platform"),
	};

	Box::into_raw(Box::new(window))
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
pub extern fn Window_Resize(_this: *mut GGD_Window, _w: u32, _h: u32) {

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
	sdlwindow: *mut c_void,
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
	window: Window,
}
