mod game_graph;
mod game_graph_driver;
mod render_engine;

use self::{game_graph_driver::GGD_DriverContext, render_engine::RENDER_ENGINE};
use crate::game_graph::GGDriverStatus;
use libc::strlen;
use nice_engine::Version;
use std::{slice, str};

const GGD_API_VERSION: u64 = 0;

mod ctx {
	use nice_engine::{Context, GpuFuture, Version};
	use std::sync::Arc;

	static mut CTX: Option<Arc<Context>> = None;

	pub unsafe fn get() -> &'static mut Arc<Context> {
		match CTX {
			Some(ref mut ctx) => &mut *ctx,
			None => panic!("tried to access uninitialized context. GGD_DriverMain must be called first."),
		}
	}

	pub unsafe fn init(name: Option<&str>, version: Option<Version>) {
		let (ctx, ctx_future) = Context::new(name, version).unwrap();
		ctx_future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
		CTX = Some(ctx);
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

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
