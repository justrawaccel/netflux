use winit::window::{ Window, WindowBuilder, WindowLevel };
use winit::event_loop::EventLoopWindowTarget;
use winit::platform::windows::WindowBuilderExtWindows;
use windows::Win32::Foundation::{ HWND, RECT, COLORREF, POINT };
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
    FW_SEMIBOLD,
    ANSI_CHARSET,
    OUT_DEFAULT_PRECIS,
    CLIP_DEFAULT_PRECIS,
    DEFAULT_QUALITY,
    FF_DONTCARE,
    DeleteObject,
    CreateSolidBrush,
    FillRect,
    FrameRect,
    CreatePen,
    PS_NULL,
    Polygon,
};
use crate::format::{ format_speed_full, get_speed_color };
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW,
    SPI_GETWORKAREA,
    SPIF_SENDCHANGE,
};
use raw_window_handle::{ HasWindowHandle, RawWindowHandle };
use std::collections::VecDeque;

pub struct Popup {
    window: Window,
    text: String,
    history: VecDeque<u64>,
}

impl Popup {
    pub fn new<T>(event_loop: &EventLoopWindowTarget<T>) -> Self {
        let window = WindowBuilder::new()
            .with_title("NetFlux Popup")
            .with_decorations(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_skip_taskbar(true)
            .with_resizable(false)
            .with_inner_size(winit::dpi::LogicalSize::new(240.0, 140.0))
            .with_visible(false)
            .build(event_loop)
            .expect("Failed to create popup window");

        Self {
            window,
            text: String::new(),
            history: VecDeque::with_capacity(240),
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

    pub fn update(&mut self, down: u64, up: u64, _text: String) {
        // We construct the text inside draw() now
        self.text = format!("{}\n{}", down, up);
        if self.history.len() >= 240 {
            self.history.pop_front();
        }
        self.history.push_back(down);

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

                let width = 240;
                let height = 140;

                let bg_color = COLORREF(0x00100f0f); // #0F0F10
                let border_color = COLORREF(0x00333333);

                let rect = RECT { left: 0, top: 0, right: width, bottom: height };

                let bg_brush = CreateSolidBrush(bg_color);
                FillRect(hdc, &rect, bg_brush);
                DeleteObject(bg_brush);

                // Parse stats
                let parts: Vec<&str> = self.text.split('\n').collect();
                let down_bps = parts.get(0).unwrap_or(&"0").parse::<u64>().unwrap_or(0);
                let up_bps = parts.get(1).unwrap_or(&"0").parse::<u64>().unwrap_or(0);

                // Draw Graph (Filled Polygon)
                let max_val = *self.history.iter().max().unwrap_or(&1);
                let max_val = if max_val == 0 { 1 } else { max_val };

                // Graph Color based on speed
                let graph_color_ref = get_speed_color(down_bps);
                let graph_brush = CreateSolidBrush(
                    windows::Win32::Foundation::COLORREF(graph_color_ref)
                );
                let null_pen = CreatePen(PS_NULL, 0, windows::Win32::Foundation::COLORREF(0));

                let old_brush = SelectObject(hdc, graph_brush);
                let old_pen = SelectObject(hdc, null_pen);

                let mut points = Vec::with_capacity(self.history.len() + 2);
                points.push(POINT { x: 0, y: height }); // Start bottom-left

                for (i, &val) in self.history.iter().enumerate() {
                    let x = i as i32;
                    // Scale height to fit in bottom 60px
                    let h = (((val as f64) / (max_val as f64)) * 60.0) as i32;
                    let y = height - h;
                    points.push(POINT { x, y });
                }

                points.push(POINT { x: self.history.len() as i32, y: height }); // End bottom-right

                Polygon(hdc, &points);

                SelectObject(hdc, old_brush);
                SelectObject(hdc, old_pen);
                DeleteObject(graph_brush);
                DeleteObject(null_pen);

                // Draw Text
                SetBkMode(hdc, TRANSPARENT);

                // 1. Download Label
                SetTextColor(hdc, COLORREF(0x00aaaaaa)); // Gray
                let hfont_label = CreateFontW(
                    -12,
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
                let old_font = SelectObject(hdc, hfont_label);
                TextOutW(hdc, 16, 12, &wide_string("↓ Download"));

                // 2. Download Value
                SetTextColor(hdc, COLORREF(0x00ffffff)); // White
                let hfont_val = CreateFontW(
                    -24,
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
                SelectObject(hdc, hfont_val);
                TextOutW(hdc, 16, 30, &wide_string(&format_speed_full(down_bps)));

                // 3. Upload Section (Bottom Left)
                SetTextColor(hdc, COLORREF(0x00aaaaaa)); // Gray
                SelectObject(hdc, hfont_label);
                TextOutW(hdc, 16, height - 30, &wide_string("↑ Upload"));

                SetTextColor(hdc, COLORREF(0x00ffffff)); // White
                TextOutW(hdc, 80, height - 30, &wide_string(&format_speed_full(up_bps)));

                SelectObject(hdc, old_font);
                DeleteObject(hfont_label);
                DeleteObject(hfont_val);

                // Border
                let border_brush = CreateSolidBrush(border_color);
                FrameRect(hdc, &rect, border_brush);
                DeleteObject(border_brush);

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
                let width = 240;
                let height = 140;
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
