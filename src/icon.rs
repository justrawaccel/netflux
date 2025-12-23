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
};
use std::ffi::c_void;

pub struct IconGenerator {}

impl IconGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate(&self, text: &str) -> Result<tray_icon::Icon, String> {
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
            let mut bits: *mut c_void = std::ptr::null_mut();
            let hbitmap = CreateDIBSection(
                hdc_mem,
                &bmi,
                DIB_RGB_COLORS,
                &mut bits,
                None,
                0
            ).map_err(|e| e.to_string())?;

            let old_bmp = SelectObject(hdc_mem, hbitmap);

            std::ptr::write_bytes(bits, 0, (width * height * 4) as usize);

            SetBkMode(hdc_mem, TRANSPARENT);
            SetTextColor(hdc_mem, windows::Win32::Foundation::COLORREF(0x00ffffff));

            let hfont = CreateFontW(
                -22,
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

            let w_text = wide_string(text);
            TextOutW(hdc_mem, 0, 4, &w_text);

            let pixel_count = (width * height) as usize;
            let pixels = std::slice::from_raw_parts_mut(bits as *mut u32, pixel_count);
            for p in pixels.iter_mut() {
                if (*p & 0x00ffffff) != 0 {
                    *p |= 0xff000000;
                }
            }

            SelectObject(hdc_mem, old_font);
            DeleteObject(hfont);
            SelectObject(hdc_mem, old_bmp);
            DeleteObject(hbitmap);
            DeleteDC(hdc_mem);
            windows::Win32::Graphics::Gdi::ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);

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

            tray_icon::Icon::from_rgba(rgba, width as u32, height as u32).map_err(|e| e.to_string())
        }
    }
}

fn wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
