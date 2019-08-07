use cgmath::{prelude::*, vec3, Deg, Quaternion};
use nice_engine::{camera::Camera, window::Window, direct_light::DirectLight, Context};
use simplelog::{ LevelFilter, SimpleLogger };
use vulkano::sync::GpuFuture;
use winit::{dpi::LogicalSize, Event, EventsLoop, VirtualKeyCode, WindowEvent};

pub fn main() {
	SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default()).unwrap();

	let (ctx, ctx_future) = Context::new(Some("nIce Engine"), None).unwrap();
	let mut events = EventsLoop::new();
	let mut win = Window::new(&ctx, &events).unwrap();

	let mut map = ctx.resources().lock().unwrap().get_model("assets/de_rebelzone/de_rebelzone.nmd");
	map.transform_mut().rot = Quaternion::from_angle_x(Deg(90.0));

	let mut cam = Camera::new();
	cam.transform_mut().pos = vec3(17.0, 36.5, -12.0);
	cam.transform_mut().rot = Quaternion::from_angle_z(Deg(180.0));
	cam.set_perspective(16.0 / 9.0, 90.0, 1.0, 1000.0);

	let mut light1 = DirectLight::new();
	light1.position = vec3(23.0, 18.0, -12.0);
	light1.color = vec3(1.0, 0.75, 0.5625);
	light1.radius = 32.0;

	let mut light2 = DirectLight::new();
	light2.position = vec3(5.2, 21.8, -12.0);
	light2.color = vec3(0.5625, 0.75, 1.0);
	light2.radius = 32.0;

	ctx_future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

	loop {
		let mut done = false;

		events.poll_events(|event| match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::CloseRequested => done = true,
				WindowEvent::Resized(LogicalSize { width, height }) => {
					win.surface().resize(width as u32, height as u32)
				},
				WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
					Some(key) => match key {
						VirtualKeyCode::W => {
							let t = cam.transform_mut();
							t.pos += t.rot.rotate_vector(vec3(0.0, 1.0, 0.0));
						},
						VirtualKeyCode::A => {
							let t = cam.transform_mut();
							t.pos += t.rot.rotate_vector(vec3(-1.0, 0.0, 0.0));
						},
						VirtualKeyCode::S => {
							let t = cam.transform_mut();
							t.pos += t.rot.rotate_vector(vec3(0.0, -1.0, 0.0));
						},
						VirtualKeyCode::D => {
							let t = cam.transform_mut();
							t.pos += t.rot.rotate_vector(vec3(1.0, 0.0, 0.0));
						},
						VirtualKeyCode::Up => {
							let t = cam.transform_mut();
							t.pos += t.rot.rotate_vector(vec3(0.0, 0.0, 1.0));
						},
						VirtualKeyCode::Down => {
							let t = cam.transform_mut();
							t.pos += t.rot.rotate_vector(vec3(0.0, 0.0, -1.0));
						},
						VirtualKeyCode::Left => {
							let t = cam.transform_mut();
							t.rot = t.rot * Quaternion::from_angle_z(Deg(11.25));
						},
						VirtualKeyCode::Right => {
							let t = cam.transform_mut();
							t.rot = t.rot * Quaternion::from_angle_z(Deg(-11.25));
						},
						VirtualKeyCode::P => {
							let t = cam.transform_mut();
							println!("cam pos={:?} rot={:?}", t.pos, t.rot);
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

		map.refresh();
		win.surface().draw(&cam, &[&map], &[&light1, &light2]);
	}
}
