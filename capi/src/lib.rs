mod render_engine;

use self::render_engine::{ RENDER_ENGINE, GGD_RenderEngine };
use libc::strlen;
use nice_engine::Version;
use std::{ ffi::CStr, os::raw::c_char, slice, str };

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

		GGDriverStatus::GGD_STATUS_DRIVER_READY
	} else {
		GGDriverStatus::GGD_STATUS_VERSION_INVALID
	}
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub enum GGDriverStatus {
	GGD_STATUS_DRIVER_INVALID = 0,
	GGD_STATUS_DRIVER_READY = 1,
	GGD_STATUS_DRIVER_ERROR = 2,
	GGD_STATUS_VERSION_INVALID = 3,
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
pub struct GGD_PhysicsEngine {
	Name: *const c_char,
	Priority: u64,
	Validate: Option<extern fn () -> i32>,
	Shutdown: Option<extern fn (*mut GGD_PhysicsEngine) -> i32>,

	// Shape_Alloc: extern fn () -> *mut GGD_Shape,
	// Shape_Free: extern fn (*mut GGD_Shape),
	// Shape_SetCacheData: extern fn (*mut GGD_Shape, buffer: *const c_void, size: u32) -> i32,
	// Shape_GetCacheData: extern fn (*mut GGD_Shape, buffer: *mut c_void, size: *mut u32) -> i32,
	// Shape_SetBox: extern fn (*mut GGD_Shape, x: f32, y: f32, z: f32),
	// Shape_SetSphere: extern fn (*mut GGD_Shape, radius: f32),
	// Shape_SetCylinder: extern fn (*mut GGD_Shape, radius: f32, height: f32),
	// Shape_SetConvexMesh: extern fn (*mut GGD_Shape, vertices: *const c_void, count: u32, format: GGVertexFormat),
	// Shape_SetTriangleMesh: extern fn (*mut GGD_Shape, vertices: *const c_void, vcount: u32, vformat: GGVertexFormat,
	// 											indices: *const c_void, icount: u32, iformat: GGIndexFormat),
	// Shape_SetDistanceData: extern fn (*mut GGD_Shape, buffer: *const c_void, x: u32, y: u32, z: u32, format: GGDistanceFormat),

	// Simulation_Alloc: extern fn () -> *mut GGD_Simulation,
	// Simulation_Free: extern fn (*mut GGD_Simulation),
	// Simulation_Gravity: extern fn (*mut GGD_Simulation, x: f32, y: f32, z: f32),
	// Simulation_Update: extern fn (*mut GGD_Simulation, dt: f32),

	// ShapeInstance_Alloc: extern fn (*mut GGD_Simulation, *mut GGD_Shape) -> *mut GGD_ShapeInstance,
	// ShapeInstance_Free: extern fn (*mut GGD_ShapeInstance),
	// ShapeInstance_SetMass: extern fn (*mut GGD_ShapeInstance, mass: f32),
	// ShapeInstance_GetMass: extern fn (*mut GGD_ShapeInstance) -> f32,
	// ShapeInstance_SetFriction: extern fn (*mut GGD_ShapeInstance, friction: f32),
	// ShapeInstance_GetFriction: extern fn (*mut GGD_ShapeInstance) -> f32,
	// ShapeInstance_SetVelocity: extern fn (*mut GGD_ShapeInstance, poseDt: *mut GGTransform),
	// ShapeInstance_GetVelocity: extern fn (*mut GGD_ShapeInstance, poseDt: *mut GGTransform),
	// ShapeInstance_SetTransform: extern fn (*mut GGD_ShapeInstance, pose: *mut GGTransform),
	// ShapeInstance_GetTransform: extern fn (*mut GGD_ShapeInstance, pose: *mut GGTransform),
	// ShapeInstance_SetVelocityPointer: extern fn (*mut GGD_ShapeInstance, poseDtPtr: *mut GGTransform),
	// ShapeInstance_SetTransformPointer: extern fn (*mut GGD_ShapeInstance, posePtr: *mut GGTransform),
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
