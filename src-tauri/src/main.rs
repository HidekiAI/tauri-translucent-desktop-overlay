// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::net::UdpSocket;
use std::path::PathBuf;
use tauri::{Emitter, Manager};

#[cfg(target_os = "linux")]
use {
    raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle},
    x11::xlib,
};

/// During `cargo tauri dev` the binary's cwd is `src-tauri/`.
/// In production it sits next to the binary.
/// Check cwd first, then parent, so both cases work.
fn project_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_default();
    if cwd.join("hud_config.json").exists() {
        cwd
    } else {
        cwd.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or(std::env::current_dir().unwrap_or_default())
    }
}

#[derive(Deserialize, Serialize, Clone)]
struct HudConfig {
    #[serde(default = "default_background_opacity")]
    background_opacity: f64,
    #[serde(default = "default_text_color")]
    text_color: String,
    #[serde(default = "default_font_size_pt")]
    font_size_pt: u32,
    #[serde(default)]
    min_font_size_pt: u32,
    #[serde(default = "default_height")]
    height: f64,
    #[serde(default = "default_bottom_margin")]
    bottom_margin: f64,
    #[serde(default = "default_udp_port")]
    udp_port: u16,
    #[serde(default = "default_default_text")]
    default_text: String,
}

fn default_background_opacity() -> f64 { 0.72 }
fn default_text_color() -> String { "#f5e642".to_string() }
fn default_font_size_pt() -> u32 { 24 }
fn default_height() -> f64 { 200.0 }
fn default_bottom_margin() -> f64 { 50.0 }
fn default_udp_port() -> u16 { 7331 }
fn default_default_text() -> String { "Hello world, Hello Shiroe!".to_string() }

impl Default for HudConfig {
    fn default() -> Self {
        Self {
            background_opacity: default_background_opacity(),
            text_color: default_text_color(),
            font_size_pt: default_font_size_pt(),
            min_font_size_pt: 0,
            height: default_height(),
            bottom_margin: default_bottom_margin(),
            udp_port: default_udp_port(),
            default_text: default_default_text(),
        }
    }
}

#[tauri::command]
fn get_display_config(config: tauri::State<HudConfig>) -> HudConfig {
    config.inner().clone()
}

#[tauri::command]
fn get_hud_text(config: tauri::State<HudConfig>) -> String {
    config.default_text.clone()
}

/// Move the HUD window to one of three vertical positions: "top", "center", "bottom".
/// Horizontal position is always centered on the current monitor.
#[tauri::command]
fn move_window(
    window: tauri::WebviewWindow,
    position: String,
    config: tauri::State<HudConfig>,
) -> Result<(), String> {
    use tauri::PhysicalPosition;

    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no monitor detected".to_string())?;

    let msize = monitor.size();
    let mpos  = monitor.position();
    let wsize = window.outer_size().map_err(|e| e.to_string())?;

    let margin = config.bottom_margin as i32;
    let x = mpos.x + (msize.width as i32 - wsize.width as i32) / 2;
    let y = match position.as_str() {
        "top"    => mpos.y + margin,
        "center" => mpos.y + (msize.height as i32 - wsize.height as i32) / 2,
        _        => mpos.y + msize.height as i32 - wsize.height as i32 - margin,
    };

    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())
}

/// Thread-safe holder for the X11 display pointer and window ID.
/// Store as plain integers so the struct is Send without unsafe impl.
/// XInitThreads() is called by GTK during init, making multi-threaded
/// Xlib calls safe.
#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
struct X11Window {
    display: usize, // *mut xlib::Display cast to usize
    window:  u64,   // xlib::Window (XID)
    width:   u32,
    height:  u32,
}

fn main() {
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");

    let config: HudConfig = fs::read_to_string(project_root().join("hud_config.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    tauri::Builder::default()
        .manage(config.clone())
        .invoke_handler(tauri::generate_handler![get_display_config, get_hud_text, move_window])
        .setup(move |app| {
            let addr = format!("127.0.0.1:{}", config.udp_port);
            let app_handle = app.handle().clone();

            // Extract X11 display/window handles once, before spawning the thread.
            #[cfg(target_os = "linux")]
            let x11: X11Window = {
                let win = app
                    .get_webview_window("main")
                    .ok_or("could not find 'main' webview window")?;

                let size = win.inner_size().unwrap_or(tauri::PhysicalSize { width: 800, height: 200 });

                let display_usize = match win.display_handle().map(|h: raw_window_handle::DisplayHandle<'_>| h.as_raw()) {
                    Ok(RawDisplayHandle::Xlib(h)) => h
                        .display
                        .map(|p| p.as_ptr() as usize)
                        .unwrap_or(0),
                    _ => 0,
                };

                let window_id: u64 = match win.window_handle().map(|h: raw_window_handle::WindowHandle<'_>| h.as_raw()) {
                    Ok(RawWindowHandle::Xlib(h)) => h.window as u64,
                    _ => 0,
                };

                eprintln!(
                    "X11 display=0x{:x}  window=0x{:x}  size={}x{}",
                    display_usize, window_id, size.width, size.height
                );

                X11Window {
                    display: display_usize,
                    window:  window_id,
                    width:   size.width,
                    height:  size.height,
                }
            };

            std::thread::spawn(move || {
                let socket = match UdpSocket::bind(&addr) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to bind UDP socket {addr}: {e}");
                        return;
                    }
                };

                eprintln!("HUD listening on UDP {addr}");

                let mut buf = vec![0u8; 65507];
                loop {
                    match socket.recv_from(&mut buf) {
                        Ok((n, _src)) => {
                            let text = String::from_utf8_lossy(&buf[..n])
                                .trim()
                                .to_string();
                            if !text.is_empty() {
                                // Tell JS to render the new text. The body-background
                                // toggle in renderText() marks the body layer dirty so
                                // WebKit repaints it in the same frame.
                                app_handle.emit("hud-text-changed", text).ok();

                                // Give WebKit time to process the event and update its
                                // internal render tree before we force a full repaint.
                                // Even a throttled (unfocused) WebKit processes IPC
                                // events; 150 ms covers ~1 render cycle at 6-8 Hz.
                                std::thread::sleep(
                                    std::time::Duration::from_millis(150),
                                );

                                // Send a synthetic X11 Expose event covering the whole
                                // window.  Unlike a dirty-rect repaint, Expose forces
                                // WebKit to write every pixel — including transparent
                                // areas — so ghost pixels from old text are cleared.
                                // (This is what Alt+Tab does, made programmatic.)
                                #[cfg(target_os = "linux")]
                                if x11.display != 0 && x11.window != 0 {
                                    unsafe {
                                        let dpy = x11.display as *mut xlib::Display;
                                        let win = x11.window as xlib::Window;
                                        let mut ev: xlib::XEvent = std::mem::zeroed();
                                        ev.expose.type_   = xlib::Expose;
                                        ev.expose.display = dpy;
                                        ev.expose.window  = win;
                                        ev.expose.x       = 0;
                                        ev.expose.y       = 0;
                                        ev.expose.width   = x11.width  as std::os::raw::c_int;
                                        ev.expose.height  = x11.height as std::os::raw::c_int;
                                        ev.expose.count   = 0;
                                        xlib::XSendEvent(
                                            dpy,
                                            win,
                                            0,                          // propagate = False
                                            xlib::ExposureMask as i64,
                                            &mut ev,
                                        );
                                        xlib::XFlush(dpy);
                                    }
                                }
                            }
                        }
                        Err(e) => eprintln!("UDP receive error: {e}"),
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
