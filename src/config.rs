use serde_json::{json, Map, Value};
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

pub fn legacy_config_path(override_path: Option<&str>) -> Option<PathBuf> {
    if override_path.is_some() { return None; }
    let p = config_home().join("nullfetch").join("config.json");
    if p.exists() { Some(p) } else { None }
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

pub fn strip_jsonc(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_str = false;
    let mut esc = false;
    let mut line_comment = false;
    let mut block_comment = false;
    while let Some(c) = chars.next() {
        if line_comment {
            if c == '\n' { line_comment = false; out.push(c); }
            continue;
        }
        if block_comment {
            if c == '*' && chars.peek() == Some(&'/') { chars.next(); block_comment = false; }
            continue;
        }
        if in_str {
            out.push(c);
            if esc { esc = false; }
            else if c == '\\' { esc = true; }
            else if c == '"' { in_str = false; }
            continue;
        }
        if c == '"' { in_str = true; out.push(c); continue; }
        if c == '/' {
            match chars.peek() {
                Some('/') => { chars.next(); line_comment = true; continue; }
                Some('*') => { chars.next(); block_comment = true; continue; }
                _ => {}
            }
        }
        out.push(c);
    }
    // remove trailing commas before } or ]
    let mut cleaned = String::with_capacity(out.len());
    let chars: Vec<char> = out.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '"' {
            cleaned.push(c); i += 1;
            let mut esc = false;
            while i < chars.len() {
                let x = chars[i]; cleaned.push(x); i += 1;
                if esc { esc = false; continue; }
                if x == '\\' { esc = true; continue; }
                if x == '"' { break; }
            }
            continue;
        }
        if c == ',' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() { j += 1; }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') { i += 1; continue; }
        }
        cleaned.push(c); i += 1;
    }
    cleaned
}

pub fn load_config(path: &Path) -> Result<Value, String> {
    let mut p = path.to_path_buf();
    if !p.exists() {
        let jsonc = p.with_extension("jsonc");
        let json = p.with_extension("json");
        if jsonc.exists() { p = jsonc; }
        else if json.exists() { p = json; }
        else if let Some(legacy) = legacy_config_path(None) { p = legacy; }
    }
    match fs::read_to_string(&p) {
        Ok(s) => {
            let stripped = strip_jsonc(&s);
            if stripped.trim().is_empty() { return Ok(json!({})); }
            serde_json::from_str(&stripped).map_err(|e| format!("cannot load config {}: {}; use --reset-config", p.display(), e))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
        Err(e) => Err(format!("cannot load config {}: {}; use --reset-config", p.display(), e)),
    }
}

pub fn write_config(path: &Path, data: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    let tmp = path.with_extension("tmp");
    let body = serde_json::to_string_pretty(data).map_err(|e| e.to_string())? + "\n";
    fs::write(&tmp, body).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}

pub fn default_config() -> Value {
    json!({
        "$schema": "https://raw.githubusercontent.com/pybindi/nullfetch/main/schema.json",
        "logo": "auto",
        "theme": "default",
        "color": "auto",
        "separator": ": ",
        "bar": { "style": "blocks", "width": 12 },
        "modules": ["user","host","os","kernel","cpu","gpu","ram","disk"]
    })
}
