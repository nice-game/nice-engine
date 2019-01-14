use std::os::raw::c_ulong;

pub struct Window { }
impl Window {
	pub fn new() -> Self {
		Self { }
	}

	pub unsafe fn from_hwnd<T, U>(_hinstance: *const T, _hwnd: *const U) -> Self {
		Self { }
	}

	pub unsafe fn from_xlib<D>(_display: *const D, _window: c_ulong) -> Self {
		Self { }
	}

	pub unsafe fn from_wayland<D, S>(_display: *const D, _surface: *const S) -> Self {
		Self { }
	}
}

#[cfg(test)]
mod tests {
	use super::Window;

	#[test]
	fn it_creates_window() {
		Window::new();
	}
}
