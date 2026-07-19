# ns-download

A cross-platform download manager built with Tauri, Vue 3, and Rust. Successor to FluxDown, supporting multi-protocol downloads with a modern desktop UI.

## Features

- **Multi-protocol**: HTTP/HTTPS, FTP, BitTorrent, ed2k, HLS (m3u8), DASH (mpd)
- **Concurrent downloads**: Configurable concurrency and per-task segment count
- **BT features**: DHT, PEX, magnet links, tracker subscriptions, encryption
- **ed2k**: Server-based source discovery, Kad network, LowID callback, UPnP port mapping
- **HLS/DASH**: Segment-wise parallel downloading, TS→MP4 remux (via ffmpeg), variant selection
- **Proxy support**: SOCKS4/5, HTTP CONNECT per-task or global
- **Speed limiting**: Global and per-task bandwidth control
- **Plugin system**: JavaScript (QuickJS) extensions for URL resolution
- **Database**: SQLite/Postgres via sqlx for task persistence and resume
- **Cross-platform**: macOS, Linux, Windows, Android, iOS

## Architecture

```
ns-download/
├── src/                    # Vue 3 frontend (Tauri webview)
│   ├── views/              # TasksView, SettingsView
│   ├── components/         # Sidebar, StatusBar, TaskList, NewDownloadDialog
│   └── lib/                # Pinia store, Tauri invoke wrappers
├── src-tauri/              # Tauri 2 desktop shell (Rust)
│   └── src/
│       ├── commands.rs     # Tauri IPC commands
│       └── lib.rs          # App setup, plugins, state
├── native/
│   ├── engine/             # Core download engine (~75K lines)
│   │   ├── src/
│   │   │   ├── download_manager.rs  # Task orchestration
│   │   │   ├── downloader.rs        # HTTP client, redirect handling
│   │   │   ├── db.rs               # SQLite/Postgres persistence
│   │   │   ├── bt_downloader.rs    # BitTorrent via librqbit
│   │   │   ├── ftp_downloader.rs   # FTP via suppaftp
│   │   │   ├── hls_downloader.rs   # HLS streaming
│   │   │   ├── dash_downloader.rs  # DASH streaming
│   │   │   ├── ed2k/              # ed2k protocol (client, Kad, server)
│   │   │   ├── segment_coordinator.rs  # Segment scheduling
│   │   │   ├── proxy_config.rs    # SOCKS/HTTP proxy
│   │   │   └── plugin/            # QuickJS plugin runtime
│   │   └── build.rs
│   ├── api/                # Public API server (REST, WebSocket, MCP)
│   ├── cli/                # CLI interface
│   ├── server/             # Standalone HTTP server
│   └── hub/                # Mobile/desktop hub
└── Cargo.toml              # Rust workspace root
```

## Prerequisites

- **Rust** 1.85+ (edition 2024)
- **Node.js** 18+ and npm
- **System deps** (macOS): Xcode Command Line Tools
- **System deps** (Linux): `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`

## Development

```bash
# Install JS dependencies
npm install

# Run in dev mode (hot-reload)
npm run tauri dev

# Run Rust checks only
cargo check

# Build for production
npm run tauri build
```

The Tauri dev server starts the Vite frontend on `http://localhost:1420` and launches a native window.

## Building

```bash
# Desktop release
cargo build --release -p ns_download_app

# Server binary
cargo build --release -p ns_download_server

# CLI binary
cargo build --release -p ns_download_cli
```

## Configuration

The engine reads configuration from the database (`tasks`, `config` tables) and environment:

| Variable | Description |
|---|---|
| `NSDOWNLOAD_APP_VERSION` | Version string override for User-Agent |
| `DATABASE_URL` | Postgres URL (default: SQLite in data dir) |
| `XDG_DATA_HOME` / `HOME` | Data directory resolution |

Proxy, BT, and download limits are configurable at runtime via the engine API.
