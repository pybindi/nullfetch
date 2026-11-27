use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub fn config_home() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() { return PathBuf::from(xdg); }
    }
    if cfg!(windows) {
        if let Ok(app) = std::env::var("APPDATA") {
            if !app.is_empty() { return PathBuf::from(app); }
        }
    }
    dirs_home().join(".config")
}

pub fn dirs_home() -> PathBuf {
    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
}

pub fn config_path(override_path: Option<&str>) -> PathBuf {
    if let Some(p) = override_path { return expand_path(p); }
    config_home().join("nullfetch").join("config.json")
}

pub fn cache_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        if !xdg.is_empty() { return PathBuf::from(xdg).join("nullfetch"); }
    }
    if cfg!(windows) {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            if !local.is_empty() { return PathBuf::from(local).join("nullfetch").join("cache"); }
        }
    }
    dirs_home().join(".cache").join("nullfetch")
}

pub fn expand_path(p: &str) -> PathBuf {
    if p == "~" { return dirs_home(); }
    if let Some(rest) = p.strip_prefix("~/").or_else(|| p.strip_prefix("~\\")) {
        return dirs_home().join(rest);
    }
    PathBuf::from(p)
}

pub fn load_config(path: &Path) -> Result<Value, String> {
    match fs::read_to_string(path) {
        Ok(s) => serde_json::from_str(&s).map_err(|e| format!("cannot load config {}: {}; use --reset-config", path.display(), e)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
        Err(e) => Err(format!("cannot load config {}: {}; use --reset-config", path.display(), e)),
    }
}

pub fn write_config(path: &Path, data: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    let tmp = path.with_extension("json.tmp");
    let body = serde_json::to_string_pretty(data).map_err(|e| e.to_string())? + "\n";
    fs::write(&tmp, body).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}
