use windows::Win32::System::Registry::{
	RegCreateKeyExW,
	RegSetValueExW,
	HKEY_CURRENT_USER,
	KEY_WRITE,
	REG_SZ,
	REG_OPTION_NON_VOLATILE,
	HKEY,
};
use windows::core::PCWSTR;

pub fn enable_autostart() {
	unsafe {
		let mut hkey = HKEY::default();
		let subkey = wide_string("Software\\Microsoft\\Windows\\CurrentVersion\\Run");

		if
			RegCreateKeyExW(
				HKEY_CURRENT_USER,
				PCWSTR::from_raw(subkey.as_ptr()),
				0,
				None,
				REG_OPTION_NON_VOLATILE,
				KEY_WRITE,
				None,
				&mut hkey,
				None
			).is_ok()
		{
			let exe_path = std::env::current_exe().unwrap_or_default();
			let path_str = exe_path.to_str().unwrap_or_default();
			let path_wide = wide_string(path_str);

			let val_name = wide_string("NetFlux");
			let _ = RegSetValueExW(
				hkey,
				PCWSTR::from_raw(val_name.as_ptr()),
				0,
				REG_SZ,
				Some(std::slice::from_raw_parts(path_wide.as_ptr() as *const u8, path_wide.len() * 2))
			);
		}
	}
}

fn wide_string(s: &str) -> Vec<u16> {
	s.encode_utf16().chain(std::iter::once(0)).collect()
}
