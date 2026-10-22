use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

/// Everything the app remembers between launches.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Folder name inside pets/.
    pub pet: String,
    /// Screen pixels per sprite pixel. Whole numbers only.
    pub scale: u32,
    /// Where the pet's feet are (bottom center of the window), in screen pixels.
    pub anchor: Option<(i32, i32)>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            pet: "pichi".into(),
            scale: 1,
            anchor: None,
        }
    }
}

impl Settings {
    fn path(app: &AppHandle) -> Option<PathBuf> {
        app.path().app_config_dir().ok().map(|dir| dir.join("settings.json"))
    }

    pub fn load(app: &AppHandle) -> Self {
        Self::path(app)
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, app: &AppHandle) {
        let Some(path) = Self::path(app) else { return };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, text);
        }
    }
}
