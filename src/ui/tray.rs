use windows::Win32::Foundation::{ HWND, SIZE };
use windows::Win32::Graphics::Gdi::{
	GetDC,
	CreateCompatibleDC,
	CreateDIBSection,
	SelectObject,
	DeleteDC,
	DeleteObject,
	SetBkMode,
	SetTextColor,
	TextOutW,
	GetTextExtentPoint32W,
	RoundRect,
	ReleaseDC,
	TRANSPARENT,
	BITMAPINFO,
	BITMAPINFOHEADER,
	BI_RGB,
	DIB_RGB_COLORS,
	FW_SEMIBOLD,
};
use std::ffi::c_void;
use crate::sys::gdi::{ create_solid_brush, create_pen, create_font, DcScope };
use crate::ui::theme::*;
use crate::utils::format::format_speed_compact;

pub struct TrayIconGenerator;

impl TrayIconGenerator {
	pub fn generate(speed: u64) -> Result<tray_icon::Icon, String> {
		unsafe {
			let width = 32;
			let height = 32;

			let bmi = BITMAPINFO {
				bmiHeader: BITMAPINFOHEADER {
					biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
					biWidth: width,
					biHeight: -height,
					biPlanes: 1,
					biBitCount: 32,
					biCompression: BI_RGB.0,
					..Default::default()
				},
				..Default::default()
			};

			let hdc_screen = GetDC(HWND(std::ptr::null_mut()));
			let hdc_mem = CreateCompatibleDC(hdc_screen);
			if hdc_mem.is_invalid() {
				return Err("CreateCompatibleDC failed".to_string());
			}

			let mut bits: *mut c_void = std::ptr::null_mut();
			let hbitmap = CreateDIBSection(hdc_mem, &bmi, DIB_RGB_COLORS, &mut bits, None, 0).map_err(|e|
				e.to_string()
			)?;

			if bits.is_null() {
				return Err("CreateDIBSection returned null bits".to_string());
			}

			let old_bmp = SelectObject(hdc_mem, hbitmap);

			{
				let mut scope = DcScope::new(hdc_mem);

				std::ptr::write_bytes(bits, 0, (width * height * 4) as usize);

				let bg_brush = create_solid_brush(COLOR_BG);
				let null_pen = create_pen(windows::Win32::Graphics::Gdi::PS_NULL, 0, 0);
				scope.select(&bg_brush);
				scope.select(&null_pen);
				let _ = RoundRect(hdc_mem, 0, 0, width, height, 6, 6);

				SetBkMode(hdc_mem, TRANSPARENT);

				let (val_str, unit_str) = format_speed_compact(speed);

				let color = if speed < 100 * 1024 {
					COLOR_TEXT_GRAY
				} else if speed < 5 * 1024 * 1024 {
					COLOR_DOWN_LINE
				} else {
					0x0008b3ea
				};

				let _ = SetTextColor(hdc_mem, windows::Win32::Foundation::COLORREF(color));

				let font_height = match val_str.len() {
					1 => -22,
					2 => -18,
					_ => -14,
				};

				let font_val = create_font(font_height, FW_SEMIBOLD.0 as i32, FONT_FACE);
				scope.select(&font_val);

				let w_val = wide_string(&val_str);
				let mut size_val = SIZE::default();
				let _ = GetTextExtentPoint32W(hdc_mem, &w_val, &mut size_val);
				let x_val = (width - size_val.cx) / 2;

				let y_val = match val_str.len() {
					1 => -5,
					2 => -3,
					_ => -1,
				};
				let _ = TextOutW(hdc_mem, x_val, y_val, &w_val);

				let font_unit = create_font(-11, 0, FONT_FACE);
				scope.select(&font_unit);
				let w_unit = wide_string(&unit_str);
				let mut size_unit = SIZE::default();
				let _ = GetTextExtentPoint32W(hdc_mem, &w_unit, &mut size_unit);
				let x_unit = (width - size_unit.cx) / 2;
				let _ = TextOutW(hdc_mem, x_unit, 15, &w_unit);
			}

			let pixel_count = (width * height) as usize;
			let pixels = std::slice::from_raw_parts_mut(bits as *mut u32, pixel_count);
			for p in pixels.iter_mut() {
				if *p != 0 {
					*p |= 0xff000000;
				}
			}

			let mut rgba = Vec::with_capacity(pixel_count * 4);
			for p in pixels.iter() {
				let a = (*p >> 24) as u8;
				let r = (*p >> 16) as u8;
				let g = (*p >> 8) as u8;
				let b = *p as u8;
				rgba.push(r);
				rgba.push(g);
				rgba.push(b);
				rgba.push(a);
			}

			let _ = SelectObject(hdc_mem, old_bmp);
			let _ = DeleteObject(hbitmap);
			let _ = DeleteDC(hdc_mem);
			ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);

			tray_icon::Icon::from_rgba(rgba, width as u32, height as u32).map_err(|e| e.to_string())
		}
	}
}

fn wide_string(s: &str) -> Vec<u16> {
	s.encode_utf16().chain(std::iter::once(0)).collect()
}
