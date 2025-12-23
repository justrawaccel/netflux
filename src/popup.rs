use winit::window::{ Window, WindowBuilder, WindowLevel };
use winit::event_loop::EventLoopWindowTarget;
use winit::platform::windows::WindowBuilderExtWindows;
use windows::Win32::Foundation::{ HWND, RECT, COLORREF };
use windows::Win32::Graphics::Gdi::{
    GetDC,
    ReleaseDC,
    TextOutW,
    SetBkMode,
    SetTextColor,
    TRANSPARENT,
    SelectObject,
    CreateFontW,
    FW_BOLD,
    ANSI_CHARSET,
    OUT_DEFAULT_PRECIS,
    CLIP_DEFAULT_PRECIS,
    DEFAULT_QUALITY,
    FF_DONTCARE,
    DeleteObject,
    CreateSolidBrush,
    FillRect,
    FrameRect,
};
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW,
    SPI_GETWORKAREA,
    SPIF_SENDCHANGE,
};
use raw_window_handle::{ HasWindowHandle, RawWindowHandle };

pub struct Popup {
    window: Window,
    text: String,
}

impl Popup {
    pub fn new<T>(event_loop: &EventLoopWindowTarget<T>) -> Self {
        let window = WindowBuilder::new()
            .with_title("NetFlux Popup")
            .with_decorations(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_skip_taskbar(true)
            .with_resizable(false)
            .with_inner_size(winit::dpi::LogicalSize::new(220.0, 110.0))
            .with_visible(false)
            .build(event_loop)
            .expect("Failed to create popup window");

        Self {
            window,
            text: String::new(),
        }
    }

    pub fn toggle(&mut self) {
        if self.window.is_visible().unwrap_or(false) {
            self.window.set_visible(false);
        } else {
            self.reposition();
            self.window.set_visible(true);
            self.window.request_redraw();
        }
    }

    pub fn update_text(&mut self, text: String) {
        self.text = text;
        if self.window.is_visible().unwrap_or(false) {
            self.window.request_redraw();
        }
    }

    pub fn draw(&self) {
        let handle = self.window.window_handle().unwrap().as_raw();
        if let RawWindowHandle::Win32(handle) = handle {
            let hwnd = HWND(handle.hwnd.get() as _);
            unsafe {
                let hdc = GetDC(hwnd);

                let bg_color = COLORREF(0x001e1e1e);
                let text_color = COLORREF(0x00f0f0f0);
                let border_color = COLORREF(0x00444444);

                let rect = RECT { left: 0, top: 0, right: 220, bottom: 110 };

                let bg_brush = CreateSolidBrush(bg_color);
                FillRect(hdc, &rect, bg_brush);
                DeleteObject(bg_brush);

                let border_brush = CreateSolidBrush(border_color);
                FrameRect(hdc, &rect, border_brush);
                DeleteObject(border_brush);

                SetBkMode(hdc, TRANSPARENT);
                SetTextColor(hdc, text_color);

                let hfont = CreateFontW(
                    -18,
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
                let old_font = SelectObject(hdc, hfont);

                let lines: Vec<&str> = self.text.split('\n').collect();
                for (i, line) in lines.iter().enumerate() {
                    let w_line = wide_string(line);
                    TextOutW(hdc, 15, 15 + (i as i32) * 24, &w_line);
                }

                SelectObject(hdc, old_font);
                DeleteObject(hfont);
                ReleaseDC(hwnd, hdc);
            }
        }
    }

    fn reposition(&self) {
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
                let width = 220;
                let height = 110;
                let x = rect.right - width - 12;
                let y = rect.bottom - height - 12;
                self.window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
            }
        }
    }
}

fn wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
