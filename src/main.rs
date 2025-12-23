#![windows_subsystem = "windows"]

mod format;
mod icon;
mod net;
mod popup;

use std::time::Duration;
use std::thread;
use winit::event::{ Event, WindowEvent };
use winit::event_loop::{ ControlFlow, EventLoopBuilder };
use tray_icon::{ TrayIconBuilder, menu::{ Menu, MenuItem }, TrayIconEvent };
use crate::net::NetMonitor;
use crate::icon::IconGenerator;
use crate::popup::Popup;
use crate::format::{ format_speed, format_speed_full };

#[derive(Debug)]
enum UserEvent {
    Tick,
}

fn main() {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(1000));
            let _ = proxy.send_event(UserEvent::Tick);
        }
    });

    let tray_menu = Menu::new();
    let quit_i = MenuItem::new("Exit", true, None);
    tray_menu.append(&quit_i).unwrap();

    let icon_gen = IconGenerator::new();
    let icon = icon_gen.generate("...").unwrap();

    let mut tray_icon = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("NetFlux")
            .with_icon(icon)
            .build()
            .unwrap()
    );

    let mut net_monitor = NetMonitor::new();
    let mut popup = Popup::new(&event_loop);

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);

            if let Ok(event) = TrayIconEvent::receiver().try_recv() {
                match event {
                    TrayIconEvent::Click { button, .. } => {
                        if button == tray_icon::MouseButton::Left {
                            popup.toggle();
                        }
                    }
                    _ => {}
                }
            }

            if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
                if event.id == quit_i.id() {
                    tray_icon = None;
                    elwt.exit();
                }
            }

            match event {
                Event::UserEvent(UserEvent::Tick) => {
                    if let Some(stats) = net_monitor.tick() {
                        let label = format_speed(stats.down_bps);
                        if let Ok(new_icon) = icon_gen.generate(&label) {
                            if let Some(tray) = &mut tray_icon {
                                let _ = tray.set_icon(Some(new_icon));
                                let tooltip = format!(
                                    "Down: {} | Up: {}",
                                    format_speed_full(stats.down_bps),
                                    format_speed_full(stats.up_bps)
                                );
                                let _ = tray.set_tooltip(Some(tooltip));
                            }
                        }

                        let popup_text = format!(
                            "Interface: {}\nDown: {}\nUp: {}",
                            stats.interface_name,
                            format_speed_full(stats.down_bps),
                            format_speed_full(stats.up_bps)
                        );
                        popup.update_text(popup_text);
                    }
                }
                Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                    popup.draw();
                }
                _ => {}
            }
        })
        .unwrap();
}
