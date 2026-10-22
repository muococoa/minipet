//! The menu bar icon's menu. Right-clicking the pet shows the same one.

use tauri::{
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

use crate::{pets, AppState};

const TRAY_ID: &str = "pet";
const MAX_SCALE: u32 = 4;

pub fn build(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let settings = app.state::<AppState>().settings.lock().unwrap().clone();

    let size = Submenu::with_id(app, "size", "Size", true)?;
    for n in 1..=MAX_SCALE {
        size.append(&CheckMenuItem::with_id(
            app,
            format!("scale:{n}"),
            format!("{n}×"),
            true,
            settings.scale == n,
            None::<&str>,
        )?)?;
    }

    let pet = Submenu::with_id(app, "pet", "Pet", true)?;
    for name in pets::list(app) {
        let checked = name == settings.pet;
        pet.append(&CheckMenuItem::with_id(app, format!("pet:{name}"), &name, true, checked, None::<&str>)?)?;
    }

    let play = Submenu::with_id(app, "play", "Play animation", true)?;
    for name in pets::animations(app, &settings.pet) {
        play.append(&MenuItem::with_id(app, format!("play:{name}"), &name, true, None::<&str>)?)?;
    }

    Menu::with_items(
        app,
        &[
            &size,
            &pet,
            &play,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "reload", "Reload pet files", true, None::<&str>)?,
            &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
        ],
    )
}

pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Minipet")
        .menu(&build(app)?)
        .show_menu_on_left_click(true);
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

fn refresh(app: &AppHandle) {
    if let (Some(tray), Ok(menu)) = (app.tray_by_id(TRAY_ID), build(app)) {
        let _ = tray.set_menu(Some(menu));
    }
}

pub fn handle(app: &AppHandle, event: MenuEvent) {
    let id = event.id().as_ref();
    let state = app.state::<AppState>();

    if id == "quit" {
        app.exit(0);
    } else if id == "reload" {
        refresh(app);
        let _ = app.emit("reload", ());
    } else if let Some(n) = id.strip_prefix("scale:").and_then(|n| n.parse::<u32>().ok()) {
        let mut settings = state.settings.lock().unwrap();
        settings.scale = n;
        settings.save(app);
        drop(settings);
        refresh(app);
        let _ = app.emit("scale", n);
    } else if let Some(name) = id.strip_prefix("pet:") {
        let mut settings = state.settings.lock().unwrap();
        settings.pet = name.to_string();
        settings.save(app);
        drop(settings);
        refresh(app);
        let _ = app.emit("reload", ());
    } else if let Some(name) = id.strip_prefix("play:") {
        let _ = app.emit("play", name);
    }
}
