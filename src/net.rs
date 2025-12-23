use std::time::Instant;
use windows::Win32::NetworkManagement::IpHelper::{
    GetIfTable2,
    MIB_IF_ROW2,
    MIB_IF_TABLE2,
    FreeMibTable,
    IF_TYPE_SOFTWARE_LOOPBACK,
};
use windows::Win32::NetworkManagement::Ndis::{ IfOperStatusUp, NET_LUID_LH };
use windows::Win32::Foundation::NO_ERROR;

pub struct NetStats {
    pub down_bps: u64,
    pub up_bps: u64,
    pub interface_name: String,
}

struct InterfaceSnapshot {
    luid: NET_LUID_LH,
    in_octets: u64,
    out_octets: u64,
    timestamp: Instant,
}

pub struct NetMonitor {
    last_snapshot: Option<InterfaceSnapshot>,
}

impl NetMonitor {
    pub fn new() -> Self {
        Self { last_snapshot: None }
    }

    pub fn tick(&mut self) -> Option<NetStats> {
        unsafe {
            let mut table_ptr: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
            if GetIfTable2(&mut table_ptr) != NO_ERROR {
                return None;
            }
            let table = &*table_ptr;

            let mut best_iface: Option<&MIB_IF_ROW2> = None;
            let mut max_bytes = 0;

            let rows = std::slice::from_raw_parts(table.Table.as_ptr(), table.NumEntries as usize);

            for row in rows {
                if row.OperStatus == IfOperStatusUp && row.Type != IF_TYPE_SOFTWARE_LOOPBACK {
                    let total = row.InOctets + row.OutOctets;
                    if total > max_bytes {
                        max_bytes = total;
                        best_iface = Some(row);
                    }
                }
            }

            let result = if let Some(row) = best_iface {
                let now = Instant::now();
                let current_in = row.InOctets;
                let current_out = row.OutOctets;

                let stats = if let Some(last) = &self.last_snapshot {
                    if last.luid.Value == row.InterfaceLuid.Value {
                        let dt = now.duration_since(last.timestamp).as_secs_f64();
                        if dt > 0.0 {
                            let down = ((current_in.saturating_sub(last.in_octets) as f64) /
                                dt) as u64;
                            let up = ((current_out.saturating_sub(last.out_octets) as f64) /
                                dt) as u64;

                            let name = String::from_utf16_lossy(&row.Alias)
                                .trim_matches(char::from(0))
                                .to_string();

                            Some(NetStats {
                                down_bps: down,
                                up_bps: up,
                                interface_name: name,
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                self.last_snapshot = Some(InterfaceSnapshot {
                    luid: row.InterfaceLuid,
                    in_octets: current_in,
                    out_octets: current_out,
                    timestamp: now,
                });

                stats
            } else {
                None
            };

            FreeMibTable(table_ptr as *const _);
            result
        }
    }
}
