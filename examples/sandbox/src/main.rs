use cgmath::{prelude::*, vec2, vec3, vec4, Deg, Quaternion};
use nice_engine::{camera::Camera, mesh_group::MeshGroup, transform::Transform, window::Window, Context, GpuFuture};
use simplelog::{LevelFilter, SimpleLogger};
use std::{collections::HashSet, time::Instant};
use winit::{
	dpi::LogicalSize, DeviceEvent, ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode, WindowEvent,
};

pub fn main() {
	SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default()).unwrap();

	let (ctx, ctx_future) = Context::new(Some("nIce Engine"), None).unwrap();
	let mut events = EventsLoop::new();
	let mut win = Window::new(&ctx, &events).unwrap();

	let mesh_group = MeshGroup::new(&ctx);

	let map = ctx.resources().get_model(mesh_group.clone(), "assets/de_rebelzone/de_rebelzone.nmd");
	for mesh in &map {
		mesh.inner()
			.write()
			.unwrap()
			.set_transform(Transform { rot: Quaternion::from_angle_x(Deg(90.0)), ..Transform::default() });
	}

	// let mut gun = ctx.resources().get_model("assets/p250/p250.nmd");
	// gun.transform_mut().pos = vec3(22.981121, 20.031065, -13.0);
	// ctx.world().add_mesh(gun);

	let mut cam = Camera::new(&ctx);
	*cam.mesh_group_mut() = mesh_group.clone();
	cam.transform_mut().pos = vec4(17.0, 36.5, -12.0, 1.0);
	cam.transform_mut().rot = Quaternion::from_angle_z(Deg(180.0));
	cam.set_perspective(16.0 / 9.0, 90.0, 1.0, 1000.0);

	ctx_future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

	let mut focused = true;
	let mut keys = HashSet::new();
	let mut frame_instant = Instant::now();
	loop {
		let mut done = false;
		let mut rotation = vec2(0.0, 0.0);

		events.poll_events(|event| match event {
			Event::DeviceEvent { event, .. } => match event {
				DeviceEvent::Key(KeyboardInput { virtual_keycode, state, .. }) => match virtual_keycode {
					Some(virtual_keycode) => match state {
						ElementState::Pressed => {
							keys.insert(virtual_keycode);
						},
						ElementState::Released => {
							keys.remove(&virtual_keycode);
						},
					},
					None => (),
				},
				DeviceEvent::MouseMotion { delta } => rotation += vec2(delta.0 as f32, delta.1 as f32),
				_ => (),
			},
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::CloseRequested => done = true,
				WindowEvent::Focused(focus) => {
					focused = focus;
					win.hide_cursor(focus);
					win.grab_cursor(focus).unwrap();
				},
				WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
					Some(key) => match key {
						VirtualKeyCode::Escape => done = true,
						_ => (),
					},
					None => (),
				},
				WindowEvent::Resized(LogicalSize { width, height }) => {
					win.surface().resize(width as u32, height as u32)
				},
				_ => (),
			},
			_ => (),
		});

		if done {
			break;
		}

		// delta time in seconds
		let now = Instant::now();
		let dtime = now.duration_since(frame_instant).as_micros() as f32 / 1000000.0;
		frame_instant = now;

		if focused {
			let speed = 10.0;
			let movement = vec3(
				keys.contains(&VirtualKeyCode::D) as u32 as f32 - keys.contains(&VirtualKeyCode::A) as u32 as f32,
				keys.contains(&VirtualKeyCode::W) as u32 as f32 - keys.contains(&VirtualKeyCode::S) as u32 as f32,
				keys.contains(&VirtualKeyCode::Space) as u32 as f32
					- keys.contains(&VirtualKeyCode::LShift) as u32 as f32,
			) * dtime * speed;

			let mouse_sensitivity = 0.2;
			rotation *= mouse_sensitivity;

			let t = cam.transform_mut();
			t.rot = Quaternion::from_angle_z(Deg(-rotation.x as f32))
				* t.rot * Quaternion::from_angle_x(Deg(-rotation.y as f32));
			t.pos += t.rot.rotate_vector(movement).extend(0.0);
		}

		win.surface().draw();
	}
}
