mod cache;
mod config;
mod image;
mod logos;
mod modules;
mod render;

use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

const VERSION: &str = "3.1.0";
const FIELDS: &[&str] = &["user","host","os","kernel","cpu","gpu","ram","disk"];
const ALL_FIELDS: &[&str] = &["user","host","os","kernel","arch","uptime","boot_time","loadavg","cpu","cpu_cores","gpu","ram","swap","disk","disks","network","local_ip","public_ip","battery","temperature","resolution","shell","terminal","de","wm","packages","processes","init","locale","timezone","date","motherboard","bios","display","audio","bluetooth","power"];
const EXTRA_FIELDS: &[&str] = &["arch","uptime","boot_time","loadavg","cpu_cores","swap","disks","network","local_ip","public_ip","battery","temperature","resolution","shell","terminal","de","wm","packages","processes","init","locale","timezone","date","motherboard","bios","display","audio","bluetooth","power"];
const IMAGE_MODES: &[&str] = &["auto","blocks","ascii","kitty","iterm","chafa","viu","jp2","img2txt","timg","sixel"];
const THEMES: &[&str] = &["default","mono","light","neon","dark","nord","gruvbox","dracula","tokyo","solarized","matrix"];
const SLOW_FIELDS: &[&str] = &["public_ip","packages","resolution","temperature","battery","audio","bluetooth","display"];

#[derive(Clone, Debug)]
struct ModuleSpec {
    kind: String,
    key: Option<String>,
    format: Option<String>,
    key_color: Option<String>,
}

impl ModuleSpec {
    fn simple(kind: String) -> Self { Self { kind, key: None, format: None, key_color: None } }
    fn display_key(&self) -> String { self.key.clone().unwrap_or_else(|| label(&self.kind)) }
    fn is_builtin(&self) -> bool { !matches!(self.kind.as_str(), "break"|"separator"|"title"|"custom") }
}

fn is_slow(field: &str) -> bool { field.starts_with("command:") || SLOW_FIELDS.contains(&field) }

fn label(key: &str) -> String {
    if key.starts_with("command:") {
        let parts: Vec<&str> = key.splitn(3, ':').collect();
        if parts.len()==3 { return parts[1].to_string(); }
    }
    match key {
        "user"=>"USER","host"=>"HOST","os"=>"OS","kernel"=>"kernel","arch"=>"arch","uptime"=>"uptime","boot_time"=>"boot","loadavg"=>"load","cpu"=>"CPU","cpu_cores"=>"cores","gpu"=>"GPU","ram"=>"RAM","swap"=>"swap","disk"=>"disk","disks"=>"disks","network"=>"network","local_ip"=>"local ip","public_ip"=>"public ip","battery"=>"battery","temperature"=>"temp","resolution"=>"resolution","shell"=>"shell","terminal"=>"terminal","de"=>"DE","wm"=>"WM","packages"=>"packages","processes"=>"processes","init"=>"init","locale"=>"locale","timezone"=>"timezone","date"=>"date","motherboard"=>"board","bios"=>"bios","display"=>"display","audio"=>"audio","bluetooth"=>"bt","power"=>"power", _ => key
    }.to_string()
}

#[derive(Clone, Debug)]
struct Opts {
    help: bool, version: bool, logo: bool, logo_name: String, logo_file: Option<String>,
    logo_color1: Option<String>, logo_color2: Option<String>,
    accent: Option<String>, color: bool, color_mode: String, theme: String, preset: Option<String>,
    compact: bool, ascii: bool, width: Option<usize>, oneline: bool,
    list_logos: bool, list_fields: bool, list_image_modes: bool, list_presets: bool, doctor: bool,
    json: bool, only: Option<String>, all: bool,
    image: Option<String>, image_mode: String, image_width: Option<usize>, image_height: Option<usize>, image_position: String, image_bg: Option<String>,
    separator: String, bar_style: String, bar_width: usize, format: String,
    cache_ttl: i64, no_cache: bool, no_network: bool, timing: bool,
    save_config: bool, show_config: bool, reset_config: bool, init_config: bool, gen_config: bool,
    set_value: Option<String>, config_file: Option<String>,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            help:false, version:false, logo:true, logo_name:"auto".into(), logo_file:None,
            logo_color1:None, logo_color2:None,
            accent:None, color: render::color_enabled(), color_mode:"auto".into(), theme:"default".into(), preset:None,
            compact:false, ascii:false, width:None, oneline:false,
            list_logos:false, list_fields:false, list_image_modes:false, list_presets:false, doctor:false,
            json:false, only:None, all:false,
            image:None, image_mode:"auto".into(), image_width:None, image_height:None, image_position:"side".into(), image_bg:None,
            separator:": ".into(), bar_style:"blocks".into(), bar_width:12, format:"text".into(),
            cache_ttl:300, no_cache:false, no_network:false, timing:false,
            save_config:false, show_config:false, reset_config:false, init_config:false, gen_config:false,
            set_value:None, config_file:None,
        }
    }
}

fn fail(msg: &str) -> ! {
    eprintln!("nullfetch: {msg}");
    eprintln!("try 'nullfetch --help'");
    std::process::exit(2);
}

fn parse_u(text:&str,name:&str,min:usize)->usize {
    let n = text.parse::<usize>().unwrap_or_else(|_| fail(&format!("invalid {name}: {text}")));
    if n < min { fail(&format!("{name} must be at least {min}: {text}")); }
    n
}

fn parse_args(args: &[String]) -> Opts {
    let mut o = Opts::default();
    let mut i = 0usize;
    while i < args.len() {
        let a = &args[i];
        let (key, val_inline) = if let Some((k,v)) = a.split_once('=') { (k.to_string(), Some(v.to_string())) } else { (a.to_string(), None) };
        let mut take_val = |name: &str| -> String {
            if let Some(v) = val_inline.clone() { return v; }
            i += 1;
            if i >= args.len() { fail(&format!("missing value for {name}")); }
            args[i].clone()
        };
        match key.as_str() {
            "-h"|"--help" => o.help=true,
            "--version" => o.version=true,
            "--no-logo" => { o.logo=false; o.logo_name="none".into(); },
            "--logo" => { let v=take_val("--logo"); o.logo_name=v.clone(); o.logo=v!="none"; if v!="none" { o.logo_file=None; } },
            "--logo-file" => { o.logo_file=Some(take_val("--logo-file")); o.logo=true; },
            "--logo-color-1" => o.logo_color1=Some(take_val("--logo-color-1")),
            "--logo-color-2" => o.logo_color2=Some(take_val("--logo-color-2")),
            "--accent" => o.accent=Some(take_val("--accent")),
            "--color" => { let v=take_val("--color"); match v.as_str(){ "auto"=>{o.color=render::color_enabled(); o.color_mode="auto".into();}, "always"=>{o.color=true; o.color_mode="always".into();}, "never"=>{o.color=false; o.color_mode="never".into();}, _=>fail(&format!("invalid color mode: {v}")) } },
            "--no-color" => { o.color=false; o.color_mode="never".into(); },
            "--theme" => { let v=take_val("--theme"); if !THEMES.contains(&v.as_str()) { fail(&format!("unknown theme: {v}")); } o.theme=v; },
            "--preset" => o.preset=Some(take_val("--preset")),
            "--compact" => o.compact=true,
            "--ascii" => o.ascii=true,
            "--width" => { let v=take_val("--width"); let n=v.parse::<usize>().unwrap_or_else(|_| fail(&format!("invalid width: {v}"))); if n<20 { fail(&format!("width must be at least 20: {v}")); } o.width=Some(n); },
            "--oneline" => o.oneline=true,
            "--list-logos" => o.list_logos=true,
            "--list-fields" => o.list_fields=true,
            "--list-image-modes" => o.list_image_modes=true,
            "--list-presets" => o.list_presets=true,
            "--doctor" => o.doctor=true,
            "--json" => { o.json=true; o.format="json".into(); },
            "-o"|"--only" => o.only=Some(take_val(&key)),
            "--all" => o.all=true,
            "--image"|"--image-file" => o.image=Some(take_val("--image")),
            "--image-mode" => { let v=take_val("--image-mode"); if !IMAGE_MODES.contains(&v.as_str()) { fail(&format!("unknown image mode: {v}")); } o.image_mode=v; },
            "--image-width" => o.image_width=Some(parse_u(&take_val("--image-width"), "image-width", 1)),
            "--image-height" => o.image_height=Some(parse_u(&take_val("--image-height"), "image-height", 1)),
            "--image-position" => { let v=take_val("--image-position"); if !["side","top","bottom"].contains(&v.as_str()) { fail(&format!("unknown image position: {v}")); } o.image_position=v; },
            "--image-bg" => o.image_bg=Some(take_val("--image-bg")),
            "--separator" => o.separator=take_val("--separator"),
            "--bar-width" => o.bar_width=parse_u(&take_val("--bar-width"), "bar-width", 1),
            "--bar-style" => { let v=take_val("--bar-style"); if !["blocks","ascii","arrows","dots","lines"].contains(&v.as_str()) { fail(&format!("unknown bar style: {v}")); } o.bar_style=v; },
            "--format" => { let v=take_val("--format"); if !["text","json","csv","markdown","oneline"].contains(&v.as_str()) { fail(&format!("unknown format: {v}")); } o.format=v.clone(); if v=="json" {o.json=true;} },
            "--cache-ttl" => { let v=take_val("--cache-ttl"); o.cache_ttl=v.parse::<i64>().unwrap_or_else(|_| fail(&format!("invalid cache-ttl: {v}"))); if o.cache_ttl<0 { fail(&format!("cache-ttl must be at least 0: {v}")); } },
            "--no-cache" => o.no_cache=true,
            "--no-network" => o.no_network=true,
            "--timing" => o.timing=true,
            "--save-config" => o.save_config=true,
            "--show-config" => o.show_config=true,
            "--reset-config" => o.reset_config=true,
            "--init-config" => o.init_config=true,
            "--gen-config" => { o.gen_config=true; if let Some(v)=val_inline.clone(){ o.config_file=Some(v); } else if i+1 < args.len() && !args[i+1].starts_with('-') { i+=1; o.config_file=Some(args[i].clone()); } },
            "--set" => o.set_value=Some(take_val("--set")),
            "--config" => o.config_file=Some(take_val("--config")),
            _ => fail(&format!("unknown option: {a}")),
        }
        i += 1;
    }
    o
}

fn normalize_kind(kind: &str) -> Result<String, String> {
    if kind.starts_with("command:") {
        let parts: Vec<&str> = kind.splitn(3,':').collect();
        if parts.len()!=3 || parts[1].trim().is_empty() || parts[2].trim().is_empty() {
            return Err(format!("invalid command field: {kind} (use command:<name>:<cmd>)"));
        }
        return Ok(kind.to_string());
    }
    if ALL_FIELDS.contains(&kind) || ["break","separator","title","custom"].contains(&kind) { Ok(kind.to_string()) }
    else { Err(format!("unknown field: {kind}")) }
}

fn normalize_modules(raw: &[String]) -> Vec<String> {
    raw.iter().map(|item| normalize_kind(item).unwrap_or_else(|e| fail(&e))).collect()
}

fn modules_from_config(cfg: &Value) -> Option<Vec<ModuleSpec>> {
    let arr = cfg.get("modules")?.as_array()?;
    let mut out = vec![];
    for item in arr {
        if let Some(s) = item.as_str() {
            let kind = normalize_kind(s).unwrap_or_else(|e| fail(&e));
            out.push(ModuleSpec::simple(kind));
        } else if let Some(obj) = item.as_object() {
            let kind_raw = obj.get("type").or_else(|| obj.get("field")).or_else(|| obj.get("module")).and_then(|v| v.as_str());
            let Some(kind_raw) = kind_raw else { fail("module object must have string 'type'"); };
            let kind = normalize_kind(kind_raw).unwrap_or_else(|e| fail(&e));
            let key = obj.get("key").and_then(|v| v.as_str()).map(|s| s.to_string());
            let format = obj.get("format").and_then(|v| v.as_str()).map(|s| s.to_string());
            let key_color = obj.get("keyColor").or_else(|| obj.get("key_color")).and_then(|v| v.as_str()).map(|s| s.to_string());
            out.push(ModuleSpec { kind, key, format, key_color });
        } else {
            fail("modules entries must be strings or objects");
        }
    }
    Some(out)
}

fn config_to_args(cfg: &Value) -> (Vec<String>, Option<Vec<ModuleSpec>>) {
    let Some(obj) = cfg.as_object() else { return (vec![], None); };
    let mut args: Vec<String> = vec![];
    let mut config_modules = None;
    for (k,v) in obj {
        match k.as_str() {
            "$schema" => {}
            "modules" => { config_modules = modules_from_config(cfg); }
            "logo" => {
                if let Some(s) = v.as_str() { args.push(format!("--logo={s}")); }
                else if let Some(o) = v.as_object() {
                    if let Some(t) = o.get("type").and_then(|x| x.as_str()) { if t == "none" { args.push("--logo=none".into()); } }
                    if let Some(src) = o.get("source").and_then(|x| x.as_str()) { args.push(format!("--logo-file={src}")); }
                    if let Some(c1) = o.get("color1").and_then(|x| x.as_str()) { args.push(format!("--logo-color-1={c1}")); }
                    if let Some(c2) = o.get("color2").and_then(|x| x.as_str()) { args.push(format!("--logo-color-2={c2}")); }
                }
            }
            "bar" => {
                if let Some(o) = v.as_object() {
                    if let Some(s) = o.get("style").and_then(|x| x.as_str()) { args.push(format!("--bar-style={s}")); }
                    if let Some(n) = o.get("width").and_then(|x| x.as_i64()) { args.push(format!("--bar-width={n}")); }
                }
            }
            "image" => {
                if let Some(s) = v.as_str() { args.push(format!("--image={s}")); }
                else if let Some(o) = v.as_object() {
                    if let Some(src) = o.get("source").or_else(|| o.get("path")).and_then(|x| x.as_str()) { args.push(format!("--image={src}")); }
                    if let Some(m) = o.get("mode").and_then(|x| x.as_str()) { args.push(format!("--image-mode={m}")); }
                    if let Some(n) = o.get("width").and_then(|x| x.as_i64()) { args.push(format!("--image-width={n}")); }
                    if let Some(n) = o.get("height").and_then(|x| x.as_i64()) { args.push(format!("--image-height={n}")); }
                    if let Some(p) = o.get("position").and_then(|x| x.as_str()) { args.push(format!("--image-position={p}")); }
                    if let Some(bg) = o.get("bg").and_then(|x| x.as_str()) { args.push(format!("--image-bg={bg}")); }
                }
            }
            _ => {
                if let Some(b) = v.as_bool() { if b { args.push(format!("--{}", k)); } }
                else if let Some(n) = v.as_i64() { args.push(format!("--{}={}", k, n)); }
                else if let Some(s) = v.as_str() { args.push(format!("--{}={}", k, s)); }
            }
        }
    }
    (args, config_modules)
}

fn apply_preset(o: &mut Opts) {
    let Some(p) = o.preset.as_deref() else { return };
    let modules = match p {
        "minimal" => Some(vec!["os","kernel","cpu","ram"]),
        "full" => { o.all=true; o.only=None; None },
        "dev" => Some(vec!["os","kernel","shell","terminal","de","wm","cpu","ram","disk","packages","processes","init"]),
        "server" => Some(vec!["host","os","kernel","uptime","loadavg","cpu","ram","disk","network","local_ip","processes","init"]),
        _ => fail(&format!("unknown preset: {p} (use minimal, full, dev, server)")),
    };
    if let Some(m)=modules { o.only=Some(m.join(",")); o.all=false; }
}
fn cache_name_for(field: &str) -> String {
    if field.starts_with("command:") {
        let cmd = field.splitn(3,':').nth(2).unwrap_or(field);
        let mut h: u64 = 1469598103934665603;
        for b in cmd.as_bytes() { h ^= *b as u64; h = h.wrapping_mul(1099511628211); }
        format!("cmd-{}-{}", field.splitn(3,':').nth(1).unwrap_or("cmd"), h)
    } else { field.to_string() }
}

fn collect(specs: &[ModuleSpec], cache_ttl: i64, use_cache: bool) -> (BTreeMap<String, modules::Value>, std::collections::HashMap<String, f64>) {
    let data: Arc<Mutex<BTreeMap<String, modules::Value>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let timings: Arc<Mutex<std::collections::HashMap<String, f64>>> = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let mut handles = vec![];
    let mut seen = std::collections::HashSet::new();
    for spec in specs.iter().cloned() {
        let f = spec.kind.clone();
        if matches!(f.as_str(), "break"|"separator"|"custom") { continue; }
        if !seen.insert(f.clone()) { continue; }
        let data = data.clone(); let timings = timings.clone();
        handles.push(thread::spawn(move || {
            let t = Instant::now();
            let ttl = if use_cache { if f.starts_with("command:") || is_slow(&f) { cache_ttl } else { 0 } } else { 0 };
            let cname = cache_name_for(&f);
            let v = if ttl > 0 {
                if let Some(cached) = cache::read(&cname, ttl) { modules::value_from_json(&cached) }
                else {
                    let got = modules::get(&f);
                    if !matches!(got, modules::Value::None) { cache::write(&cname, &modules::value_to_json(&got)); }
                    got
                }
            } else { modules::get(&f) };
            timings.lock().unwrap().insert(f.clone(), t.elapsed().as_secs_f64()*1000.0);
            data.lock().unwrap().insert(f, v);
        }));
    }
    for h in handles { let _ = h.join(); }
    (data.lock().unwrap().clone(), timings.lock().unwrap().clone())
}

fn format_with_template(template: &str, kind: &str, v: &modules::Value) -> String {
    let mut out = template.to_string();
    let text = modules::value_text(v);
    out = out.replace("{type}", kind);
    out = out.replace("{key}", &label(kind));
    out = out.replace("{value}", &text);
    out = out.replace("{text}", &text);
    if let modules::Value::Size{used,total,percent} = v {
        out = out.replace("{used}", &used.to_string());
        out = out.replace("{total}", &total.to_string());
        out = out.replace("{percent}", &percent.to_string());
    }
    if let modules::Value::Info{fields,..} = v {
        for (k,val) in fields {
            out = out.replace(&format!("{{{k}}}"), val);
        }
    }
    // generic {field} lookup
    let mut res = String::new();
    let chars: Vec<char> = out.chars().collect();
    let mut i=0;
    while i < chars.len() {
        if chars[i]=='{' {
            if let Some(end) = chars[i+1..].iter().position(|c| *c=='}') {
                let key: String = chars[i+1..i+1+end].iter().collect();
                if let Some(val)=modules::value_field(v, &key) {
                    res.push_str(&val); i += end+2; continue;
                }
            }
        }
        res.push(chars[i]); i+=1;
    }
    res
}

fn get_logo(opts: &Opts) -> (Vec<String>, bool) {
    if !opts.logo || opts.logo_name=="none" { return (vec![], false); }
    if let Some(path) = &opts.logo_file {
        let p = config::expand_path(path);
        match std::fs::read_to_string(&p) {
            Ok(s) => {
                if s.len() > 100_000 { fail("logo file is too large (maximum: 100 KiB)"); }
                let lines: Vec<String> = s.lines().map(|l| l.to_string()).collect();
                if lines.is_empty() { fail(&format!("logo file is empty: {}", p.display())); }
                if lines.len() > 100 { fail("logo file has too many lines (maximum: 100)"); }
                if lines.iter().any(|l| l.chars().count() > 2000) { fail("logo file has an overlong line (maximum: 2000 characters)"); }
                return (lines, false);
            }
            Err(_) => fail(&format!("cannot read logo file: {}", p.display())),
        }
    }
    let name = if opts.logo_name=="auto" { modules::detect_distro_id() } else { opts.logo_name.clone() };
    let map = logos::logos();
    if opts.logo_name != "auto" && !map.iter().any(|(k,_)| *k==name) && !logos::aliases().contains_key(name.as_str()) {
        let mut names: Vec<&str> = map.iter().map(|(k,_)| *k).collect(); names.sort();
        fail(&format!("unknown logo: {} (available: {})", opts.logo_name, names.join(", ")));
    }
    for (k,v) in &map { if *k == name { return (v.lines().map(|l| l.to_string()).collect(), false); } }
    let aliases = logos::aliases();
    if let Some(real)=aliases.get(name.as_str()) {
        for (k,v) in &map { if k==real { return (v.lines().map(|l| l.to_string()).collect(), false); } }
    }
    let fallback = match std::env::consts::OS { "macos" => "macos", "windows" => "windows", _ => "linux" };
    for (k,v) in &map { if *k == fallback { return (v.lines().map(|l| l.to_string()).collect(), false); } }
    for (k,v) in &map { if *k == "linux" { return (v.lines().map(|l| l.to_string()).collect(), false); } }
    (vec![], false)
}

fn parse_bg(s: Option<&str>) -> [u8;3] {
    let Some(v)=s else { return [0,0,0]; };
    render::parse_hex_color(v).or_else(|| render::named_color(v)).map(|(r,g,b)| [r,g,b]).unwrap_or([0,0,0])
}

fn truncate_visible(s: &str, max: usize, ascii: bool) -> String {
    let v = render::visible_len(s);
    if v <= max { return s.to_string(); }
    let ell = if ascii { "..." } else { "…" };
    let ell_len = if ascii {3} else {1};
    if max <= ell_len { return ell.chars().take(max).collect(); }
    let target = max - ell_len;
    let mut out = String::new();
    let mut n = 0usize;
    let mut chars = s.chars().peekable();
    while let Some(c)=chars.next() {
        if c=='\x1b' {
            out.push(c);
            if chars.peek()==Some(&'[') { out.push(chars.next().unwrap()); while let Some(&x)=chars.peek() { out.push(chars.next().unwrap()); if x.is_ascii_alphabetic() { break; } } }
            continue;
        }
        if n >= target { break; }
        out.push(c); n+=1;
    }
    out.push_str(ell);
    out
}

fn render_text(specs: &[ModuleSpec], data: &BTreeMap<String, modules::Value>, opts: &Opts) {
    let accent = render::resolve_accent(opts.accent.as_deref(), &opts.theme);
    let (mut logo, _precolored) = get_logo(opts);
    let info = render::render_info_lines_spec(specs, data, opts, accent);

    if let Some(path)=&opts.image {
        let (c,l)=image::image_dims(opts.image_width, opts.image_height);
        match image::render(&opts.image_mode, path, c, l, parse_bg(opts.image_bg.as_deref())) {
            Ok(img) => {
                if opts.image_position=="top" {
                    for l in &img.lines { println!("{}", truncate_visible(l, opts.width.unwrap_or(usize::MAX), opts.ascii)); }
                    for l in &info { println!("{}", truncate_visible(l, opts.width.unwrap_or(usize::MAX), opts.ascii)); }
                    return;
                }
                if opts.image_position=="bottom" {
                    for l in &info { println!("{}", truncate_visible(l, opts.width.unwrap_or(usize::MAX), opts.ascii)); }
                    for l in &img.lines { println!("{}", truncate_visible(l, opts.width.unwrap_or(usize::MAX), opts.ascii)); }
                    return;
                }
                logo = img.lines;
            }
            Err(e)=>{ eprintln!("nullfetch: {e}"); std::process::exit(2); }
        }
    }

    let width = opts.width.unwrap_or_else(|| term_width().0);
    if logo.is_empty() {
        for l in info { println!("{}", truncate_visible(&l, width, opts.ascii)); }
        return;
    }
    let logo_w = logo.iter().map(|l| render::visible_len(l)).max().unwrap_or(0);
    let gap = 2usize;
    let stacked = width < logo_w + gap + 20;
    if stacked {
        for l in &logo { println!("{}", truncate_visible(l, width, opts.ascii)); }
        if !opts.compact { println!(); }
        for l in &info { println!("{}", truncate_visible(l, width, opts.ascii)); }
        return;
    }
    let info_w = width.saturating_sub(logo_w + gap);
    let max_lines = logo.len().max(info.len());
    for i in 0..max_lines {
        let l = logo.get(i).map(|s| s.as_str()).unwrap_or("");
        let r = info.get(i).map(|s| s.as_str()).unwrap_or("");
        let lpad = render::pad_visible(l, logo_w);
        if r.is_empty() { println!("{}", truncate_visible(lpad.trim_end(), width, opts.ascii)); }
        else { let r = truncate_visible(r, info_w, opts.ascii); println!("{}{}{}", lpad, " ".repeat(gap), r); }
    }
}

fn term_width() -> (usize, usize) {
    #[cfg(unix)]
    if let Ok(out) = std::process::Command::new("stty").args(["size"]).output() {
        let s = String::from_utf8_lossy(&out.stdout);
        let mut it = s.split_whitespace();
        let rows = it.next().and_then(|x| x.parse().ok()).unwrap_or(0usize);
        let cols = it.next().and_then(|x| x.parse().ok()).unwrap_or(0usize);
        if cols > 0 && rows > 0 { return (cols, rows); }
    }
    if let (Ok(c), Ok(r)) = (std::env::var("COLUMNS"), std::env::var("LINES")) {
        if let (Ok(c), Ok(r)) = (c.parse::<usize>(), r.parse::<usize>()) { if c>0 && r>0 { return (c,r); } }
    }
    #[cfg(windows)]
    if let Ok(out) = std::process::Command::new("powershell").args(["-NoProfile","-Command","$h=$Host.UI.RawUI.WindowSize; \"$($h.Width) $($h.Height)\""]).output() {
        let s = String::from_utf8_lossy(&out.stdout);
        let mut it = s.split_whitespace();
        let cols = it.next().and_then(|x| x.parse().ok()).unwrap_or(80usize);
        let rows = it.next().and_then(|x| x.parse().ok()).unwrap_or(24usize);
        if cols > 0 && rows > 0 { return (cols, rows); }
    }
    (80,24)
}

fn print_json(specs: &[ModuleSpec], data: &BTreeMap<String, modules::Value>) {
    let mut m = Map::new();
    let default_only = specs.len()==FIELDS.len() && specs.iter().zip(FIELDS.iter()).all(|(a,b)| a.kind==*b);
    let kinds: Vec<String> = if default_only { FIELDS.iter().map(|s|s.to_string()).collect() } else { specs.iter().map(|s|s.kind.clone()).collect() };
    for k in kinds {
        let v = data.get(&k).cloned().unwrap_or(modules::Value::Text("?".into()));
        m.insert(k, modules::value_to_json(&v));
    }
    println!("{}", serde_json::to_string_pretty(&Value::Object(m)).unwrap());
}

fn print_csv(specs: &[ModuleSpec], data: &BTreeMap<String, modules::Value>) {
    println!("field,value");
    for sp in specs {
        let v = data.get(&sp.kind).cloned().unwrap_or(modules::Value::Text("?".into()));
        let txt = if let Some(t)=&sp.format { format_with_template(t,&sp.kind,&v) } else { modules::value_text(&v) };
        println!("\"{}\",\"{}\"", sp.display_key().replace('"', "\"\""), txt.replace('"', "\"\""));
    }
}

fn print_md(specs: &[ModuleSpec], data: &BTreeMap<String, modules::Value>) {
    println!("| Field | Value |");
    println!("|---|---|");
    for sp in specs {
        let v = data.get(&sp.kind).cloned().unwrap_or(modules::Value::Text("?".into()));
        let txt = if let Some(t)=&sp.format { format_with_template(t,&sp.kind,&v) } else { modules::value_text(&v) };
        println!("| {} | {} |", sp.display_key().replace('|', "\\|"), txt.replace('|', "\\|"));
    }
}

fn print_oneline(specs: &[ModuleSpec], data: &BTreeMap<String, modules::Value>, ascii: bool) {
    let sep = if ascii { " | " } else { " · " };
    let mut parts = vec![];
    for sp in specs {
        let v = data.get(&sp.kind).cloned().unwrap_or(modules::Value::Text("?".into()));
        let txt = if let Some(t)=&sp.format { format_with_template(t,&sp.kind,&v) } else { match &v { modules::Value::Size{used,total,percent}=>format!("{used}/{total}GiB {percent}%"), _=>modules::value_text(&v) } };
        parts.push(format!("{}: {}", sp.display_key(), txt));
    }
    println!("{}", parts.join(sep));
}

fn print_doctor(opts:&Opts) {
    println!("version: {VERSION}");
    println!("platform: {} {}", std::env::consts::OS, std::env::consts::ARCH);
    println!("term: {}", std::env::var("TERM").unwrap_or_default());
    println!("colorterm: {}", std::env::var("COLORTERM").unwrap_or_default());
    println!("no_color: {}", std::env::var_os("NO_COLOR").is_some());
    println!("color: {}", opts.color);
    println!("config: {}", config::config_path(opts.config_file.as_deref()).display());
    println!("cache: {}", config::cache_dir().display());
    println!("rust_core: yes");
    println!("image_modes: {}", IMAGE_MODES.join(","));
    for cmd in ["chafa","viu","jp2a","img2txt","timg"] {
        let ok = std::process::Command::new("sh").args(["-c", &format!("command -v {cmd} >/dev/null 2>&1")]).status().map(|s|s.success()).unwrap_or(false);
        println!("{cmd}: {}", if ok {"yes"} else {"no"});
    }
}

fn set_config(opts: &Opts, text: &str) -> Result<(), String> {
    let Some((k,v)) = text.split_once('=') else { return Err(format!("--set needs KEY=VALUE: {text}")); };
    let path = config::config_path(opts.config_file.as_deref());
    let mut data = config::load_config(&path)?;
    let obj = data.as_object_mut().ok_or("invalid config")?;
    match k {
        "modules" => { let mods = normalize_modules(&v.split(',').map(|s|s.trim().to_string()).filter(|s|!s.is_empty()).collect::<Vec<_>>()); obj.insert(k.to_string(), json!(mods)); }
        "cache-ttl"|"bar-width"|"image-width"|"image-height" => { let n=v.parse::<i64>().map_err(|_| format!("invalid {k}: {v}"))?; obj.insert(k.to_string(), json!(n)); }
        "ascii"|"compact"|"oneline"|"all"|"no-cache"|"no-network"|"timing" => { let b = matches!(v, "true"|"yes"|"1"); obj.insert(k.to_string(), json!(b)); }
        _ => { obj.insert(k.to_string(), json!(v)); }
    }
    config::write_config(&path, &data)?;
    println!("set {k} = {v}");
    Ok(())
}

fn print_help() {
    println!("nullfetch {VERSION} - PC info\n\nUsage:\n  nullfetch [options]\n\nOptions:\n  -h, --help          show this help\n      --version       show version\n      --no-logo       hide logo\n      --logo NAME     logo name or auto/none\n      --logo-file F   custom ASCII logo file\n      --image FILE    show image\n      --image-mode M  {}\n      --image-width N image width\n      --image-height N image height\n      --image-position P side/top/bottom\n      --image-bg C    image background\n      --accent COLOR  accent color\n      --color MODE    auto/always/never\n      --no-color      disable color\n      --theme NAME    theme\n      --compact       compact layout\n      --all           all modules\n      --preset NAME   minimal/full/dev/server\n      --doctor        diagnostics\n  -o, --only FIELDS   comma separated fields\n      --format F      text,json,csv,markdown,oneline\n      --json          JSON output\n      --oneline       one line\n      --separator S   separator\n      --bar-width N   bar width\n      --bar-style S   blocks/ascii/arrows/dots/lines\n      --cache-ttl N   cache ttl seconds\n      --no-cache      ignore cache\n      --no-network    skip public_ip/local_ip/network\n      --timing        timing to stderr\n      --list-fields   list fields\n      --list-presets  list presets\n      --list-logos    list logos\n      --list-image-modes list image modes\n      --save-config   save defaults\n      --show-config   show config\n      --reset-config  reset config\n      --init-config   write example config\n      --gen-config [F] write config (alias)\n      --set K=V       set config\n      --config FILE   config file\n\nConfig:\n  JSON/JSONC. modules can be strings or objects:\n  {{\"type\":\"cpu\",\"key\":\"CPU\",\"format\":\"{{value}}\",\"keyColor\":\"#ff0\"}}\n\nFields:\n  {} (default)\n  {} (extra)\n  command:<name>:<cmd> (custom)\n\nExamples:\n  nullfetch --preset dev\n  nullfetch --only cpu,ram,command:git:git\\ branch\\ --show-current\n  nullfetch --all --format markdown\n  nullfetch --gen-config ~/.config/nullfetch/config.jsonc", IMAGE_MODES.join(", "), FIELDS.join(" "), EXTRA_FIELDS.join(" "));
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let pre = parse_args(&args);
    if pre.reset_config { let p=config::config_path(pre.config_file.as_deref()); let _=std::fs::remove_file(&p); println!("reset {}", p.display()); return ExitCode::SUCCESS; }
    if pre.show_config { let p=config::config_path(pre.config_file.as_deref()); match config::load_config(&p){Ok(v)=>{ println!("config: {}", p.display()); println!("{}", serde_json::to_string_pretty(&v).unwrap_or_else(|_| "{}".into())); }, Err(e)=>fail(&e)}; return ExitCode::SUCCESS; }
    if pre.init_config || pre.gen_config { let p=config::config_path(pre.config_file.as_deref()); if p.exists() { fail(&format!("config already exists: {}; use --set or --reset-config", p.display())); } let sample=config::default_config(); if let Err(e)=config::write_config(&p,&sample){fail(&e)} println!("config written: {}", p.display()); return ExitCode::SUCCESS; }
    if let Some(sv)=pre.set_value.clone() { if let Err(e)=set_config(&pre, &sv){fail(&e)} return ExitCode::SUCCESS; }

    let cfg_path = config::config_path(pre.config_file.as_deref());
    let mut config_modules: Option<Vec<ModuleSpec>> = None;
    let mut opts = if cfg_path.exists() || config::legacy_config_path(pre.config_file.as_deref()).is_some() {
        match config::load_config(&cfg_path) {
            Ok(cfg) => { let (cfg_args, mods)=config_to_args(&cfg); config_modules=mods; let mut merged=cfg_args; merged.extend(args.clone()); parse_args(&merged) },
            Err(e) => fail(&e),
        }
    } else {
        parse_args(&args)
    };

    apply_preset(&mut opts);
    if opts.logo_name == "none" { opts.logo = false; }
    if opts.color_mode == "never" { opts.color = false; }
    if opts.color_mode == "always" { opts.color = true; }

    if opts.help { print_help(); return ExitCode::SUCCESS; }
    if opts.version { println!("nullfetch {VERSION}"); return ExitCode::SUCCESS; }
    if opts.list_fields { for f in ALL_FIELDS { println!("{f}"); } return ExitCode::SUCCESS; }
    if opts.list_logos { let mut names: Vec<&str>=logos::logos().iter().map(|(k,_)|*k).collect(); names.sort(); for n in names { println!("{n}"); } return ExitCode::SUCCESS; }
    if opts.list_presets { println!("minimal: os,kernel,cpu,ram\nfull: *\ndev: os,kernel,shell,terminal,de,wm,cpu,ram,disk,packages,processes,init\nserver: host,os,kernel,uptime,loadavg,cpu,ram,disk,network,local_ip,processes,init"); return ExitCode::SUCCESS; }
    if opts.list_image_modes { for m in IMAGE_MODES { println!("{m}"); } return ExitCode::SUCCESS; }
    if opts.doctor { print_doctor(&opts); return ExitCode::SUCCESS; }

    let mut specs: Vec<ModuleSpec> = if opts.all {
        ALL_FIELDS.iter().map(|s| ModuleSpec::simple((*s).to_string())).collect()
    } else if let Some(only)=&opts.only {
        normalize_modules(&only.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>())
            .into_iter().map(ModuleSpec::simple).collect()
    } else if opts.oneline || opts.format=="oneline" {
        vec!["user","host","os","kernel","ram","disk"].into_iter().map(|s| ModuleSpec::simple(s.to_string())).collect()
    } else if let Some(cfg_specs)=config_modules.as_ref() {
        cfg_specs.clone()
    } else {
        FIELDS.iter().map(|s| ModuleSpec::simple((*s).to_string())).collect()
    };
    if opts.no_network { specs.retain(|s| s.kind!="public_ip" && s.kind!="local_ip" && s.kind!="network"); }
    let specs = specs;

    let started = Instant::now();
    let (data, timings) = collect(&specs, opts.cache_ttl, !opts.no_cache);
    if opts.timing {
        let elapsed = started.elapsed().as_secs_f64()*1000.0;
        let mut tv: Vec<_> = timings.iter().collect();
        tv.sort_by(|a,b| b.1.partial_cmp(a.1).unwrap());
        let slow: Vec<String> = tv.iter().take(3).map(|(k,v)| format!("{}={:.0}ms", label(k), v)).collect();
        let suffix = if slow.is_empty(){String::new()} else { format!(" slowest: {}", slow.join(", ")) };
        eprintln!("nullfetch: {} fields in {:.1} ms{}", specs.len(), elapsed, suffix);
    }

    if opts.save_config {
        let p = config::config_path(opts.config_file.as_deref());
        let mut save = json!({
            "logo": if opts.logo { opts.logo_name.clone() } else { "none".into() },
            "theme": opts.theme,
            "color": opts.color_mode,
            "separator": opts.separator,
            "bar-style": opts.bar_style,
            "bar-width": opts.bar_width,
            "format": opts.format,
            "cache-ttl": opts.cache_ttl,
        });
        let obj = save.as_object_mut().unwrap();
        if let Some(a)=&opts.accent { obj.insert("accent".into(), json!(a)); }
        if let Some(f)=&opts.logo_file { obj.insert("logo-file".into(), json!(f)); }
        if let Some(im)=&opts.image { obj.insert("image".into(), json!(im)); }
        if opts.image_mode!="auto" { obj.insert("image-mode".into(), json!(opts.image_mode)); }
        if opts.image_position!="side" { obj.insert("image-position".into(), json!(opts.image_position)); }
        if let Some(bg)=&opts.image_bg { obj.insert("image-bg".into(), json!(bg)); }
        if let Some(w)=opts.image_width { obj.insert("image-width".into(), json!(w)); }
        if let Some(h)=opts.image_height { obj.insert("image-height".into(), json!(h)); }
        if opts.ascii { obj.insert("ascii".into(), json!(true)); }
        if opts.compact { obj.insert("compact".into(), json!(true)); }
        let mod_names: Vec<String> = specs.iter().map(|s| s.kind.clone()).collect();
        if opts.all { obj.insert("all".into(), json!(true)); obj.insert("modules".into(), json!(mod_names)); }
        else if opts.only.is_some() || config_modules.is_some() { obj.insert("modules".into(), json!(mod_names)); }
        if let Some(preset)=&opts.preset { obj.insert("preset".into(), json!(preset)); }
        if let Err(e)=config::write_config(&p,&save) { fail(&e); }
        println!("saved defaults: {}", p.display());
        return ExitCode::SUCCESS;
    }

    if opts.json || opts.format=="json" { print_json(&specs, &data); }
    else if opts.format=="csv" { print_csv(&specs, &data); }
    else if opts.format=="markdown" { print_md(&specs, &data); }
    else if opts.oneline || opts.format=="oneline" { print_oneline(&specs, &data, opts.ascii); }
    else { render_text(&specs, &data, &opts); }

    ExitCode::SUCCESS
}
