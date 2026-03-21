// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Deserialize;
use std::fs;
use tauri::Manager;

#[derive(Deserialize)]
struct HudConfig {
    #[serde(default = "default_width")]
    width: f64,
    #[serde(default = "default_height")]
    height: f64,
}

fn default_width() -> f64 {
    600.0
}
fn default_height() -> f64 {
    300.0
}

impl Default for HudConfig {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
        }
    }
}

fn main() {
    let config: HudConfig = fs::read_to_string("hud_config.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    tauri::Builder::default()
        .setup(move |app| {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.set_size(tauri::Size::Logical(tauri::LogicalSize {
                    width: config.width,
                    height: config.height,
                }));
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
