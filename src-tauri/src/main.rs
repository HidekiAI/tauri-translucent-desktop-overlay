// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_url() -> String {
    // Return YouTube login page URL
    "https://accounts.google.com/signin/v2/identifier?service=youtube".to_string()
}

#[tauri::command]
fn send_custom_url(url: &str) -> String {
    // Validate and return the URL
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, get_url, send_custom_url])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
