// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use notify::{EventKind, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use tauri::{Emitter, Manager};

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
    #[serde(default = "default_bottom_margin")]
    bottom_margin: f64,
    #[serde(default = "default_text_file")]
    text_file: String,
}

fn default_background_opacity() -> f64 { 0.72 }
fn default_text_color() -> String { "#f5e642".to_string() }
fn default_font_size_pt() -> u32 { 24 }
fn default_bottom_margin() -> f64 { 50.0 }
fn default_text_file() -> String { "hud_text.txt".to_string() }

impl Default for HudConfig {
    fn default() -> Self {
        Self {
            background_opacity: default_background_opacity(),
            text_color: default_text_color(),
            font_size_pt: default_font_size_pt(),
            min_font_size_pt: 0,
            bottom_margin: default_bottom_margin(),
            text_file: default_text_file(),
        }
    }
}

#[tauri::command]
fn get_display_config(config: tauri::State<HudConfig>) -> HudConfig {
    config.inner().clone()
}

#[tauri::command]
fn get_hud_text(config: tauri::State<HudConfig>) -> String {
    fs::read_to_string(project_root().join(&config.text_file))
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// Called by the frontend after each render to resize and reposition the window.
#[tauri::command]
fn set_window_height(height: f64, config: tauri::State<HudConfig>, window: tauri::WebviewWindow) {
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let scale = monitor.scale_factor();
        let mon_w = monitor.size().width as f64 / scale;
        let mon_h = monitor.size().height as f64 / scale;
        let mon_x = monitor.position().x as f64 / scale;
        let mon_y = monitor.position().y as f64 / scale;

        let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: mon_w,
            height,
        }));
        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition {
            x: mon_x,
            y: mon_y + mon_h - height - config.bottom_margin,
        }));
    }
}

fn main() {
    let config: HudConfig = fs::read_to_string(project_root().join("hud_config.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    tauri::Builder::default()
        .manage(config.clone())
        .invoke_handler(tauri::generate_handler![
            get_display_config,
            get_hud_text,
            set_window_height
        ])
        .setup(move |app| {
            if let Some(win) = app.get_webview_window("main") {
                // Set initial size: full monitor width, small height (frontend will resize)
                if let Ok(Some(monitor)) = win.primary_monitor() {
                    let scale = monitor.scale_factor();
                    let mon_w = monitor.size().width as f64 / scale;
                    let mon_h = monitor.size().height as f64 / scale;
                    let mon_x = monitor.position().x as f64 / scale;
                    let mon_y = monitor.position().y as f64 / scale;
                    let initial_h = 120.0_f64;

                    let _ = win.set_size(tauri::Size::Logical(tauri::LogicalSize {
                        width: mon_w,
                        height: initial_h,
                    }));
                    let _ = win.set_position(tauri::Position::Logical(tauri::LogicalPosition {
                        x: mon_x,
                        y: mon_y + mon_h - initial_h - config.bottom_margin,
                    }));
                }
            }

            // Watch hud_text.txt for changes and emit event to frontend
            let text_path = project_root().join(&config.text_file);
            let app_handle = app.handle().clone();

            std::thread::spawn(move || {
                let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();

                let mut watcher = match notify::recommended_watcher(tx) {
                    Ok(w) => w,
                    Err(e) => {
                        eprintln!("Failed to create file watcher: {e}");
                        return;
                    }
                };

                if let Err(e) = watcher.watch(&text_path, RecursiveMode::NonRecursive) {
                    eprintln!("Failed to watch {}: {e}", text_path.display());
                    return;
                }

                for result in rx {
                    if let Ok(event) = result {
                        match event.kind {
                            EventKind::Modify(_) | EventKind::Create(_) => {
                                if let Ok(content) = fs::read_to_string(&text_path) {
                                    app_handle
                                        .emit("hud-text-changed", content.trim().to_string())
                                        .ok();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
