use windows::Win32::Foundation::COLORREF;
use windows::Win32::Graphics::Gdi::{
	CreateSolidBrush,
	CreatePen,
	CreateFontW,
	SelectObject,
	DeleteObject,
	HBRUSH,
	HGDIOBJ,
	HDC,
	PEN_STYLE,
	ANSI_CHARSET,
	OUT_DEFAULT_PRECIS,
	CLIP_DEFAULT_PRECIS,
	DEFAULT_QUALITY,
	FF_DONTCARE,
};
use windows::core::PCWSTR;

pub struct AutoGdiObject {
	handle: HGDIOBJ,
}

impl AutoGdiObject {
	pub fn new(handle: HGDIOBJ) -> Self {
		Self { handle }
	}

	pub fn handle(&self) -> HGDIOBJ {
		self.handle
	}

	pub fn as_brush(&self) -> HBRUSH {
		HBRUSH(self.handle.0)
	}
}

impl Drop for AutoGdiObject {
	fn drop(&mut self) {
		unsafe {
			if !self.handle.is_invalid() {
				let _ = DeleteObject(self.handle);
			}
		}
	}
}

pub fn create_solid_brush(color: u32) -> AutoGdiObject {
	unsafe {
		let handle = CreateSolidBrush(COLORREF(color));
		AutoGdiObject::new(HGDIOBJ(handle.0 as _))
	}
}

pub fn create_pen(style: PEN_STYLE, width: i32, color: u32) -> AutoGdiObject {
	unsafe {
		let handle = CreatePen(style, width, COLORREF(color));
		AutoGdiObject::new(HGDIOBJ(handle.0 as _))
	}
}

pub fn create_font(height: i32, weight: i32, face: &str) -> AutoGdiObject {
	unsafe {
		let face_wide: Vec<u16> = face.encode_utf16().chain(std::iter::once(0)).collect();
		let handle = CreateFontW(
			height,
			0,
			0,
			0,
			weight,
			0,
			0,
			0,
			ANSI_CHARSET.0 as u32,
			OUT_DEFAULT_PRECIS.0 as u32,
			CLIP_DEFAULT_PRECIS.0 as u32,
			DEFAULT_QUALITY.0 as u32,
			FF_DONTCARE.0 as u32,
			PCWSTR::from_raw(face_wide.as_ptr())
		);
		AutoGdiObject::new(HGDIOBJ(handle.0 as _))
	}
}

pub struct DcScope<'a> {
	hdc: HDC,
	saved_objects: Vec<HGDIOBJ>,
	_marker: std::marker::PhantomData<&'a HDC>,
}

impl<'a> DcScope<'a> {
	pub fn new(hdc: HDC) -> Self {
		Self {
			hdc,
			saved_objects: Vec::new(),
			_marker: std::marker::PhantomData,
		}
	}

	pub fn select(&mut self, obj: &AutoGdiObject) {
		unsafe {
			let old = SelectObject(self.hdc, obj.handle());
			self.saved_objects.push(old);
		}
	}
}

impl<'a> Drop for DcScope<'a> {
	fn drop(&mut self) {
		unsafe {
			for old in self.saved_objects.iter().rev() {
				let _ = SelectObject(self.hdc, *old);
			}
		}
	}
}
