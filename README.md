# tauri-translucent-desktop-overlay

A minimal Tauri desktop HUD overlay — transparent, decoration-free window that renders a translucent subtitle-style display over your desktop.

**Stack:** Tauri 2.x · Vanilla JS · Plain HTML/CSS

## Features

- Transparent, borderless window (no OS decorations)
- Window dimensions driven by `hud_config.json` — no recompile needed
- Live text updates via UDP loopback — pipe any string to port 7331 and the HUD updates instantly

## Setup

```bash
# Install system dependencies (Debian/Ubuntu)
sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev

# Install Tauri CLI
cargo install tauri-cli --version "^2.0.0" --locked

# Update Rust dependencies
cd src-tauri && cargo update && cd ..
```

Or just run `./setup.sh` to do all of the above and launch dev mode.

No npm/pnpm required — the frontend is plain HTML/CSS/JS in `ui/`.

## Run

```bash
cargo tauri dev
```

## Configuration

Edit `hud_config.json` at the project root. All changes take effect on the next launch. The file is optional — defaults are used if it is missing or malformed.

```json
{
  "height": 200,
  "background_opacity": 0.72,
  "text_color": "#f5e642",
  "font_size_pt": 24,
  "min_font_size_pt": 0,
  "bottom_margin": 50,
  "udp_port": 7331,
  "default_text": "Hello world, Hello Shiroe!"
}
```

| Field | Description | Default |
|---|---|---|
| `height` | Window height in logical pixels. Width is always full monitor width. Set tall enough for your longest expected wrapped text | `200` |
| `background_opacity` | Alpha of the dark box behind the text (0.0–1.0) | `0.72` |
| `text_color` | Text color, any CSS color string | `"#f5e642"` |
| `font_size_pt` | Preferred font size in points | `24` |
| `min_font_size_pt` | If > 0 and < `font_size_pt`: shrink to this size when text overflows one line, then wrap. Set to `0` to disable (always wrap at `font_size_pt`) | `0` |
| `bottom_margin` | Gap in logical pixels between the window's bottom edge and the screen bottom | `50` |
| `udp_port` | UDP port the HUD listens on (`127.0.0.1`) | `7331` |
| `default_text` | Text displayed on startup before any message arrives | `"Hello world, Hello Shiroe!"` |

Window width is always the full monitor width. Size and position are set **once at startup** — the subtitle box floats at the bottom of the window via CSS. No runtime resizing.

## Sending text to the HUD

The HUD listens on `127.0.0.1:7331` (UDP). Send any text with `nc`:

```bash
# Single line
printf 'Now entering the dungeon...' | nc -u -w1 127.0.0.1 7331

# Multi-line — newlines render as line breaks
printf 'Line one\nLine two' | nc -u -w1 127.0.0.1 7331
```

Each UDP datagram is one complete message. The port is configurable via `udp_port` in `hud_config.json`.

## IDE Setup

[VS Code](https://code.visualstudio.com/) +
[Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) +
[rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
