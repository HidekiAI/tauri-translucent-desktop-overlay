# tauri-translucent-desktop-overlay

A minimal Tauri desktop HUD overlay — transparent, decoration-free window that renders a translucent subtitle-style display over your desktop.

**Stack:** Tauri 2.x · SolidJS · TypeScript · Vite · pnpm

## Features

- Transparent, borderless window (no OS decorations)
- Window dimensions driven by `hud_config.json` — no recompile needed
- Default view: movie-subtitle style "Hello world, Hello Shiroe!" panel

## Setup

```bash
# Install system dependencies (Debian/Ubuntu)
sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev

# Install Tauri CLI
cargo install tauri-cli --version "^2.0.0" --locked

# Install JS dependencies
pnpm install

# Update Rust dependencies
cd src-tauri && cargo update && cd ..
```

Or just run `./setup.sh` to do all of the above and launch dev mode.

## Run

```bash
pnpm run:tauri
```

## Configuration

Edit `hud_config.json` at the project root to set the window size:

```json
{
  "width": 600,
  "height": 300
}
```

Changes take effect on the next launch. Defaults to 600×300 if the file is missing or malformed.

## IDE Setup

[VS Code](https://code.visualstudio.com/) +
[Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) +
[rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
