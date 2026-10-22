#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod activity;
mod menu;
#[cfg(target_os = "macos")]
mod panel;
mod pets;
mod settings;

use std::sync::Mutex;

use settings::Settings;
use tauri::{AppHandle, Manager, PhysicalPosition, State, WebviewWindow};

pub struct AppState {
    pub settings: Mutex<Settings>,
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Settings {
    state.settings.lock().unwrap().clone()
}

/// The current pet's json, exactly as written in pets/<name>/<name>.json.
#[tauri::command]
fn load_pet(app: AppHandle, state: State<AppState>) -> Result<serde_json::Value, String> {
    let name = state.settings.lock().unwrap().pet.clone();
    pets::read_definition(&app, &name)
}

/// Remembers where the pet's feet are, so he comes back to the same spot.
#[tauri::command]
fn save_position(app: AppHandle, window: WebviewWindow, state: State<AppState>) {
    let (Ok(pos), Ok(size)) = (window.outer_position(), window.outer_size()) else {
        return;
    };
    let mut settings = state.settings.lock().unwrap();
    settings.anchor = Some((pos.x + size.width as i32 / 2, pos.y + size.height as i32));
    settings.save(&app);
}

/// Puts the window where the pet's feet were last time, or bottom right on first run.
#[tauri::command]
fn place_pet(window: WebviewWindow, state: State<AppState>) -> Result<(), String> {
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let saved = state.settings.lock().unwrap().anchor;
    let (x, y) = saved
        .filter(|&(x, y)| is_on_screen(&window, x, y))
        .or_else(|| default_anchor(&window))
        .unwrap_or((400, 400));
    window
        .set_position(PhysicalPosition::new(x - size.width as i32 / 2, y - size.height as i32))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn show_pet(app: AppHandle, window: WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        let _ = window;
        panel::show(&app);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
        let _ = window.show();
    }
}

#[tauri::command]
fn show_menu(app: AppHandle, window: WebviewWindow) {
    if let Ok(menu) = menu::build(&app) {
        let _ = window.popup_menu(&menu);
    }
}

/// Lets the page print to the terminal you ran `npm run dev` from.
#[tauri::command]
fn log(message: String) {
    eprintln!("[pet] {message}");
}

fn is_on_screen(window: &WebviewWindow, x: i32, y: i32) -> bool {
    window
        .available_monitors()
        .map(|monitors| {
            monitors.iter().any(|m| {
                let (p, s) = (m.position(), m.size());
                x >= p.x && x < p.x + s.width as i32 && y > p.y && y <= p.y + s.height as i32
            })
        })
        .unwrap_or(false)
}

fn default_anchor(window: &WebviewWindow) -> Option<(i32, i32)> {
    let monitor = window.primary_monitor().ok().flatten()?;
    let (p, s, k) = (monitor.position(), monitor.size(), monitor.scale_factor());
    Some((
        p.x + s.width as i32 - (160.0 * k) as i32,
        p.y + s.height as i32 - (120.0 * k) as i32,
    ))
}

fn main() {
    let builder = tauri::Builder::default();
    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_nspanel::init());

    builder
        .register_uri_scheme_protocol("pet", pets::protocol)
        .invoke_handler(tauri::generate_handler![
            get_settings,
            load_pet,
            save_position,
            place_pet,
            show_pet,
            show_menu,
            log
        ])
        .setup(|app| {
            // No Dock icon. The pet lives in the menu bar (tray) instead.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            app.manage(AppState {
                settings: Mutex::new(Settings::load(app.handle())),
            });

            #[cfg(target_os = "macos")]
            panel::init(&app.get_webview_window("main").unwrap());

            menu::create_tray(app.handle())?;
            activity::start(app.handle().clone());
            Ok(())
        })
        .on_menu_event(menu::handle)
        .run(tauri::generate_context!())
        .expect("error while running minipet");
}
