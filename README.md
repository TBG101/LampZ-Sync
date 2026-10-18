# LampZ Sync

[![Tauri](https://img.shields.io/badge/Tauri-2-24C8D8?logo=tauri&logoColor=white)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-090909?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/Platform-Windows-0078D6?logo=windows&logoColor=white)](https://www.microsoft.com/windows/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

LampZ Sync is a Windows desktop application that mirrors the colors on a selected monitor to a compatible Tuya lamp over the local network. It captures the screen locally, converts sampled frames into HSV color data, and only sends meaningful changes to the lamp so the ambient lighting follows the display without unnecessary network traffic.

The app runs quietly in the system tray and remembers the configured lamp, monitor, and visual response settings.

## Features

- Connect to a lamp using its device ID, local IP address, and local key.
- Select which connected monitor supplies the ambient color.
- Map one or more capture regions on the selected monitor, with move, resize, delete, and snapping controls.
- Tune polling frequency, gamma correction, and color-change thresholds.
- Switch between **Sync** and **Static color** modes, and enable or disable syncing on the fly.
- Expose an embedded MCP server so external agents can drive the lamp.
- Persist configuration between launches.
- Run from the Windows system tray and hide the main window without stopping synchronization.
- Use local screen capture and local-network lamp control; no cloud service is required by LampZ Sync.

## Requirements

LampZ Sync currently targets **Windows** because screen capture is implemented with Windows DXGI Desktop Duplication.

For development, install:

- [Windows 10 or later](https://www.microsoft.com/windows/)
- [Rust](https://www.rust-lang.org/tools/install) with the MSVC toolchain
- [Node.js](https://nodejs.org/) and [Bun](https://bun.sh/)
- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the Desktop development with C++ workload
- [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)

You will also need a compatible Tuya lamp that supports local control and its device ID, local IP address, and local key. LampZ Sync does not discover or recover local keys for you.

## Getting Started

Clone the repository and install the frontend dependencies:

```powershell
git clone https://github.com/TBG101/LampZ-Sync.git
cd LampZ-Sync
bun install
```

Start the Tauri development application:

```powershell
bun run tauri dev
```

In the app:

1. Enter the lamp ID, local IP address, and local key.
2. Select the monitor to sample.
3. Draw one or more capture regions in the **Capture regions** panel. Use **Snap** to align region edges to the 5% grid or monitor boundaries.
4. Choose **Sync** or **Static color** mode. In static mode, pick a color and brightness instead of following the screen.
5. Adjust the visual response settings if needed.
6. Choose **Connect lamp**, enable syncing, then save the tuning and capture-region settings.

## Useful Commands

```powershell
# Start the Vite frontend only
bun run dev

# Type-check and build the frontend
bun run build

# Regenerate TypeScript types from the Rust definitions
bun run bindings

# Build a native Tauri bundle
bun run tauri build

# Check and compile the Rust backend
cargo check --manifest-path src-tauri/Cargo.toml
```

## MCP Server

LampZ Sync embeds an [MCP](https://modelcontextprotocol.io) server so external agents can control the lamp. It is a **Streamable HTTP** server bound to `127.0.0.1` only and is disabled by default; enable it under the MCP settings and optionally set a bearer token.

Exposed tools are lighting-focused (`get_lamp_state`, `set_color`, `set_brightness`, `set_mode`, `set_power`, `set_sync_enabled`). They call the same config helpers as the UI, so every change is persisted and picked up by the worker threads.

## How It Works

The application is split into a React interface and a Rust backend:

- `src/` contains the React and TypeScript settings UI.
- `src-tauri/src/config.rs` defines the persisted `Config` schema; `app_state.rs` holds the shared `RwLock<Config>`.
- `src-tauri/src/screen_capture.rs` captures frames from the selected Windows monitor.
- `src-tauri/src/color.rs` and `src-tauri/src/detector.rs` process frames and detect useful color changes.
- `src-tauri/src/lamp.rs` sends local Tuya commands to the lamp.
- `src-tauri/src/capture_thread.rs` and `src-tauri/src/lamp_thread.rs` coordinate screen sampling and lamp updates.
- `src-tauri/src/commands.rs` exposes Tauri commands and is the single path that mutates, saves, and publishes config changes.
- `src-tauri/src/mcp.rs` runs the embedded MCP server.
- `src-tauri/src/app_store.rs` persists application settings through the Tauri store plugin.

The frontend communicates with the backend through Tauri commands. Runtime work stays in Rust threads so screen capture and network communication do not block the interface.

## Contributing

Contributions, bug reports, and device compatibility notes are welcome. Before opening a pull request:

1. Create a focused branch from the latest `main` branch.
2. Make the smallest change that addresses the issue.
3. Run `bun run build` and `cargo check --manifest-path src-tauri/Cargo.toml`.
4. Describe the Windows version, lamp model, and reproduction steps when reporting device or capture issues.

Never commit a real lamp local key, private IP address, or other credentials. Use placeholders in examples and test data.

## Links

- [Repository](https://github.com/TBG101/LampZ-Sync)
- [Issues](https://github.com/TBG101/LampZ-Sync/issues)
