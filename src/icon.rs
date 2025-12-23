use windows::Win32::Foundation::{ HWND, SIZE };
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC,
    CreateFontW,
    DeleteDC,
    DeleteObject,
    SelectObject,
    SetBkMode,
    SetTextColor,
    TextOutW,
    TRANSPARENT,
    FW_BOLD,
    FW_SEMIBOLD,
    ANSI_CHARSET,
    OUT_DEFAULT_PRECIS,
    CLIP_DEFAULT_PRECIS,
    DEFAULT_QUALITY,
    FF_DONTCARE,
    CreateDIBSection,
    BITMAPINFO,
    BITMAPINFOHEADER,
    BI_RGB,
    DIB_RGB_COLORS,
    GetTextExtentPoint32W,
    RoundRect,
    CreateSolidBrush,
    CreatePen,
    PS_NULL,
};
use std::ffi::c_void;
use std::collections::VecDeque;
use crate::format::{ get_speed_parts, get_speed_color };

pub struct IconGenerator {
    history: VecDeque<u64>,
}

impl IconGenerator {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(32),
        }
    }

    pub fn generate(&mut self, speed: u64) -> Result<tray_icon::Icon, String> {
        if self.history.len() >= 32 {
            self.history.pop_front();
        }
        self.history.push_back(speed);

        let max_speed = *self.history.iter().max().unwrap_or(&1);
        let max_speed = if max_speed == 0 { 1 } else { max_speed };

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

            let hdc_screen = windows::Win32::Graphics::Gdi::GetDC(HWND(std::ptr::null_mut()));
            let hdc_mem = CreateCompatibleDC(hdc_screen);
            if hdc_mem.is_invalid() {
                return Err("CreateCompatibleDC failed".to_string());
            }

            let mut bits: *mut c_void = std::ptr::null_mut();
            let hbitmap = CreateDIBSection(
                hdc_mem,
                &bmi,
                DIB_RGB_COLORS,
                &mut bits,
                None,
                0
            ).map_err(|e| e.to_string())?;

            if bits.is_null() {
                return Err("CreateDIBSection returned null bits".to_string());
            }

            let old_bmp = SelectObject(hdc_mem, hbitmap);

            // Clear background (transparent)
            std::ptr::write_bytes(bits, 0, (width * height * 4) as usize);

            // Draw Rounded Background
            // Background: #0B0B0E -> 0x000E0B0B
            let bg_brush = CreateSolidBrush(windows::Win32::Foundation::COLORREF(0x000e0b0b));
            let pen = CreatePen(PS_NULL, 0, windows::Win32::Foundation::COLORREF(0));
            let old_brush = SelectObject(hdc_mem, bg_brush);
            let old_pen = SelectObject(hdc_mem, pen);

            RoundRect(hdc_mem, 0, 0, width, height, 6, 6);

            SelectObject(hdc_mem, old_brush);
            SelectObject(hdc_mem, old_pen);
            DeleteObject(bg_brush);
            DeleteObject(pen);

            SetBkMode(hdc_mem, TRANSPARENT);

            // Custom formatting for Icon (Integer only, max 3 digits)
            let (val_str, unit_str) = if speed < 1024 {
                (speed.to_string(), "B".to_string())
            } else if speed < 1024 * 1024 {
                ((speed / 1024).to_string(), "KB".to_string())
            } else if speed < 1024 * 1024 * 1024 {
                ((speed / (1024 * 1024)).to_string(), "MB".to_string())
            } else {
                ((speed / (1024 * 1024 * 1024)).to_string(), "GB".to_string())
            };

            let color_ref = get_speed_color(speed);
            SetTextColor(hdc_mem, windows::Win32::Foundation::COLORREF(color_ref));

            // Dynamic Font Size based on length
            let font_height = match val_str.len() {
                1 => -22,
                2 => -18,
                _ => -14,
            };

            // Draw Value (Top, Large)
            let hfont_val = CreateFontW(
                font_height,
                0,
                0,
                0,
                FW_SEMIBOLD.0 as i32,
                0,
                0,
                0,
                ANSI_CHARSET.0 as u32,
                OUT_DEFAULT_PRECIS.0 as u32,
                CLIP_DEFAULT_PRECIS.0 as u32,
                DEFAULT_QUALITY.0 as u32,
                FF_DONTCARE.0 as u32,
                windows::core::PCWSTR::from_raw(wide_string("Segoe UI").as_ptr())
            );
            let old_font = SelectObject(hdc_mem, hfont_val);

            let w_val = wide_string(&val_str);
            let mut size_val = SIZE::default();
            GetTextExtentPoint32W(hdc_mem, &w_val, &mut size_val);
            let x_val = (width - size_val.cx) / 2;
            
            // Adjust Y based on font size
            let y_val = match val_str.len() {
                1 => -5,
                2 => -3,
                _ => -1,
            };
            TextOutW(hdc_mem, x_val, y_val, &w_val);

            SelectObject(hdc_mem, old_font);
            DeleteObject(hfont_val);

            // Draw Unit (Bottom, Small)
            let hfont_unit = CreateFontW(
                -11,
                0,
                0,
                0,
                0, // FW_REGULAR (0 or 400)
                0,
                0,
                0,
                ANSI_CHARSET.0 as u32,
                OUT_DEFAULT_PRECIS.0 as u32,
                CLIP_DEFAULT_PRECIS.0 as u32,
                DEFAULT_QUALITY.0 as u32,
                FF_DONTCARE.0 as u32,
                windows::core::PCWSTR::from_raw(wide_string("Segoe UI").as_ptr())
            );
            SelectObject(hdc_mem, hfont_unit);

            let w_unit = wide_string(&unit_str);
            let mut size_unit = SIZE::default();
            GetTextExtentPoint32W(hdc_mem, &w_unit, &mut size_unit);
            let x_unit = (width - size_unit.cx) / 2;
            let y_unit = 15;
            TextOutW(hdc_mem, x_unit, y_unit, &w_unit);

            SelectObject(hdc_mem, old_font);
            DeleteObject(hfont_unit);

            // Fix Alpha channel
            // We drew a rounded rect on transparent background.
            // The pixels inside the rect are 0x00111111 (if we ignore alpha for a sec).
            // Text is drawn on top.
            // We need to set alpha to 255 for all non-transparent pixels.

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

            SelectObject(hdc_mem, old_bmp);
            DeleteObject(hbitmap);
            DeleteDC(hdc_mem);
            windows::Win32::Graphics::Gdi::ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);

            tray_icon::Icon::from_rgba(rgba, width as u32, height as u32).map_err(|e| e.to_string())
        }
    }
}

fn wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
