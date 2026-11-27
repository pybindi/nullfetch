use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn cache_dir() -> PathBuf { crate::config::cache_dir() }

fn path_for(name: &str) -> PathBuf {
    let safe: String = name.chars().map(|c| if c.is_ascii_alphanumeric()||c=='-'||c=='_' {c} else {'_'}).collect();
    cache_dir().join(format!("{safe}.json"))
}

pub fn read(name: &str, ttl: i64) -> Option<Value> {
    if ttl <= 0 { return None; }
    let p = path_for(name);
    let s = fs::read_to_string(p).ok()?;
    let v: Value = serde_json::from_str(&s).ok()?;
    let t = v.get("time")?.as_f64()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs_f64();
    if now - t <= ttl as f64 { v.get("value").cloned() } else { None }
}

pub fn write(name: &str, value: &Value) {
    let p = path_for(name);
    if let Some(parent) = p.parent() { let _ = fs::create_dir_all(parent); }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0);
    let body = serde_json::json!({"time": now, "value": value});
    let tmp = p.with_extension("json.tmp");
    if fs::write(&tmp, serde_json::to_string(&body).unwrap_or_default()).is_ok() {
        let _ = fs::rename(&tmp, &p);
    }
}
