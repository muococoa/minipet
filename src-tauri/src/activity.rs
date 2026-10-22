//! Tells the page how long it has been since you last touched the keyboard or mouse.
//! On macOS this reads the system idle timer, which needs no special permissions.

use std::{thread, time::Duration};

use tauri::{AppHandle, Emitter};

pub fn start(app: AppHandle) {
    thread::spawn(move || loop {
        if let Some(seconds) = seconds_since_input() {
            let _ = app.emit("activity", seconds);
        }
        thread::sleep(Duration::from_secs(1));
    });
}

#[cfg(target_os = "macos")]
fn seconds_since_input() -> Option<f64> {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(state_id: i32, event_type: u32) -> f64;
    }
    const COMBINED_SESSION_STATE: i32 = 0;
    const ANY_INPUT_EVENT: u32 = u32::MAX;
    Some(unsafe { CGEventSourceSecondsSinceLastEventType(COMBINED_SESSION_STATE, ANY_INPUT_EVENT) })
}

#[cfg(not(target_os = "macos"))]
fn seconds_since_input() -> Option<f64> {
    None
}
