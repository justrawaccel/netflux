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
        (format!("{}", bps), "B".to_string())
    } else if bps < MB {
        let val = (bps as f64) / (KB as f64);
        (format!("{:.1}", val), "K".to_string())
    } else if bps < GB {
        let val = (bps as f64) / (MB as f64);
        (format!("{:.1}", val), "M".to_string())
    } else {
        let val = (bps as f64) / (GB as f64);
        (format!("{:.1}", val), "G".to_string())
    }
}

pub fn get_speed_color(bps: u64) -> u32 {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;

    // Colors in 0x00BBGGRR format
    if bps < 100 * KB {
        0x00afa39c // Idle: #9CA3AF (Gray)
    } else if bps < 5 * MB {
        0x005ec522 // Load: #22C55E (Green)
    } else if bps < 20 * MB {
        0x0008b3ea // Active: #EAB308 (Yellow)
    } else {
        0x004444ef // Peak: #EF4444 (Red)
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
