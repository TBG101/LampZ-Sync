# LampZ Sync

LampZ Sync is a Windows desktop application that mirrors the colors on a selected monitor to a compatible Tuya lamp over the local network. It captures the screen locally, converts sampled frames into HSV color data, and only sends meaningful changes to the lamp so the ambient lighting follows the display without unnecessary network traffic.

The app runs quietly in the system tray and remembers the configured lamp, monitor, and visual response settings.

> **Project status:** early development (`0.1.0`). Expect rough edges and breaking changes while the device protocol and user experience continue to evolve.

## Features

- Connect to a lamp using its device ID, local IP address, and local key.
- Select which connected monitor supplies the ambient color.
- Map one or more capture regions on the selected monitor, with move, resize, delete, and snapping controls.
- Tune polling frequency, gamma correction, and color-change thresholds.
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
4. Adjust the visual response settings if needed.
5. Choose **Connect lamp**, then save the tuning and capture-region settings.

## Useful Commands

```powershell
# Start the Vite frontend only
bun run dev

# Type-check and build the frontend
bun run build

# Build a native Tauri bundle
bun run tauri build

# Check and compile the Rust backend
cargo check --manifest-path src-tauri/Cargo.toml
```

## How It Works

The application is split into a React interface and a Rust backend:

- `src/` contains the React and TypeScript settings UI.
- `src-tauri/src/screen_capture.rs` captures frames from the selected Windows monitor.
- `src-tauri/src/color.rs` and `src-tauri/src/detector.rs` process frames and detect useful color changes.
- `src-tauri/src/lamp.rs` sends local Tuya commands to the lamp.
- `src-tauri/src/capture_thread.rs` and `src-tauri/src/lamp_thread.rs` coordinate screen sampling and lamp updates.
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
