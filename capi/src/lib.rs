#![allow(improper_ctypes)]
#![feature(async_closure)]

mod game_graph;
mod game_graph_driver;
mod render_engine;

use self::{game_graph_driver::GGD_DriverContext, render_engine::RENDER_ENGINE};
use crate::game_graph::GGDriverStatus::{self, *};
use libc::strlen;
use nice_engine::Version;
use simplelog::{LevelFilter, SimpleLogger};
use std::{panic, ptr::null, slice, str};

const GGD_API_VERSION: u64 = 0;

mod ctx {
	use nice_engine::{Context, GpuFuture, Version};
	use std::sync::Arc;

	static mut CTX: Option<Arc<Context>> = None;

	pub unsafe fn get() -> &'static Arc<Context> {
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
	panic::set_hook(Box::new(|info| println!("{}", info)));
	SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default()).unwrap();

	let X = &*X;

	if X.APIVersion == GGD_API_VERSION {
		let name = if X.GameName != null() {
			Some(str::from_utf8_unchecked(slice::from_raw_parts(X.GameName as _, strlen(X.GameName))))
		} else {
			None
		};
		let version = Some(Version::from_vulkan_version(X.GameVersion as u32));
		ctx::init(name, version);

		(X.RegisterRenderEngine)(&RENDER_ENGINE);

		GGD_STATUS_DRIVER_READY
	} else {
		GGD_STATUS_VERSION_INVALID
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
