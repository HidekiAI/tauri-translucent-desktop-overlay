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

| Field                | Description                                                                                                                                     | Default                        |
| -------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------ |
| `height`             | Window height in logical pixels. Width is always full monitor width. Set tall enough for your longest expected wrapped text                     | `200`                          |
| `background_opacity` | Alpha of the dark box behind the text (0.0–1.0)                                                                                                 | `0.72`                         |
| `text_color`         | Text color, any CSS color string                                                                                                                | `"#f5e642"`                    |
| `font_size_pt`       | Preferred font size in points                                                                                                                   | `24`                           |
| `min_font_size_pt`   | If > 0 and < `font_size_pt`: shrink to this size when text overflows one line, then wrap. Set to `0` to disable (always wrap at `font_size_pt`) | `0`                            |
| `bottom_margin`      | Gap in logical pixels between the window's bottom edge and the screen bottom                                                                    | `50`                           |
| `udp_port`           | UDP port the HUD listens on (`127.0.0.1`)                                                                                                       | `7331`                         |
| `default_text`       | Text displayed on startup before any message arrives                                                                                            | `"Hello world, Hello Shiroe!"` |

Window width is always the full monitor width. Size and position are set **once at startup** — the subtitle box floats at the bottom of the window via CSS. No runtime resizing.

## Keyboard controls

While the HUD window has focus, use the arrow keys to reposition it vertically:

| Key           | Action                                   |
| ------------- | ---------------------------------------- |
| `↑` ArrowUp   | Move HUD toward the top of the screen    |
| `↓` ArrowDown | Move HUD toward the bottom of the screen |

The HUD cycles through three positions: **top**, **center**, and **bottom** (default). Horizontal position is always centered on the current monitor. The `bottom_margin` config value controls the gap from the top/bottom screen edges.

> **Note:** The HUD spawns at the OS-default position on startup. Repositioning is intentionally deferred to the first keypress — setting the position at startup was found to trigger a duplicate process bug in Tauri.

## Sending text to the HUD

The HUD listens on `127.0.0.1:7331` (UDP). Send any text with `nc`:

```bash
# Single line
printf 'Now entering the dungeon...' | nc -u -w1 127.0.0.1 7331

# Multi-line — newlines render as line breaks
printf 'Line one\nLine two' | nc -u -w1 127.0.0.1 7331
```

Each UDP datagram is one complete message. The port is configurable via `udp_port` in `hud_config.json`.

## Troubleshooting: window not transparent (X11 compositor)

### Symptom

The HUD window background appears opaque or dark instead of showing the desktop through it. This happens on X11 sessions using a compositor that does not correctly handle ARGB (per-pixel alpha) windows — most commonly **xfwm4** (the default XFCE window manager).

### Root cause

xfwm4's built-in compositor uses alpha-blend-over accumulation: each new semi-transparent frame is blended _on top of the previous compositor buffer_ rather than composited fresh against the desktop. The result is that transparent areas fill up with accumulated dark content over time instead of showing what is behind the window.

### Diagnosis

Check which compositor is running:

```bash
ps auxf | grep -E "picom|compton|xfwm|mutter|kwin|openbox"
```

If you see `xfwm4` and **no** `picom`/`compton`, this is likely your issue.

Also confirm you are on X11 (not Wayland):

```bash
echo $XDG_SESSION_TYPE # should print "x11"
echo $WAYLAND_DISPLAY  # should be empty
```

### Fix: replace xfwm4's compositor with picom

**1. Install picom:**

```bash
sudo apt install -y picom
```

**2. Disable xfwm4's built-in compositor:**

```bash
xfconf-query -c xfwm4 -p /general/use_compositing -s false
```

**3. Start picom with the GLX backend:**

```bash
picom --backend glx --no-use-damage &
```

`--no-use-damage` forces full-surface redraws, which prevents ghost pixels when text changes.

**4. Restart the HUD** so the window is registered under the new compositor:

```bash
pkill -f "tauri-desktop-HUD"
cargo tauri dev
```

### Make picom start automatically

In XFCE: **Session and Startup → Application Autostart → Add**

- Name: `picom`
- Command: `picom --backend glx --no-use-damage`

Leave xfwm4 compositing disabled permanently.

### Alternative: xrender backend

If GLX is unavailable (e.g. no hardware acceleration), use xrender instead:

```bash
picom --backend xrender --no-use-damage &
```

---

## Known issue: ghost / residual text pixels (X11 WebKit dirty-rect bug)

### Symptom

When shorter text replaces longer text, the characters from the old (wider) text remain visible on screen as a ghost overlaid on the new text. Switching focus away and back (Alt+Tab) sometimes clears the ghost.

### Root cause (confirmed)

WebKit2GTK's software renderer tracks **dirty rectangles**. When the subtitle pill shrinks (shorter text → narrower box), the area vacated by the old pill transitions from "pill background" → "transparent window background". WebKit's dirty-rect optimiser treats this as `transparent → transparent = no change` and **skips writing those pixels to the X11 surface**. The old text pixels remain in the X11 surface indefinitely.

Focus-switching clears ghosts because the X11 window manager sends an **Expose event**, which forces WebKit to perform a full-window repaint (bypassing dirty-rect logic) and write all pixels — including the previously-skipped transparent ones.

Software rendering is forced on via env vars in `main()` (`WEBKIT_DISABLE_COMPOSITING_MODE=1`, `WEBKIT_DISABLE_DMABUF_RENDERER=1`, `LIBGL_ALWAYS_SOFTWARE=1`). These are required for the window to be transparent; removing them breaks transparency.

### What has been tried (and why it failed)

| Approach | Why it failed |
|---|---|
| 3-frame `requestAnimationFrame` erase | rAF is throttled by WebKit when the HUD window is not focused; callbacks never fire |
| `setTimeout` erase | Same throttling as rAF |
| `hud-erase` event (Rust sleep 50ms + opaque box) | When WebKit is throttled, both `hud-erase` and `hud-text-changed` events queue and fire in the same frame — the erase phase has no effect. Worse: the *opaque* erase frame itself becomes the ghost. |
| CSS `body { background: rgba(10,10,10,0.02) }` (near-zero) | Should force WebKit to paint vacated area (non-zero → WebKit must repaint). Insufficient on its own — WebKit may not repaint body layer when only subtitle-box changes. |
| JS body-background toggle in `renderText` | Marks body element dirty (forces body-layer repaint in same frame). Still intermittent — may not reach the X11 surface in all compositor configurations. |
| `XSendEvent(Expose)` from Rust (150ms after text update) | Programmatic version of the Alt+Tab fix. Still intermittent — unclear whether: (a) 150ms isn't long enough for throttled WebKit to update its render tree before Expose fires, or (b) WebKit's Expose handler also uses dirty-rect optimization and skips transparent areas. |

### Current state of the code

- `main.rs`: emits `hud-text-changed`, sleeps 150ms, then sends `XSendEvent(Expose, full window)` via `x11` crate. X11 display and window handles extracted at startup and stored as `usize` for thread safety. The `x11` and `raw-window-handle` crates are in `Cargo.toml`.
- `ui/app.js`: `renderText()` toggles body background between `rgba(10,10,10,0.02)` and `rgba(10,10,10,0.04)` on each call to force a body-layer repaint.
- `ui/styles.css`: `body { background: rgba(10,10,10,0.02) }` (non-zero base).

### Next things to try

1. **Confirm Expose is reaching WebKit.** Add an Expose event listener in GTK using the `gtk` crate to verify the synthetic event is received and triggers a draw signal. If GTK filters synthetic Expose events, `XSendEvent` with `propagate=False` may be ignored.

2. **Use `gdk_window_invalidate_rect(NULL, false)` instead of raw Expose.** This calls GTK's own invalidation API (via the `gdk` crate), which is the correct way to request a full repaint from outside the GTK main loop. Requires adding `gdk = "0.18"` to `Cargo.toml` and getting the GDK window from the Tauri `WebviewWindow`. May need `WebviewWindow::gtk_window()` if that method is exposed in tauri 2.x.

3. **Increase the Expose delay.** If 150ms isn't enough for throttled WebKit to process the DOM update, try 300–500ms. The ghost would be visible briefly after each text change before clearing. For subtitle use this may be acceptable.

4. **Always use full-width pill** (`width: 90%` instead of `max-width: 90%`). Eliminates the "pill shrinks" case entirely since the box never gets narrower. Trades the content-hugging pill aesthetic for reliability. User previously found this too visually prominent — may be acceptable as a config option.

5. **Force a 1-pixel window resize** from Rust (`window.set_size(w, h+1)` then `set_size(w, h)`). A resize triggers X11 ConfigureNotify → GTK relayout → WebKit full repaint. Risk: visible 1-frame flicker.
