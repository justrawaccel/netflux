use windows::Win32::Foundation::HWND;
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
    SIZE,
};
use std::ffi::c_void;
use std::collections::VecDeque;
use crate::format::format_speed;

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

            std::ptr::write_bytes(bits, 0, (width * height * 4) as usize);

            // Draw Graph directly on pixels
            let pixels = std::slice::from_raw_parts_mut(
                bits as *mut u32,
                (width * height) as usize
            );

            // Draw history graph
            for (i, &val) in self.history.iter().enumerate() {
                if i >= (width as usize) {
                    break;
                }

                let bar_height = (
                    ((val as f64) / (max_speed as f64)) *
                    (height as f64)
                ).ceil() as i32;
                let bar_height = bar_height.clamp(0, height);

                for y in height - bar_height..height {
                    let idx = (y * width + (i as i32)) as usize;
                    if idx < pixels.len() {
                        // Greenish color for graph: 0xAA4CAF50 (ARGB)
                        // GDI uses BGRA in memory usually, but let's check.
                        // 0xAARRGGBB -> 0xAA50AF4C (Little Endian u32)
                        // Let's use a solid color for now, we fix alpha later.
                        // 0x00FF00 (Green)
                        pixels[idx] = 0x0000ff00;
                    }
                }
            }

            SetBkMode(hdc_mem, TRANSPARENT);
            SetTextColor(hdc_mem, windows::Win32::Foundation::COLORREF(0x00ffffff));

            let hfont = CreateFontW(
                -11,
                0,
                0,
                0,
                FW_BOLD.0 as i32,
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
            let old_font = SelectObject(hdc_mem, hfont);

            let text = format_speed(speed);
            let w_text = wide_string(&text);

            // Measure text to center it
            let mut size = SIZE::default();
            GetTextExtentPoint32W(hdc_mem, &w_text, &mut size);
            let x = (width - size.cx) / 2;
            let y = (height - size.cy) / 2 - 2; // Slightly higher

            TextOutW(hdc_mem, x, y, &w_text);

            let pixel_count = (width * height) as usize;
            let pixels = std::slice::from_raw_parts_mut(bits as *mut u32, pixel_count);
            for p in pixels.iter_mut() {
                if (*p & 0x00ffffff) != 0 {
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

            SelectObject(hdc_mem, old_font);
            DeleteObject(hfont);
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
