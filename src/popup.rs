use winit::window::{ Window, WindowBuilder };
use winit::event_loop::EventLoop;
use winit::platform::windows::WindowBuilderExtWindows;
use windows::Win32::Foundation::{ HWND, RECT, POINT };
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
    InvalidateRect,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics,
    SM_CXSCREEN,
    SM_CYSCREEN,
    SW_HIDE,
    SW_SHOW,
    ShowWindow,
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
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let window = WindowBuilder::new()
            .with_title("NetFlux Popup")
            .with_decorations(false)
            .with_always_on_top(true)
            .with_skip_taskbar(true)
            .with_resizable(false)
            .with_inner_size(winit::dpi::LogicalSize::new(200.0, 100.0))
            .with_visible(false) // Start hidden
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

                // Clear background (simple fill)
                let mut rect = RECT { left: 0, top: 0, right: 200, bottom: 100 };
                windows::Win32::Graphics::Gdi::FillRect(
                    hdc,
                    &rect,
                    windows::Win32::Graphics::Gdi::HBRUSH(
                        windows::Win32::Graphics::Gdi::GetStockObject(
                            windows::Win32::Graphics::Gdi::WHITE_BRUSH
                        ).0
                    )
                );

                SetBkMode(hdc, TRANSPARENT);
                SetTextColor(hdc, windows::Win32::Foundation::COLORREF(0x00000000)); // Black text

                let hfont = CreateFontW(
                    -16,
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
                let old_font = SelectObject(hdc, hfont);

                let lines: Vec<&str> = self.text.split('\n').collect();
                for (i, line) in lines.iter().enumerate() {
                    let w_line = wide_string(line);
                    TextOutW(hdc, 10, 10 + (i as i32) * 20, &w_line);
                }

                SelectObject(hdc, old_font);
                DeleteObject(hfont);
                ReleaseDC(hwnd, hdc);
            }
        }
    }

    fn reposition(&self) {
        // Position bottom-right of work area
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
                let width = 200;
                let height = 100;
                let x = rect.right - width - 10;
                let y = rect.bottom - height - 10;
                self.window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
            }
        }
    }
}

fn wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
