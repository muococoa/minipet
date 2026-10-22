//! Pets live in pets/<name>/, with a <name>.json next to their sprite sheets.
//! Adding a pet is adding a folder. Nothing here knows about a specific pet.

use std::{borrow::Cow, fs, path::PathBuf};

use tauri::{
    http::{header, Request, Response, StatusCode},
    AppHandle, Manager, Runtime, UriSchemeContext,
};

pub fn dir<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    if cfg!(debug_assertions) {
        // While developing, read straight from the repo so new sprites show up on reload.
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../pets")
    } else {
        app.path()
            .resource_dir()
            .map(|dir| dir.join("pets"))
            .unwrap_or_default()
    }
}

/// Every folder in pets/ that has a <name>.json inside it.
pub fn list<R: Runtime>(app: &AppHandle<R>) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir(app))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| dir(app).join(name).join(format!("{name}.json")).is_file())
        .collect();
    names.sort();
    names
}

pub fn read_definition<R: Runtime>(app: &AppHandle<R>, name: &str) -> Result<serde_json::Value, String> {
    let path = dir(app).join(name).join(format!("{name}.json"));
    let text = fs::read_to_string(&path).map_err(|e| format!("Can't read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{} is not valid json: {e}", path.display()))
}

/// Animation names in the order they appear in the pet's json.
pub fn animations<R: Runtime>(app: &AppHandle<R>, name: &str) -> Vec<String> {
    read_definition(app, name)
        .ok()
        .and_then(|def| def.get("animations")?.as_object().map(|a| a.keys().cloned().collect()))
        .unwrap_or_default()
}

/// Serves pets/ to the page as pet://localhost/<name>/<file>.
pub fn protocol<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    let path = request.uri().path().trim_start_matches('/');
    let not_found = || {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Cow::Borrowed(&[][..]))
            .unwrap()
    };
    if path.is_empty() || path.split('/').any(|part| part == ".." || part.contains('\\')) {
        return not_found();
    }
    let Ok(bytes) = fs::read(dir(ctx.app_handle()).join(path)) else {
        return not_found();
    };
    let mime = match path.rsplit('.').next() {
        Some("png") => "image/png",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    };
    Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, "no-store")
        .body(Cow::Owned(bytes))
        .unwrap()
}
