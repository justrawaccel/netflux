#![windows_subsystem = "windows"]

mod format;
mod icon;
mod net;
mod popup;

use std::time::Duration;
use std::thread;
use winit::event::{ Event, WindowEvent };
use winit::event_loop::{ ControlFlow, EventLoopBuilder };
use tray_icon::{ TrayIconBuilder, menu::{ Menu, MenuItem, Submenu, CheckMenuItem }, TrayIconEvent };
use crate::net::NetMonitor;
use crate::icon::IconGenerator;
use crate::popup::{ Popup, PopupMode };
use crate::format::format_speed_full;
use windows::Win32::System::Registry::{
    RegCreateKeyExW,
    RegSetValueExW,
    HKEY_CURRENT_USER,
    KEY_WRITE,
    REG_SZ,
    REG_OPTION_NON_VOLATILE,
    HKEY,
};

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

    let tray_menu = Menu::new();

    let mode_menu = Submenu::new("View Mode", true);
    let mode_all = CheckMenuItem::new("All", true, true, None);
    let mode_down = CheckMenuItem::new("Download Only", true, false, None);
    let mode_up = CheckMenuItem::new("Upload Only", true, false, None);

    mode_menu.append(&mode_all).unwrap();
    mode_menu.append(&mode_down).unwrap();
    mode_menu.append(&mode_up).unwrap();

    tray_menu.append(&mode_menu).unwrap();
    // tray_menu.append(&MenuItem::new_separator()).unwrap(); // Separator not available in this version or syntax wrong

    let quit_i = MenuItem::new("Exit", true, None);
    tray_menu.append(&quit_i).unwrap();

    let mut icon_gen = IconGenerator::new();
    let icon = icon_gen.generate(0).unwrap();

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
                } else if event.id == mode_all.id() {
                    let _ = mode_all.set_checked(true);
                    let _ = mode_down.set_checked(false);
                    let _ = mode_up.set_checked(false);
                    popup.set_mode(PopupMode::All);
                } else if event.id == mode_down.id() {
                    let _ = mode_all.set_checked(false);
                    let _ = mode_down.set_checked(true);
                    let _ = mode_up.set_checked(false);
                    popup.set_mode(PopupMode::DownloadOnly);
                } else if event.id == mode_up.id() {
                    let _ = mode_all.set_checked(false);
                    let _ = mode_down.set_checked(false);
                    let _ = mode_up.set_checked(true);
                    popup.set_mode(PopupMode::UploadOnly);
                }
            }

            match event {
                Event::UserEvent(UserEvent::Tick) => {
                    if let Some(stats) = net_monitor.tick() {
                        if let Ok(new_icon) = icon_gen.generate(stats.down_bps) {
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
                        popup.update(stats.down_bps, stats.up_bps, popup_text);
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

fn enable_autostart() {
    unsafe {
        let mut hkey = HKEY::default();
        let subkey = wide_string("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        if
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                windows::core::PCWSTR::from_raw(subkey.as_ptr()),
                0,
                None,
                REG_OPTION_NON_VOLATILE,
                KEY_WRITE,
                None,
                &mut hkey,
                None
            ).is_ok()
        {
            let exe_path = std::env::current_exe().unwrap_or_default();
            let path_str = exe_path.to_str().unwrap_or_default();
            let path_wide = wide_string(path_str);

            let val_name = wide_string("NetFlux");
            let _ = RegSetValueExW(
                hkey,
                windows::core::PCWSTR::from_raw(val_name.as_ptr()),
                0,
                REG_SZ,
                Some(
                    std::slice::from_raw_parts(path_wide.as_ptr() as *const u8, path_wide.len() * 2)
                )
            );
        }
    }
}

fn wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
