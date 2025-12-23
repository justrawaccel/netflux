# NetFlux

Network speed monitor for Windows 11 System Tray.

## Prerequisites

- Rust (stable)
- Visual Studio Build Tools with "Desktop development with C++" workload (for `link.exe`)

## Build & Run

```powershell
cargo run --release
```

## Features

- Tray icon showing Download speed (e.g. "12M", "850K").
- Tooltip showing Down/Up speed.
- Left-click popup showing detailed stats.
- Right-click menu to Exit.
- Updates every 1 second.
