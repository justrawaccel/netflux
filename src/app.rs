use winit::event_loop::EventLoopWindowTarget;
use tray_icon::{ TrayIcon, TrayIconBuilder, menu::{ Menu, MenuItem, Submenu, CheckMenuItem } };

use crate::core::monitor::NetMonitor;
use crate::core::state::{ AppState, ViewMode };
use crate::ui::popup::Popup;
use crate::ui::tray::TrayIconGenerator;
use crate::utils::format::format_speed_full;

pub struct App {
	monitor: NetMonitor,
	state: AppState,
	popup: Popup,
	tray_icon: Option<TrayIcon>,

	menu_quit: MenuItem,
	menu_mode_all: CheckMenuItem,
	menu_mode_down: CheckMenuItem,
	menu_mode_up: CheckMenuItem,
}

impl App {
	pub fn new<T>(event_loop: &EventLoopWindowTarget<T>) -> Self {
		let tray_menu = Menu::new();

		let mode_menu = Submenu::new("View Mode", true);
		let menu_mode_all = CheckMenuItem::new("All", true, true, None);
		let menu_mode_down = CheckMenuItem::new("Download Only", true, false, None);
		let menu_mode_up = CheckMenuItem::new("Upload Only", true, false, None);

		mode_menu.append(&menu_mode_all).unwrap();
		mode_menu.append(&menu_mode_down).unwrap();
		mode_menu.append(&menu_mode_up).unwrap();

		tray_menu.append(&mode_menu).unwrap();

		let menu_quit = MenuItem::new("Exit", true, None);
		tray_menu.append(&menu_quit).unwrap();

		let icon = TrayIconGenerator::generate(0).unwrap();
		let tray_icon = Some(
			TrayIconBuilder::new()
				.with_menu(Box::new(tray_menu))
				.with_tooltip("NetFlux")
				.with_icon(icon)
				.build()
				.unwrap()
		);

		Self {
			monitor: NetMonitor::new(),
			state: AppState::new(),
			popup: Popup::new(event_loop),
			tray_icon,
			menu_quit,
			menu_mode_all,
			menu_mode_down,
			menu_mode_up,
		}
	}

	pub fn tick(&mut self) {
		if let Some(stats) = self.monitor.tick() {
			self.state.update(stats.down_bps, stats.up_bps, stats.interface_name);

			if let Ok(new_icon) = TrayIconGenerator::generate(self.state.down_bps) {
				if let Some(tray) = &mut self.tray_icon {
					let _ = tray.set_icon(Some(new_icon));
					let tooltip = format!(
						"Down: {} | Up: {}",
						format_speed_full(self.state.down_bps),
						format_speed_full(self.state.up_bps)
					);
					let _ = tray.set_tooltip(Some(tooltip));
				}
			}

			self.popup.update(&self.state);
		}
	}

	pub fn toggle_popup(&mut self) {
		self.popup.toggle(&self.state);
	}

	pub fn redraw_popup(&self) {
		self.popup.draw(&self.state);
	}

	pub fn handle_menu_event(&mut self, event_id: &str) -> bool {
		if event_id == self.menu_quit.id().0.as_str() {
			self.tray_icon = None;
			return true;
		} else if event_id == self.menu_mode_all.id().0.as_str() {
			self.set_view_mode(ViewMode::All);
		} else if event_id == self.menu_mode_down.id().0.as_str() {
			self.set_view_mode(ViewMode::DownloadOnly);
		} else if event_id == self.menu_mode_up.id().0.as_str() {
			self.set_view_mode(ViewMode::UploadOnly);
		}
		false
	}

	fn set_view_mode(&mut self, mode: ViewMode) {
		self.state.set_view_mode(mode);

		let _ = self.menu_mode_all.set_checked(mode == ViewMode::All);
		let _ = self.menu_mode_down.set_checked(mode == ViewMode::DownloadOnly);
		let _ = self.menu_mode_up.set_checked(mode == ViewMode::UploadOnly);

		self.popup.update(&self.state);
	}
}
