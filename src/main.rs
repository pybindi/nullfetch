mod config;
mod logos;
mod modules;

use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap};
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

const VERSION: &str = "3.0.0";
const FIELDS: &[&str] = &["user","host","os","kernel","cpu","gpu","ram","disk"];
const ALL_FIELDS: &[&str] = &["user","host","os","kernel","arch","uptime","boot_time","loadavg","cpu","cpu_cores","gpu","ram","swap","disk","disks","network","local_ip","public_ip","battery","temperature","resolution","shell","terminal","de","wm","packages","processes","init","locale","timezone","date","motherboard","bios","display","audio","bluetooth","power"];
const EXTRA_FIELDS: &[&str] = &["arch","uptime","boot_time","loadavg","cpu_cores","swap","disks","network","local_ip","public_ip","battery","temperature","resolution","shell","terminal","de","wm","packages","processes","init","locale","timezone","date","motherboard","bios","display","audio","bluetooth","power"];

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
    accent: Option<String>, color: bool, no_color: bool, theme: String, preset: Option<String>,
    compact: bool, ascii: bool, width: Option<usize>, oneline: bool,
    list_logos: bool, list_fields: bool, list_image_modes: bool, list_presets: bool, doctor: bool,
    json: bool, only: Option<String>, all: bool,
    image: Option<String>, image_mode: String, image_width: Option<usize>, image_height: Option<usize>, image_position: String, image_bg: Option<String>,
    separator: String, bar_style: String, bar_width: usize, format: String,
    cache_ttl: i64, no_cache: bool, no_network: bool, timing: bool,
    save_config: bool, show_config: bool, reset_config: bool, init_config: bool,
    set_value: Option<String>, config_file: Option<String>,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            help:false, version:false, logo:true, logo_name:"auto".into(), logo_file:None,
            accent:None, color: std::env::var_os("NO_COLOR").is_none(), no_color:false, theme:"default".into(), preset:None,
            compact:false, ascii:false, width:None, oneline:false,
            list_logos:false, list_fields:false, list_image_modes:false, list_presets:false, doctor:false,
            json:false, only:None, all:false,
            image:None, image_mode:"auto".into(), image_width:None, image_height:None, image_position:"side".into(), image_bg:None,
            separator:": ".into(), bar_style:"blocks".into(), bar_width:12, format:"text".into(),
            cache_ttl:300, no_cache:false, no_network:false, timing:false,
            save_config:false, show_config:false, reset_config:false, init_config:false,
            set_value:None, config_file:None,
        }
    }
}

fn fail(msg: &str) -> ! { eprintln!("nullfetch: {msg}"); std::process::exit(1); }

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
            "--logo" => { let v=take_val("--logo"); o.logo_name=v.clone(); o.logo=v!="none"; },
            "--logo-file" => { o.logo_file=Some(take_val("--logo-file")); o.logo=true; },
            "--accent" => o.accent=Some(take_val("--accent")),
            "--color" => { let v=take_val("--color"); match v.as_str(){ "auto"=>{o.color=std::env::var_os("NO_COLOR").is_none(); o.no_color=false;}, "always"=>{o.color=true;o.no_color=false;}, "never"=>{o.color=false;o.no_color=true;}, _=>fail(&format!("invalid color mode: {v}")) } },
            "--no-color" => { o.color=false; o.no_color=true; },
            "--theme" => o.theme=take_val("--theme"),
            "--preset" => o.preset=Some(take_val("--preset")),
            "--compact" => o.compact=true,
            "--ascii" => o.ascii=true,
            "--width" => o.width=take_val("--width").parse().ok(),
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
            "--image-mode" => o.image_mode=take_val("--image-mode"),
            "--image-width" => o.image_width=take_val("--image-width").parse().ok(),
            "--image-height" => o.image_height=take_val("--image-height").parse().ok(),
            "--image-position" => o.image_position=take_val("--image-position"),
            "--image-bg" => o.image_bg=Some(take_val("--image-bg")),
            "--separator" => o.separator=take_val("--separator"),
            "--bar-width" => o.bar_width=take_val("--bar-width").parse().unwrap_or(12),
            "--bar-style" => o.bar_style=take_val("--bar-style"),
            "--format" => { o.format=take_val("--format"); if o.format=="json" {o.json=true;} },
            "--cache-ttl" => o.cache_ttl=take_val("--cache-ttl").parse().unwrap_or(300),
            "--no-cache" => o.no_cache=true,
            "--no-network" => o.no_network=true,
            "--timing" => o.timing=true,
            "--save-config" => o.save_config=true,
            "--show-config" => o.show_config=true,
            "--reset-config" => o.reset_config=true,
            "--init-config" => o.init_config=true,
            "--set" => o.set_value=Some(take_val("--set")),
            "--config" => o.config_file=Some(take_val("--config")),
            _ => fail(&format!("unknown option: {a}")),
        }
        i += 1;
    }
    o
}

fn normalize_modules(raw: &[String]) -> Vec<String> {
    let mut out = vec![];
    for item in raw {
        if item.starts_with("command:") {
            let parts: Vec<&str> = item.splitn(3,':').collect();
            if parts.len()!=3 || parts[1].trim().is_empty() || parts[2].trim().is_empty() { fail(&format!("invalid command field: {item} (use command:<name>:<cmd>)")); }
            out.push(item.clone());
        } else if ALL_FIELDS.contains(&item.as_str()) { out.push(item.clone()); }
        else { fail(&format!("unknown field: {item}")); }
    }
    out
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

fn collect(fields: &[String]) -> (BTreeMap<String, modules::Value>, HashMap<String, f64>) {
    let data: Arc<Mutex<BTreeMap<String, modules::Value>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let timings: Arc<Mutex<HashMap<String, f64>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = vec![];
    for f in fields.iter().cloned() {
        let data = data.clone(); let timings = timings.clone();
        handles.push(thread::spawn(move || {
            let t = Instant::now();
            let v = modules::get(&f);
            timings.lock().unwrap().insert(f.clone(), t.elapsed().as_secs_f64()*1000.0);
            data.lock().unwrap().insert(f, v);
        }));
    }
    for h in handles { let _ = h.join(); }
    let data = data.lock().unwrap().clone();
    let timings = timings.lock().unwrap().clone();
    (data, timings)
}

fn render_text(fields: &[String], data: &BTreeMap<String, modules::Value>, opts: &Opts) {
    let mut max_label = 0usize;
    for f in fields { max_label = max_label.max(label(f).len()); }
    for f in fields {
        let v = data.get(f).cloned().unwrap_or(modules::Value::Text("?".into()));
        let val = match v {
            modules::Value::Size{used,total,percent} => format!("{used}/{total} GiB ({percent}%)"),
            modules::Value::Text(s) => s,
            modules::Value::None => "?".into(),
        };
        if opts.compact {
            println!("{}{}{}", label(f), opts.separator, val);
        } else {
            println!("{:>width$}{}{}", label(f), opts.separator, val, width=max_label);
        }
    }
}

fn print_json(fields: &[String], data: &BTreeMap<String, modules::Value>) {
    let mut m = Map::new();
    for f in fields {
        let v = data.get(f).cloned().unwrap_or(modules::Value::Text("?".into()));
        let j = match v {
            modules::Value::Size{used,total,percent} => json!({"used":used,"total":total,"percent":percent}),
            modules::Value::Text(s) => json!(s),
            modules::Value::None => json!("?"),
        };
        m.insert(f.clone(), j);
    }
    println!("{}", serde_json::to_string_pretty(&Value::Object(m)).unwrap());
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut opts = parse_args(&args);

    if opts.help { print_help(); return ExitCode::SUCCESS; }
    if opts.version { println!("nullfetch {VERSION}"); return ExitCode::SUCCESS; }
    if opts.list_fields { for f in ALL_FIELDS { println!("{f}"); } return ExitCode::SUCCESS; }
    if opts.list_logos { for (k,_) in logos::logos() { println!("{k}"); } return ExitCode::SUCCESS; }
    if opts.list_presets { println!("minimal: os,kernel,cpu,ram\nfull: *\ndev: os,kernel,shell,terminal,de,wm,cpu,ram,disk,packages,processes,init\nserver: host,os,kernel,uptime,loadavg,cpu,ram,disk,network,local_ip,processes,init"); return ExitCode::SUCCESS; }
    if opts.list_image_modes { println!("auto\nblocks\nascii\nkitty\niterm\nchafa\nviu\njp2\nimg2txt\ntimg\nsixel"); return ExitCode::SUCCESS; }
    if opts.doctor { print_doctor(); return ExitCode::SUCCESS; }
    if opts.show_config { let p=config::config_path(opts.config_file.as_deref()); match config::load_config(&p){Ok(v)=>println!("{}", serde_json::to_string_pretty(&v).unwrap()), Err(e)=>fail(&e)}; return ExitCode::SUCCESS; }
    if opts.reset_config { let p=config::config_path(opts.config_file.as_deref()); let _=std::fs::remove_file(&p); println!("reset {}", p.display()); return ExitCode::SUCCESS; }
    if opts.init_config { let p=config::config_path(opts.config_file.as_deref()); let sample=json!({"logo":"auto","theme":"default","modules":["os","kernel","cpu","ram","disk"]}); if let Err(e)=config::write_config(&p,&sample){fail(&e)} println!("wrote {}", p.display()); return ExitCode::SUCCESS; }
    if let Some(sv)=opts.set_value.clone() { if let Err(e)=set_config(&opts, &sv){fail(&e)} return ExitCode::SUCCESS; }

    apply_preset(&mut opts);

    let mut fields: Vec<String> = if opts.all { ALL_FIELDS.iter().map(|s| s.to_string()).collect() }
        else if let Some(only)=&opts.only { normalize_modules(&only.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>()) }
        else if opts.oneline || opts.format=="oneline" { vec!["user","host","os","kernel","ram","disk"].into_iter().map(String::from).collect() }
        else { FIELDS.iter().map(|s| s.to_string()).collect() };
    if opts.no_network { fields.retain(|f| f!="public_ip"); }
    let fields = fields;

    let started = Instant::now();
    let (data, timings) = collect(&fields);
    if opts.timing {
        let elapsed = started.elapsed().as_secs_f64()*1000.0;
        let mut tv: Vec<_> = timings.iter().collect();
        tv.sort_by(|a,b| b.1.partial_cmp(a.1).unwrap());
        let slow: Vec<String> = tv.iter().take(3).map(|(k,v)| format!("{}={:.0}ms", label(k), v)).collect();
        let suffix = if slow.is_empty(){String::new()} else { format!(" slowest: {}", slow.join(", ")) };
        eprintln!("nullfetch: {} fields in {:.1} ms{}", fields.len(), elapsed, suffix);
    }

    if opts.json || opts.format=="json" { print_json(&fields, &data); }
    else if opts.format=="csv" { print_csv(&fields, &data); }
    else if opts.format=="markdown" { print_md(&fields, &data); }
    else if opts.oneline || opts.format=="oneline" { print_oneline(&fields, &data, opts.ascii); }
    else { render_text(&fields, &data, &opts); }

    ExitCode::SUCCESS
}

fn print_csv(fields: &[String], data: &BTreeMap<String, modules::Value>) {
    println!("field,value");
    for f in fields {
        let v = data.get(f).map(|x| match x { modules::Value::Size{used,total,percent}=>format!("{used}/{total} GiB ({percent}%)"), modules::Value::Text(s)=>s.clone(), modules::Value::None=>"?".into() }).unwrap_or("?".into());
        let v = v.replace('"',"\"\"");
        println!("{},\"{}\"", label(f).replace(','," "), v);
    }
}
fn print_md(fields: &[String], data: &BTreeMap<String, modules::Value>) {
    println!("| Field | Value |");
    println!("|---|---|");
    for f in fields {
        let v = data.get(f).map(|x| match x { modules::Value::Size{used,total,percent}=>format!("{used}/{total} GiB ({percent}%)"), modules::Value::Text(s)=>s.clone(), modules::Value::None=>"?".into() }).unwrap_or("?".into());
        println!("| {} | {} |", label(f), v.replace('|',"\\|"));
    }
}
fn print_oneline(fields: &[String], data: &BTreeMap<String, modules::Value>, ascii: bool) {
    let sep = if ascii { " | " } else { " · " };
    let mut parts = vec![];
    for f in fields {
        let val = data.get(f).map(|x| match x { modules::Value::Size{used,total,percent}=>format!("{used}/{total}GiB {percent}%"), modules::Value::Text(s)=>s.clone(), modules::Value::None=>"?".into() }).unwrap_or("?".into());
        parts.push(format!("{}: {}", label(f), val));
    }
    println!("{}", parts.join(sep));
}
fn print_doctor() {
    println!("version: {VERSION}");
    println!("platform: {} {}", std::env::consts::OS, std::env::consts::ARCH);
    println!("config: {}", config::config_path(None).display());
    println!("cache: {}", config::cache_dir().display());
    println!("image_module: builtin-rust-todo");
}
fn set_config(opts: &Opts, text: &str) -> Result<(), String> {
    let Some((k,v)) = text.split_once('=') else { return Err(format!("--set needs KEY=VALUE: {text}")); };
    let path = config::config_path(opts.config_file.as_deref());
    let mut data = config::load_config(&path)?;
    let obj = data.as_object_mut().ok_or("invalid config")?;
    if k=="modules" { let mods = normalize_modules(&v.split(',').map(|s|s.trim().to_string()).filter(|s|!s.is_empty()).collect::<Vec<_>>()); obj.insert(k.to_string(), json!(mods)); }
    else { obj.insert(k.to_string(), json!(v)); }
    config::write_config(&path, &data)?;
    println!("set {k} = {v}");
    Ok(())
}
fn print_help() {
    println!("nullfetch {VERSION} - PC info\n\nUsage:\n  nullfetch [options]\n\nOptions:\n  -h, --help          show this help\n      --version       show version\n      --all           show every available module\n      --preset NAME   minimal, full, dev, server\n      --doctor        diagnostics\n  -o, --only FIELDS   comma separated fields\n      --format F      text, json, csv, markdown, oneline\n      --json          JSON output\n      --timing        timing to stderr\n      --no-network    skip public_ip\n      --list-fields   list fields\n      --list-presets  list presets\n      --list-logos    list logos\n      --set K=V       set config\n      --show-config   show config\n      --reset-config  reset config\n      --init-config   write example config\n\nFields:\n  {} (default)\n  {} (extra)", FIELDS.join(" "), EXTRA_FIELDS.join(" "));
}
