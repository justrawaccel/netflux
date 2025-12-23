#![windows_subsystem = "windows"]

mod app;
mod core;
mod sys;
mod ui;
mod utils;

use std::time::Duration;
use std::thread;
use winit::event::{ Event, WindowEvent };
use winit::event_loop::{ ControlFlow, EventLoopBuilder };
use tray_icon::TrayIconEvent;
use crate::app::App;
use crate::sys::registry::enable_autostart;

#[derive(Debug)]
enum UserEvent {
	Tick,
}

fn main() {
	enable_autostart();

	let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build().unwrap();
	let proxy = event_loop.create_proxy();

	thread::spawn(move || {
		loop {
			thread::sleep(Duration::from_millis(1000));
			let _ = proxy.send_event(UserEvent::Tick);
		}
	});

	let mut app = App::new(&event_loop);

	event_loop
		.run(move |event, elwt| {
			elwt.set_control_flow(ControlFlow::Wait);

			if let Ok(event) = TrayIconEvent::receiver().try_recv() {
				match event {
					TrayIconEvent::Click { button, .. } => {
						if button == tray_icon::MouseButton::Left {
							app.toggle_popup();
						}
					}
					_ => {}
				}
			}

			if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
				if app.handle_menu_event(event.id.0.as_str()) {
					elwt.exit();
				}
			}

			match event {
				Event::UserEvent(UserEvent::Tick) => {
					app.tick();
				}
				Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
					app.redraw_popup();
				}
				_ => {}
			}
		})
		.unwrap();
}
