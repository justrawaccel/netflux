pub fn format_speed(bps: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;

    if bps < MB {
        let k = (bps as f64) / (KB as f64);
        format!("{:.0}K", k)
    } else {
        let m = (bps as f64) / (MB as f64);
        if m < 10.0 {
            format!("{:.1}M", m)
        } else {
            format!("{:.0}M", m)
        }
    }
}

pub fn get_speed_parts(bps: u64) -> (String, String) {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;

    if bps < KB {
        (format!("{}", bps), "B/s".to_string())
    } else if bps < MB {
        let val = (bps as f64) / (KB as f64);
        (format!("{:.1}", val), "KB/s".to_string())
    } else if bps < GB {
        let val = (bps as f64) / (MB as f64);
        (format!("{:.1}", val), "MB/s".to_string())
    } else {
        let val = (bps as f64) / (GB as f64);
        (format!("{:.1}", val), "GB/s".to_string())
    }
}

pub fn get_speed_color(bps: u64) -> u32 {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;

    // Colors in 0x00BBGGRR format
    if bps < 100 * KB {
        0x00aaaaaa // Gray
    } else if bps < 1 * MB {
        0x0050af4c // Green
    } else if bps < 10 * MB {
        0x0000ff00 // Bright Green
    } else if bps < 50 * MB {
        0x0000ffff // Yellow (GDI: BBGGRR -> 00FFFF is Yellow? No. R=FF, G=FF, B=00 -> 0000FFFF)
    } else {
        0x000000ff // Red (R=FF, G=00, B=00 -> 000000FF)
    }
}

pub fn format_speed_full(bps: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;

    if bps < KB {
        format!("{} B/s", bps)
    } else if bps < MB {
        format!("{:.1} KB/s", (bps as f64) / (KB as f64))
    } else if bps < GB {
        format!("{:.1} MB/s", (bps as f64) / (MB as f64))
    } else {
        format!("{:.1} GB/s", (bps as f64) / (GB as f64))
    }
}
