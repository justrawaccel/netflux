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
    PS_SOLID,
    Polygon,
    Polyline,
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
    down_history: VecDeque<u64>,
    up_history: VecDeque<u64>,
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

        Self {
            window,
            text: String::new(),
            down_history: VecDeque::with_capacity(240),
            up_history: VecDeque::with_capacity(240),
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

        if self.down_history.len() >= 240 {
            self.down_history.pop_front();
        }
        self.down_history.push_back(down);

        if self.up_history.len() >= 240 {
            self.up_history.pop_front();
        }
        self.up_history.push_back(up);

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
                let height = 220;

                // Background: #0B0B0E -> 0x000E0B0B
                let bg_color = COLORREF(0x000e0b0b);

                let rect = RECT { left: 0, top: 0, right: width, bottom: height };

                let bg_brush = CreateSolidBrush(bg_color);
                FillRect(hdc, &rect, bg_brush);
                DeleteObject(bg_brush);

                // Parse stats
                let parts: Vec<&str> = self.text.split('\n').collect();
                let down_bps = parts.get(0).unwrap_or(&"0").parse::<u64>().unwrap_or(0);
                let up_bps = parts.get(1).unwrap_or(&"0").parse::<u64>().unwrap_or(0);

                // --- DOWNLOAD SECTION (Top) ---
                let max_down = *self.down_history.iter().max().unwrap_or(&1);
                let max_down = if max_down == 0 { 1 } else { max_down };

                let down_baseline = 100;
                let graph_height = 50.0;

                // Fill (Shadow)
                let down_fill_brush = CreateSolidBrush(COLORREF(0x001c390f)); // Dark Green
                let null_pen = CreatePen(PS_NULL, 0, COLORREF(0));
                let old_brush = SelectObject(hdc, down_fill_brush);
                let old_pen = SelectObject(hdc, null_pen);

                let mut points = Vec::with_capacity(self.down_history.len() + 2);
                points.push(POINT { x: 0, y: down_baseline });
                for (i, &val) in self.down_history.iter().enumerate() {
                    let x = i as i32;
                    let h = (((val as f64) / (max_down as f64)) * graph_height) as i32;
                    let y = down_baseline - h;
                    points.push(POINT { x, y });
                }
                points.push(POINT { x: self.down_history.len() as i32, y: down_baseline });
                Polygon(hdc, &points);

                SelectObject(hdc, old_brush);
                SelectObject(hdc, old_pen);
                DeleteObject(down_fill_brush);
                DeleteObject(null_pen);

                // Line (Bright)
                let down_pen = CreatePen(PS_SOLID, 2, COLORREF(0x005ec522)); // Bright Green
                let old_pen = SelectObject(hdc, down_pen);
                // Rebuild points without baseline anchors
                let mut line_points = Vec::with_capacity(self.down_history.len());
                for (i, &val) in self.down_history.iter().enumerate() {
                    let x = i as i32;
                    let h = (((val as f64) / (max_down as f64)) * graph_height) as i32;
                    let y = down_baseline - h;
                    line_points.push(POINT { x, y });
                }
                if !line_points.is_empty() {
                    Polyline(hdc, &line_points);
                }
                SelectObject(hdc, old_pen);
                DeleteObject(down_pen);

                // --- UPLOAD SECTION (Bottom) ---
                let max_up = *self.up_history.iter().max().unwrap_or(&1);
                let max_up = if max_up == 0 { 1 } else { max_up };

                let up_baseline = 210;

                // Fill (Shadow)
                let up_fill_brush = CreateSolidBrush(COLORREF(0x004a184a)); // Dark Pink
                let null_pen = CreatePen(PS_NULL, 0, COLORREF(0));
                let old_brush = SelectObject(hdc, up_fill_brush);
                let old_pen = SelectObject(hdc, null_pen);

                let mut points = Vec::with_capacity(self.up_history.len() + 2);
                points.push(POINT { x: 0, y: up_baseline });
                for (i, &val) in self.up_history.iter().enumerate() {
                    let x = i as i32;
                    let h = (((val as f64) / (max_up as f64)) * graph_height) as i32;
                    let y = up_baseline - h;
                    points.push(POINT { x, y });
                }
                points.push(POINT { x: self.up_history.len() as i32, y: up_baseline });
                Polygon(hdc, &points);

                SelectObject(hdc, old_brush);
                SelectObject(hdc, old_pen);
                DeleteObject(up_fill_brush);
                DeleteObject(null_pen);

                // Line (Bright)
                let up_pen = CreatePen(PS_SOLID, 2, COLORREF(0x00ff55ff)); // Bright Pink
                let old_pen = SelectObject(hdc, up_pen);
                let mut line_points = Vec::with_capacity(self.up_history.len());
                for (i, &val) in self.up_history.iter().enumerate() {
                    let x = i as i32;
                    let h = (((val as f64) / (max_up as f64)) * graph_height) as i32;
                    let y = up_baseline - h;
                    line_points.push(POINT { x, y });
                }
                if !line_points.is_empty() {
                    Polyline(hdc, &line_points);
                }
                SelectObject(hdc, old_pen);
                DeleteObject(up_pen);

                // --- TEXT ---
                SetBkMode(hdc, TRANSPARENT);

                // Fonts
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

                // Download Text
                SetTextColor(hdc, COLORREF(0x00aaaaaa)); // Gray
                let old_font = SelectObject(hdc, hfont_label);
                TextOutW(hdc, 16, 12, &wide_string("↓ DOWNLOAD"));

                SetTextColor(hdc, COLORREF(0x00ffffff)); // White
                SelectObject(hdc, hfont_val);
                TextOutW(hdc, 16, 30, &wide_string(&format_speed_full(down_bps)));

                // Upload Text
                SetTextColor(hdc, COLORREF(0x00aaaaaa)); // Gray
                SelectObject(hdc, hfont_label);
                TextOutW(hdc, 16, 120, &wide_string("↑ UPLOAD"));

                SetTextColor(hdc, COLORREF(0x00ffffff)); // White
                SelectObject(hdc, hfont_val);
                TextOutW(hdc, 16, 138, &wide_string(&format_speed_full(up_bps)));

                SelectObject(hdc, old_font);
                DeleteObject(hfont_label);
                DeleteObject(hfont_val);

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
                let height = 220;
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
