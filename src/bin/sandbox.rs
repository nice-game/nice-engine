use cgmath::{prelude::*, vec3, Deg, Quaternion, Rad};
use nice_engine::{camera::Camera, codecs::model::from_nice_model, mesh::Mesh, window::Window, Context};
use vulkano::sync::GpuFuture;
use winit::{dpi::LogicalSize, Event, EventsLoop, VirtualKeyCode, WindowEvent};

pub fn main() {
	let (ctx, ctx_future) = Context::new(Some("nIce Engine"), None).unwrap();
	let mut events = EventsLoop::new();
	let mut win = Window::new(&ctx, &events).unwrap();

	let (map, mats, map_future) = from_nice_model(&ctx, "assets/de_rebelzone/de_rebelzone.nmd");

	let mut mesh = Mesh::new(ctx.clone());
	mesh.transform_mut().rot = Quaternion::from_angle_x(Deg(-90.0));
	mesh.set_mesh_data(Some(map));
	mesh.set_materials(mats);

	let mut cam = Camera::new();
	cam.transform_mut().pos = vec3(-1.1, 0.0, 0.0);
	cam.transform_mut().rot = Quaternion::from_angle_z(Deg(-90.0)) * Quaternion::from_angle_y(Deg(90.0));
	cam.set_perspective(16.0 / 9.0, 90.0, 1.0, 50.0);

	ctx_future.join(map_future).then_signal_fence_and_flush().unwrap().wait(None).unwrap();
	loop {
		let mut done = false;

		events.poll_events(|event| match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::CloseRequested => done = true,
				WindowEvent::Resized(LogicalSize { width, height }) => {
					win.surface().resize(width as u32, height as u32)
				},
				WindowEvent::KeyboardInput { device_id, input } => match input.virtual_keycode {
					Some(key) => match key {
						VirtualKeyCode::W => {
							let mut t = cam.transform_mut();
							t.pos += t.rot.rotate_vector(vec3(0.0, 1.0, 0.0));
						},
						VirtualKeyCode::A => {
							let mut t = cam.transform_mut();
							t.pos += t.rot.rotate_vector(vec3(-1.0, 0.0, 0.0));
						},
						VirtualKeyCode::S => {
							let mut t = cam.transform_mut();
							t.pos += t.rot.rotate_vector(vec3(0.0, -1.0, 0.0));
						},
						VirtualKeyCode::D => {
							let mut t = cam.transform_mut();
							t.pos += t.rot.rotate_vector(vec3(1.0, 0.0, 0.0));
						},
						_ => (),
					},
					None => (),
				},
				_ => (),
			},
			_ => (),
		});

		if done {
			break;
		}

		win.surface().draw(&cam, &[&mesh]);
	}
}
