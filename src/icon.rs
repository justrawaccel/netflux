use windows::Win32::Foundation::{ HWND, RECT, POINT };
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
    HFONT,
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
use windows::Win32::UI::WindowsAndMessaging::{ CreateIconIndirect, DestroyIcon, HICON, ICONINFO };
use std::ffi::c_void;

pub struct IconGenerator {
    // Cache GDI objects if needed, but for MVP we can recreate.
}

impl IconGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate(&self, text: &str) -> Result<tray_icon::Icon, String> {
        unsafe {
            let width = 32;
            let height = 32;

            // 1. Create a DIB section to draw on
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // Top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB,
                    ..Default::default()
                },
                ..Default::default()
            };

            let hdc_screen = windows::Win32::Graphics::Gdi::GetDC(HWND(0));
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

            // 2. Clear background (transparent or color)
            // For tray icon, we want transparency.
            // However, GDI doesn't handle alpha channel text drawing easily.
            // Simple hack: Fill with black, draw white text, use as mask?
            // Or just draw text on a background color.
            // Let's try drawing white text on transparent background.
            // We need to manually clear the bits to 0 (transparent).
            std::ptr::write_bytes(bits, 0, (width * height * 4) as usize);

            // 3. Draw Text
            SetBkMode(hdc_mem, TRANSPARENT);
            SetTextColor(hdc_mem, windows::Win32::Foundation::COLORREF(0x00ffffff)); // White

            // Create Font
            let hfont = CreateFontW(
                -24, // Height (approx 10pt)
                0,
                0,
                0,
                FW_BOLD.0 as i32,
                0,
                0,
                0,
                ANSI_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                DEFAULT_QUALITY,
                FF_DONTCARE,
                windows::core::PCWSTR::from_raw(wide_string("Segoe UI").as_ptr())
            );
            let old_font = SelectObject(hdc_mem, hfont);

            let w_text = wide_string(text);
            // Center text roughly
            // For MVP, just draw at 0,0 or slightly offset
            TextOutW(hdc_mem, 0, 4, &w_text);

            // 4. Create Icon from Bitmap
            // We need a mask bitmap as well for CreateIconIndirect
            // Or we can use the alpha channel if we set it correctly.
            // GDI TextOut doesn't set alpha.
            // We need to fix alpha channel for pixels that have text.
            // Iterate pixels: if color != 0, set alpha to 255.
            let pixel_count = (width * height) as usize;
            let pixels = std::slice::from_raw_parts_mut(bits as *mut u32, pixel_count);
            for p in pixels.iter_mut() {
                if (*p & 0x00ffffff) != 0 {
                    *p |= 0xff000000;
                }
            }

            let mut icon_info = ICONINFO {
                fIcon: 1, // TRUE for icon
                xHotspot: 0,
                yHotspot: 0,
                hbmMask: hbitmap, // If we use alpha, mask can be same? No, mask should be monochrome.
                hbmColor: hbitmap,
            };

            // Create an empty mask for full alpha support
            let hbm_mask = CreateDIBSection(
                hdc_mem,
                &bmi,
                DIB_RGB_COLORS,
                &mut std::ptr::null_mut(),
                None,
                0
            ).unwrap();
            icon_info.hbmMask = hbm_mask;

            let hicon = CreateIconIndirect(&icon_info).map_err(|e| e.to_string())?;

            // Cleanup
            SelectObject(hdc_mem, old_font);
            DeleteObject(hfont);
            SelectObject(hdc_mem, old_bmp);
            DeleteObject(hbitmap);
            DeleteObject(hbm_mask);
            DeleteDC(hdc_mem);
            windows::Win32::Graphics::Gdi::ReleaseDC(HWND(0), hdc_screen);

            // Convert HICON to tray_icon::Icon
            // tray_icon::Icon::from_handle is not available?
            // tray_icon expects RGBA bytes.
            // Wait, tray_icon crate takes `Icon::from_rgba`.
            // So we don't need HICON! We just need the RGBA buffer.
            // We already have `pixels` (ARGB). We need to convert to RGBA.

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

            // We don't need CreateIconIndirect if we use tray_icon crate's builder.
            // But wait, the user plan said "CreateIconIndirect".
            // Maybe they want to use raw WinAPI for the tray?
            // "tray-icon (удобно)" implies using the crate.
            // The crate `tray-icon` takes `Icon` struct.

            DestroyIcon(hicon); // We don't need the HICON if we use the crate's buffer method.

            tray_icon::Icon::from_rgba(rgba, width as u32, height as u32).map_err(|e| e.to_string())
        }
    }
}

fn wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
