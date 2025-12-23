pub fn format_speed_full(bytes_per_sec: u64) -> String {
	if bytes_per_sec < 1024 {
		format!("{} B/s", bytes_per_sec)
	} else if bytes_per_sec < 1024 * 1024 {
		format!("{:.1} KB/s", (bytes_per_sec as f64) / 1024.0)
	} else if bytes_per_sec < 1024 * 1024 * 1024 {
		format!("{:.1} MB/s", (bytes_per_sec as f64) / (1024.0 * 1024.0))
	} else {
		format!("{:.1} GB/s", (bytes_per_sec as f64) / (1024.0 * 1024.0 * 1024.0))
	}
}

pub fn format_speed_compact(bytes_per_sec: u64) -> (String, String) {
	if bytes_per_sec < 1024 {
		(format!("{}", bytes_per_sec), "B".to_string())
	} else if bytes_per_sec < 1024 * 1024 {
		(format!("{:.0}", (bytes_per_sec as f64) / 1024.0), "KB".to_string())
	} else if bytes_per_sec < 1024 * 1024 * 1024 {
		(format!("{:.1}", (bytes_per_sec as f64) / (1024.0 * 1024.0)), "MB".to_string())
	} else {
		(format!("{:.1}", (bytes_per_sec as f64) / (1024.0 * 1024.0 * 1024.0)), "GB".to_string())
	}
}
