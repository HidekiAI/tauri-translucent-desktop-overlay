// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::net::UdpSocket;
use std::path::PathBuf;
use tauri::{Emitter, Manager as _};

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

fn main() {
    let config: HudConfig = fs::read_to_string(project_root().join("hud_config.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    tauri::Builder::default()
        .manage(config.clone())
        .invoke_handler(tauri::generate_handler![get_display_config, get_hud_text])
        .setup(move |app| {

            // Listen for incoming text on a UDP loopback socket.
            // Each datagram is one complete message; newlines within are preserved.
            let addr = format!("127.0.0.1:{}", config.udp_port);
            let app_handle = app.handle().clone();

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
                                app_handle.emit("hud-text-changed", text).ok();
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
