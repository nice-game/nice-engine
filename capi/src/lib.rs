mod game_graph;
mod render_engine;

use crate::game_graph::GGDriverStatus;
use self::render_engine::{ RENDER_ENGINE, GGD_RenderEngine };
use libc::strlen;
use nice_engine::Version;
use std::{ os::raw::c_char, slice, str };

const GGD_API_VERSION: u64 = 0;

mod ctx {
	use nice_engine::{ Context, Version };

	static mut CTX: Option<Context> = None;

	pub unsafe fn get() -> &'static mut Context {
		match CTX {
			Some(ref mut ctx) => &mut *ctx,
			None => panic!("tried to access uninitialized context. GGD_DriverMain must be called first."),
		}
	}

	pub unsafe fn init(name: Option<&str>, version: Option<Version>) {
		CTX = Some(Context::new(name, version).unwrap());
	}
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern fn GGD_DriverMain(X: *mut GGD_DriverContext) -> GGDriverStatus {
	let X = &*X;

	if X.APIVersion == GGD_API_VERSION {
		(X.RegisterRenderEngine)(&mut RENDER_ENGINE);

		let name = Some(str::from_utf8_unchecked(slice::from_raw_parts(X.GameName as _, strlen(X.GameName))));
		let version = Some(Version::from_vulkan_version(X.GameVersion as u32));
		ctx::init(name, version);

		GGDriverStatus::DRIVER_READY
	} else {
		GGDriverStatus::VERSION_INVALID
	}
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct GGD_DriverContext {
	APIVersion: u64,
	GameVersion: u64,
	GameName: *const c_char,
	RegisterRenderEngine: extern fn (*mut GGD_RenderEngine),
	RegisterPhysicsEngine: extern fn (*mut GGD_PhysicsEngine),
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct GGD_PhysicsEngine { }

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
