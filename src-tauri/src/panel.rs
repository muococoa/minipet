//! Turns the pet window into a macOS panel so it floats above everything,
//! full-screen apps included, without ever stealing focus.

use tauri::{AppHandle, WebviewWindow};
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelLevel, StyleMask, WebviewWindowExt,
};

tauri_panel! {
    panel!(PetPanel {
        config: {
            can_become_key_window: false,
            is_floating_panel: true
        }
    })
}

pub fn init(window: &WebviewWindow) {
    let panel = match window.to_panel::<PetPanel>() {
        Ok(panel) => panel,
        Err(e) => {
            eprintln!("[pet] could not make a panel: {e:?}");
            return;
        }
    };
    panel.set_level(PanelLevel::Status.value());
    let _ = panel.add_style_mask(StyleMask::empty().nonactivating_panel().into());
    panel.set_collection_behavior(
        CollectionBehavior::new()
            .can_join_all_spaces()
            .stationary()
            .full_screen_auxiliary()
            .ignores_cycle()
            .into(),
    );
    panel.set_hides_on_deactivate(false);
    panel.set_has_shadow(false);
}

pub fn show(app: &AppHandle) {
    if let Ok(panel) = app.get_webview_panel("main") {
        panel.show();
    }
}
