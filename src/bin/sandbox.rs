use nice_engine::{window::Window, Context};
use winit::{dpi::LogicalSize, Event, EventsLoop, WindowEvent};

pub fn main() {
	let mut ctx = Context::new(Some("nIce Engine"), None).unwrap();
	let mut events = EventsLoop::new();
	let mut win = Window::new(&mut ctx, &events).unwrap();

	loop {
		let mut done = false;
		events.poll_events(|event| match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::CloseRequested => done = true,
				WindowEvent::Resized(LogicalSize { width, height }) => win.resize(width as u32, height as u32),
				_ => (),
			},
			_ => (),
		});

		win.draw(&ctx);

		if done {
			break;
		}
	}
}
