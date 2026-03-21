# tauri-translucent-desktop-overlay

A minimal Tauri desktop HUD overlay — transparent, decoration-free window that renders a translucent subtitle-style display over your desktop.

**Stack:** Tauri 2.x · SolidJS · TypeScript · Vite · pnpm

## Features

- Transparent, borderless window (no OS decorations)
- Window dimensions driven by `hud_config.json` — no recompile needed
- Live text updates — edit `hud_text.txt` and the HUD refreshes instantly, no restart needed

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

Edit `hud_config.json` at the project root. All changes take effect on the next launch. The file is optional — defaults are used if it is missing or malformed.

```json
{
  "background_opacity": 0.72,
  "text_color": "#f5e642",
  "font_size_pt": 24,
  "min_font_size_pt": 0,
  "bottom_margin": 50,
  "text_file": "hud_text.txt"
}
```

| Field | Description | Default |
|---|---|---|
| `background_opacity` | Alpha of the dark box behind the text (0.0–1.0) | `0.72` |
| `text_color` | Text color, any CSS color string | `"#f5e642"` |
| `font_size_pt` | Preferred font size in points | `24` |
| `min_font_size_pt` | If > 0 and < `font_size_pt`: shrink to this size when text overflows one line, then wrap. Set to `0` to disable (always wrap at `font_size_pt`) | `0` |
| `bottom_margin` | Gap in logical pixels between the window's bottom edge and the screen bottom | `50` |
| `text_file` | Path to the text file to display. Absolute paths (e.g. `/dev/shm/hud.txt`) are used as-is; relative paths are resolved from the project root | `"hud_text.txt"` |

Window width is always the full width of the primary monitor. Window height adjusts automatically to fit the content.

## Live text updates

Edit `hud_text.txt` (or whatever `text_file` points to) while the HUD is running — the overlay refreshes automatically the moment the file is saved. No restart required.

```bash
# Relative path (default, resolved from project root):
echo "Now entering the dungeon..." > hud_text.txt

# Or use an absolute path — handy for tmpfs/shared memory:
# hud_config.json: "text_file": "/dev/shm/hud.txt"
echo "Now entering the dungeon..." > /dev/shm/hud.txt
```

## IDE Setup

[VS Code](https://code.visualstudio.com/) +
[Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) +
[rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
