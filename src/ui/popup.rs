use winit::window::{ Window, WindowBuilder, WindowLevel };
use winit::event_loop::EventLoopWindowTarget;
use winit::platform::windows::WindowBuilderExtWindows;
use windows::Win32::Foundation::{ HWND, RECT, POINT };
use windows::Win32::Graphics::Gdi::{
	GetDC,
	ReleaseDC,
	FillRect,
	Polygon,
	Polyline,
	TextOutW,
	SetBkMode,
	SetTextColor,
	TRANSPARENT,
	PS_SOLID,
	PS_NULL,
	FW_SEMIBOLD,
	FW_BOLD,
};
use windows::Win32::UI::WindowsAndMessaging::{
	SystemParametersInfoW,
	SPI_GETWORKAREA,
	SPIF_SENDCHANGE,
};
use raw_window_handle::{ HasWindowHandle, RawWindowHandle };

use crate::core::state::{ AppState, ViewMode };
use crate::sys::gdi::{ create_solid_brush, create_pen, create_font, DcScope };
use crate::ui::theme::*;
use crate::utils::format::format_speed_full;

pub struct Popup {
	window: Window,
}

impl Popup {
	pub fn new<T>(event_loop: &EventLoopWindowTarget<T>) -> Self {
		let window = WindowBuilder::new()
			.with_title("NetFlux Popup")
			.with_decorations(false)
			.with_window_level(WindowLevel::AlwaysOnTop)
			.with_skip_taskbar(true)
			.with_resizable(false)
			.with_inner_size(winit::dpi::LogicalSize::new(240.0, 220.0))
			.with_visible(false)
			.build(event_loop)
			.expect("Failed to create popup window");

		Self { window }
	}

	pub fn toggle(&self, state: &AppState) {
		if self.window.is_visible().unwrap_or(false) {
			self.window.set_visible(false);
		} else {
			self.reposition(state);
			self.window.set_visible(true);
			self.window.request_redraw();
		}
	}

	pub fn update(&self, state: &AppState) {
		if self.window.is_visible().unwrap_or(false) {
			self.reposition(state);
			self.window.request_redraw();
		}
	}

	pub fn draw(&self, state: &AppState) {
		let handle = self.window.window_handle().unwrap().as_raw();
		if let RawWindowHandle::Win32(handle) = handle {
			let hwnd = HWND(handle.hwnd.get() as _);
			unsafe {
				let hdc = GetDC(hwnd);
				let mut scope = DcScope::new(hdc);

				let width = 240;
				let height = match state.view_mode {
					ViewMode::All => 220,
					_ => 110,
				};

				let rect = RECT { left: 0, top: 0, right: width, bottom: height };
				let bg_brush = create_solid_brush(COLOR_BG);
				let _ = FillRect(hdc, &rect, bg_brush.as_brush());

				SetBkMode(hdc, TRANSPARENT);

				let font_label = create_font(-12, FW_SEMIBOLD.0 as i32, FONT_FACE);
				let font_val = create_font(-24, FW_BOLD.0 as i32, FONT_FACE);

				if state.view_mode == ViewMode::All || state.view_mode == ViewMode::DownloadOnly {
					let max_down = std::cmp::max(*state.down_history.iter().max().unwrap_or(&1), 1024 * 1024);
					let baseline = 100;
					let graph_h = 50.0;

					let fill_brush = create_solid_brush(COLOR_DOWN_FILL);
					let null_pen = create_pen(PS_NULL, 0, 0);
					scope.select(&fill_brush);
					scope.select(&null_pen);

					let mut points = Vec::with_capacity(state.down_history.len() + 2);
					points.push(POINT { x: 0, y: baseline });
					for (i, &val) in state.down_history.iter().enumerate() {
						let h = (((val as f64) / (max_down as f64)) * graph_h) as i32;
						points.push(POINT { x: i as i32, y: baseline - h });
					}
					points.push(POINT { x: state.down_history.len() as i32, y: baseline });
					let _ = Polygon(hdc, &points);

					let line_pen = create_pen(PS_SOLID, 2, COLOR_DOWN_LINE);
					scope.select(&line_pen);
					let mut line_points = Vec::with_capacity(state.down_history.len());
					for (i, &val) in state.down_history.iter().enumerate() {
						let h = (((val as f64) / (max_down as f64)) * graph_h) as i32;
						line_points.push(POINT { x: i as i32, y: baseline - h });
					}
					if !line_points.is_empty() {
						let _ = Polyline(hdc, &line_points);
					}

					SetTextColor(hdc, windows::Win32::Foundation::COLORREF(COLOR_TEXT_GRAY));
					scope.select(&font_label);
					let label = wide_string("↓ DOWNLOAD");
					let _ = TextOutW(hdc, 16, 12, &label);

					SetTextColor(hdc, windows::Win32::Foundation::COLORREF(COLOR_TEXT_WHITE));
					scope.select(&font_val);
					let val = wide_string(&format_speed_full(state.down_bps));
					let _ = TextOutW(hdc, 16, 30, &val);
				}

				if state.view_mode == ViewMode::All || state.view_mode == ViewMode::UploadOnly {
					let max_up = std::cmp::max(*state.up_history.iter().max().unwrap_or(&1), 1024 * 1024);
					let baseline = if state.view_mode == ViewMode::All { 210 } else { 100 };
					let graph_h = 50.0;
					let y_label = if state.view_mode == ViewMode::All { 120 } else { 12 };
					let y_val = if state.view_mode == ViewMode::All { 138 } else { 30 };

					let fill_brush = create_solid_brush(COLOR_UP_FILL);
					let null_pen = create_pen(PS_NULL, 0, 0);
					scope.select(&fill_brush);
					scope.select(&null_pen);

					let mut points = Vec::with_capacity(state.up_history.len() + 2);
					points.push(POINT { x: 0, y: baseline });
					for (i, &val) in state.up_history.iter().enumerate() {
						let h = (((val as f64) / (max_up as f64)) * graph_h) as i32;
						points.push(POINT { x: i as i32, y: baseline - h });
					}
					points.push(POINT { x: state.up_history.len() as i32, y: baseline });
					let _ = Polygon(hdc, &points);

					let line_pen = create_pen(PS_SOLID, 2, COLOR_UP_LINE);
					scope.select(&line_pen);
					let mut line_points = Vec::with_capacity(state.up_history.len());
					for (i, &val) in state.up_history.iter().enumerate() {
						let h = (((val as f64) / (max_up as f64)) * graph_h) as i32;
						line_points.push(POINT { x: i as i32, y: baseline - h });
					}
					if !line_points.is_empty() {
						let _ = Polyline(hdc, &line_points);
					}

					SetTextColor(hdc, windows::Win32::Foundation::COLORREF(COLOR_TEXT_GRAY));
					scope.select(&font_label);
					let label = wide_string("↑ UPLOAD");
					let _ = TextOutW(hdc, 16, y_label, &label);

					SetTextColor(hdc, windows::Win32::Foundation::COLORREF(COLOR_TEXT_WHITE));
					scope.select(&font_val);
					let val = wide_string(&format_speed_full(state.up_bps));
					let _ = TextOutW(hdc, 16, y_val, &val);
				}

				ReleaseDC(hwnd, hdc);
			}
		}
	}

	fn reposition(&self, state: &AppState) {
		unsafe {
			let mut rect = RECT::default();
			if
				SystemParametersInfoW(
					SPI_GETWORKAREA,
					0,
					Some(&mut rect as *mut _ as *mut _),
					SPIF_SENDCHANGE
				).is_ok()
			{
				let width = 240;
				let height = match state.view_mode {
					ViewMode::All => 220,
					_ => 110,
				};
				let x = rect.right - width - 12;
				let y = rect.bottom - height - 12;
				self.window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
				let _ = self.window.request_inner_size(
					winit::dpi::LogicalSize::new(width as f64, height as f64)
				);
			}
		}
	}
}

fn wide_string(s: &str) -> Vec<u16> {
	s.encode_utf16().chain(std::iter::once(0)).collect()
}
