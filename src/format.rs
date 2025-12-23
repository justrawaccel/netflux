pub fn format_speed(bps: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;

    if bps < MB {
        // < 1 MB/s -> ###K
        let k = (bps as f64) / (KB as f64);
        format!("{:.0}K", k)
    } else {
        // >= 1 MB/s -> #.#M or ##M
        let m = (bps as f64) / (MB as f64);
        if m < 10.0 {
            format!("{:.1}M", m)
        } else {
            format!("{:.0}M", m)
        }
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
