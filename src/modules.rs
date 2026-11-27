use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug)]
pub enum Value {
    Text(String),
    Size { used: f64, total: f64, percent: i64 },
    None,
}

impl Value {
    pub fn as_string(&self) -> String {
        match self {
            Value::Text(s) => s.clone(),
            Value::Size { used, total, percent } => format!("{}/{ } GiB ({}%)", used, total, percent),
            Value::None => "?".to_string(),
        }
    }
}

trait StrExt { fn if_empty(self, d:String)->String; }
impl StrExt for String { fn if_empty(self,d:String)->String { if self.trim().is_empty(){d} else {self} } }
fn read(path: &str) -> String { fs::read_to_string(path).unwrap_or_default() }
fn cmd(prog: &str, args: &[&str]) -> String {
    Command::new(prog).args(args).output().ok().and_then(|o| {
        let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if s.is_empty() { Some(String::from_utf8_lossy(&o.stderr).trim().to_string()) } else { Some(s) }
    }).unwrap_or_default()
}

pub fn get(field: &str) -> Value {
    let s = match field {
        "user" => std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "?".into()),
        "host" => hostname(),
        "os" => os_name(),
        "kernel" => kernel(),
        "arch" => std::env::consts::ARCH.to_string(),
        "uptime" => uptime(),
        "boot_time" => boot_time(),
        "loadavg" => loadavg(),
        "cpu" => cpu(),
        "cpu_cores" => std::thread::available_parallelism().map(|n| n.get().to_string()).unwrap_or("?".into()),
        "gpu" => gpu(),
        "ram" => return ram(),
        "swap" => return swap(),
        "disk" => return disk("/"),
        "disks" => disks(),
        "network" => network(),
        "local_ip" => local_ip(),
        "public_ip" => "?".into(),
        "battery" => battery(),
        "temperature" => temperature(),
        "resolution" => resolution(),
        "shell" => shell(),
        "terminal" => terminal(),
        "de" => std::env::var("XDG_CURRENT_DESKTOP").or_else(|_| std::env::var("DESKTOP_SESSION")).unwrap_or("?".into()),
        "wm" => wm(),
        "packages" => packages(),
        "processes" => processes(),
        "init" => init(),
        "locale" => std::env::var("LANG").or_else(|_| std::env::var("LC_ALL")).unwrap_or("?".into()),
        "timezone" => timezone(),
        "date" => date_now(),
        "motherboard" => motherboard(),
        "bios" => bios(),
        "display" => display(),
        "audio" => audio(),
        "bluetooth" => bluetooth(),
        "power" => power(),
        _ if field.starts_with("command:") => command_field(field),
        _ => "?".into(),
    };
    Value::Text(s)
}

fn hostname() -> String {
    if let Ok(s) = fs::read_to_string("/etc/hostname") {
        let t = s.trim().to_string();
        if !t.is_empty() { return t; }
    }
    if let Ok(o) = Command::new("hostname").output() {
        let t = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if !t.is_empty() { return t; }
    }
    "?".into()
}
fn os_name() -> String {
    if let Ok(s) = fs::read_to_string("/etc/os-release") {
        for line in s.lines() {
            if let Some(v) = line.strip_prefix("PRETTY_NAME=") { return v.trim_matches('"').to_string(); }
        }
    }
    if cfg!(windows) { return "Windows".into(); }
    std::env::consts::OS.to_string()
}

pub fn detect_distro_id() -> String {
    if let Ok(s) = fs::read_to_string("/etc/os-release") {
        let mut id = String::new();
        let mut id_like = String::new();
        for line in s.lines() {
            if let Some(v) = line.strip_prefix("ID=") { id = v.trim_matches('"').to_lowercase(); }
            if let Some(v) = line.strip_prefix("ID_LIKE=") { id_like = v.trim_matches('"').to_lowercase(); }
        }
        if !id.is_empty() { return id; }
        if let Some(first) = id_like.split_whitespace().next() { return first.to_string(); }
    }
    std::env::consts::OS.to_string()
}
fn kernel() -> String { read("/proc/sys/kernel/osrelease").trim().to_string().if_empty("?".into()) }
fn uptime() -> String {
    let s = read("/proc/uptime");
    let secs: f64 = s.split_whitespace().next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    fmt_dur(secs as u64)
}
fn fmt_dur(secs: u64) -> String {
    let d = secs / 86400; let h = (secs % 86400) / 3600; let m = (secs % 3600) / 60;
    let mut out = vec![];
    if d>0 {out.push(format!("{d}d"));} if h>0 {out.push(format!("{h}h"));} out.push(format!("{m}m")); out.join(" ")
}
fn boot_time() -> String {
    let s = read("/proc/stat");
    for line in s.lines() {
        if let Some(v) = line.strip_prefix("btime ") {
            if let Ok(ts) = v.trim().parse::<i64>() {
                // format as UTC-ish local via `date` fallback avoided; keep epoch->local using libc localtime
                unsafe extern "C" { fn localtime(t: *const i64) -> *mut tm; }
                #[repr(C)] struct tm { tm_sec:i32,tm_min:i32,tm_hour:i32,tm_mday:i32,tm_mon:i32,tm_year:i32,tm_wday:i32,tm_yday:i32,tm_isdst:i32,tm_gmtoff:i64,tm_zone:*const i8 }
                unsafe {
                    let p = localtime(&ts as *const i64);
                    if !p.is_null() {
                        let t = &*p;
                        return format!("{:04}-{:02}-{:02} {:02}:{:02}", t.tm_year+1900, t.tm_mon+1, t.tm_mday, t.tm_hour, t.tm_min);
                    }
                }
            }
        }
    }
    "?".into()
}
fn loadavg() -> String {
    let s = read("/proc/loadavg");
    let p: Vec<&str> = s.split_whitespace().collect();
    if p.len()>=3 { format!("{} {} {}", p[0],p[1],p[2]) } else { "?".into() }
}
fn cpu() -> String {
    let s = read("/proc/cpuinfo");
    for line in s.lines() {
        if line.to_lowercase().starts_with("model name") {
            if let Some((_,v)) = line.split_once(':') { return v.trim().to_string(); }
        }
    }
    std::env::consts::ARCH.to_string()
}
fn gpu() -> String {
    // fast path: /sys/class/drm
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        let mut names = vec![];
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if !name.starts_with("card") || name.contains('-') { continue; }
            let dev = e.path().join("device");
            let vendor = read(dev.join("vendor").to_str().unwrap_or("")).trim().to_string();
            let device = read(dev.join("device").to_str().unwrap_or("")).trim().to_string();
            if vendor.is_empty() || device.is_empty() { continue; }
            let label = pci_lookup(&vendor, &device).unwrap_or_else(|| format!("{vendor}:{device}"));
            if !names.contains(&label) { names.push(label); }
        }
        if !names.is_empty() { return names.join(", "); }
    }
    let out = cmd("sh", &["-c", "lspci 2>/dev/null | grep -Ei 'vga|3d|display' | head -n1 | sed 's/.*: //' "]);
    if !out.is_empty() { return out.lines().next().unwrap_or("?").to_string(); }
    "?".into()
}
fn pci_lookup(vendor: &str, device: &str) -> Option<String> {
    let v = vendor.trim_start_matches("0x").to_lowercase();
    let d = device.trim_start_matches("0x").to_lowercase();
    let db = read("/usr/share/hwdata/pci.ids");
    if db.is_empty() { return None; }
    let mut in_vendor = false;
    for line in db.lines() {
        if line.starts_with('#') || line.trim().is_empty() { continue; }
        if !line.starts_with('\t') {
            in_vendor = line.starts_with(&v);
            continue;
        }
        if in_vendor && line.starts_with('\t') && !line.starts_with("\t\t") {
            let l = line.trim_start();
            if l.starts_with(&d) {
                let name = l[d.len()..].trim();
                if !name.is_empty() { return Some(name.to_string()); }
            }
        }
    }
    None
}

fn meminfo() -> HashMap<String,u64> {
    let mut m=HashMap::new();
    for line in read("/proc/meminfo").lines() {
        if let Some((k,v)) = line.split_once(':') {
            let n = v.split_whitespace().next().and_then(|x| x.parse::<u64>().ok()).unwrap_or(0);
            m.insert(k.trim().to_string(), n);
        }
    }
    m
}
fn size_value(used_kb: u64, total_kb: u64) -> Value {
    let total = (total_kb as f64 / 1024.0 / 1024.0 * 10.0).round()/10.0;
    let used = (used_kb as f64 / 1024.0 / 1024.0 * 10.0).round()/10.0;
    let pct = if total_kb>0 { ((used_kb as f64/total_kb as f64)*100.0).round() as i64 } else {0};
    Value::Size{used,total,percent:pct}
}
fn ram() -> Value { let m=meminfo(); let t=m.get("MemTotal").copied().unwrap_or(0); let a=m.get("MemAvailable").copied().unwrap_or_else(|| m.get("MemFree").copied().unwrap_or(0)); if t==0 {Value::Text("?".into())} else {size_value(t-a,t)} }
fn swap() -> Value { let m=meminfo(); let t=m.get("SwapTotal").copied().unwrap_or(0); let f=m.get("SwapFree").copied().unwrap_or(0); if t==0 {Value::Text("none".into())} else {size_value(t-f,t)} }
fn disk(path: &str) -> Value {
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::mem::MaybeUninit;
        unsafe extern "C" { fn statvfs(path: *const i8, buf: *mut libc_statvfs) -> i32; }
        #[repr(C)] struct libc_statvfs { f_bsize: u64, f_frsize: u64, f_blocks: u64, f_bfree: u64, f_bavail: u64, f_files: u64, f_ffree: u64, f_favail: u64, f_fsid: u64, f_flag: u64, f_namemax: u64, __f_spare: [i32;6] }
        let c = match CString::new(path) { Ok(c)=>c, Err(_)=>return Value::Text("?".into()) };
        unsafe {
            let mut buf = MaybeUninit::<libc_statvfs>::uninit();
            if statvfs(c.as_ptr(), buf.as_mut_ptr()) == 0 {
                let b = buf.assume_init();
                let total = b.f_blocks.saturating_mul(b.f_frsize) / 1024;
                let avail = b.f_bavail.saturating_mul(b.f_frsize) / 1024;
                let used = total.saturating_sub(avail);
                return size_value(used, total);
            }
        }
    }
    let out = Command::new("df").args(["-Pk", path]).output().ok();
    if let Some(o)=out { let s=String::from_utf8_lossy(&o.stdout); if let Some(line)=s.lines().nth(1) { let p:Vec<&str>=line.split_whitespace().collect(); if p.len()>=4 { let t=p[1].parse().unwrap_or(0); let u=p[2].parse().unwrap_or(0); return size_value(u,t); } } }
    Value::Text("?".into())
}
fn disks() -> String {
    let m = read("/proc/mounts");
    let mut out = vec![];
    for line in m.lines() {
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len()<3 { continue; }
        let fs = p[2]; let mp = p[1];
        if matches!(fs, "tmpfs"|"devtmpfs"|"squashfs"|"overlay"|"proc"|"sysfs"|"cgroup2"|"efivarfs") { continue; }
        if !p[0].starts_with('/') { continue; }
        if let Value::Size{used,total,percent}=disk(mp) {
            out.push(format!("{mp}: {used}/{total} GiB ({percent}%)"));
        }
        if out.len()>=5 { break; }
    }
    if out.is_empty(){"?".into()} else {out.join(", ")}
}
fn network() -> String {
    let ip = local_ip();
    let iface = default_iface().unwrap_or_default();
    if !iface.is_empty() && ip != "?" { format!("{iface} ({ip})") } else if ip != "?" { ip } else { "?".into() }
}
fn default_iface() -> Option<String> {
    let s = read("/proc/net/route");
    for line in s.lines().skip(1) {
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len()>=3 && p[1]=="00000000" && p[2]!="00000000" { return Some(p[0].to_string()); }
    }
    None
}
fn local_ip() -> String {
    // parse ip -4 route get via /proc/net/route + get local addr from /sys not available; fallback to command but cache? keep simple: use `ip` once
    let out = cmd("ip", &["-4","route","get","1.1.1.1"]);
    for tok in out.split_whitespace().collect::<Vec<_>>().windows(2) { if tok[0]=="src" { return tok[1].to_string(); } }
    "?".into()
}
fn battery() -> String {
    let base = Path::new("/sys/class/power_supply");
    if let Ok(entries) = fs::read_dir(base) {
        for e in entries.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if !n.starts_with("BAT") { continue; }
            let cap = read(&format!("{}/capacity", e.path().display())).trim().to_string();
            let status = read(&format!("{}/status", e.path().display())).trim().to_string();
            if !cap.is_empty() { return if status.is_empty(){format!("{cap}%")} else {format!("{cap}% ({status})")}; }
        }
    }
    "?".into()
}
fn temperature() -> String {
    if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
        for e in entries.flatten() {
            let p = e.path();
            let t = read(&format!("{}/temp", p.display())).trim().to_string();
            if let Ok(v) = t.parse::<f64>() { if v>1000.0 { return format!("{:.1}°C", v/1000.0); } }
        }
    }
    let out = cmd("sh", &["-c", "sensors 2>/dev/null | awk '/Package id 0|Tctl|temp1/ {gsub(/[+°C]/,\"\",$0); for(i=1;i<=NF;i++) if($i ~ /[0-9]+\\.[0-9]/){print $i; exit}}'"]);
    if !out.is_empty() { return format!("{}°C", out.lines().next().unwrap_or("?")); }
    "?".into()
}
fn resolution() -> String {
    if let Ok(entries)=fs::read_dir("/sys/class/drm") {
        for e in entries.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if !n.contains('-') { continue; }
            let modes = read(e.path().join("modes").to_str().unwrap_or(""));
            if let Some(first)=modes.lines().next() { let t=first.trim(); if !t.is_empty() { return t.to_string(); } }
        }
    }
    let out = cmd("sh", &["-c", "xrandr --current 2>/dev/null | awk '/\\*/{print $1; exit}'"]);
    if !out.is_empty() { return out.lines().next().unwrap_or("?").to_string(); }
    "?".into()
}
fn shell() -> String { std::env::var("SHELL").ok().and_then(|s| Path::new(&s).file_name().map(|n| n.to_string_lossy().to_string())).unwrap_or("?".into()) }
fn terminal() -> String { std::env::var("TERM_PROGRAM").or_else(|_| std::env::var("TERMINAL_EMULATOR")).or_else(|_| std::env::var("TERM")).unwrap_or("?".into()) }
fn wm() -> String {
    if let Ok(w) = std::env::var("WAYLAND_DISPLAY") { if !w.is_empty() { return std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "Wayland".into()); } }
    std::env::var("XDG_SESSION_DESKTOP").or_else(|_| std::env::var("XDG_CURRENT_DESKTOP")).unwrap_or_else(|_| std::env::var("XDG_SESSION_TYPE").unwrap_or("?".into()))
}
fn packages() -> String {
    // fast paths: read package db dirs
    if let Ok(rd)=fs::read_dir("/var/lib/pacman/local") {
        let n = rd.flatten().filter(|e| e.file_type().map(|t|t.is_dir()).unwrap_or(false)).count();
        if n>1 { return format!("{} (pacman)", n.saturating_sub(1)); }
    }
    if let Ok(s)=fs::read_to_string("/var/lib/dpkg/status") {
        let n = s.lines().filter(|l| l.starts_with("Package: ")).count();
        if n>0 { return format!("{n} (dpkg)"); }
    }
    let checks = [("rpm -qa 2>/dev/null | wc -l"," (rpm)")];
    for (c,suf) in checks { let out=cmd("sh", &["-c", c]).trim().to_string(); if let Ok(n)=out.parse::<i64>() { if n>0 { return format!("{n}{suf}"); } } }
    "?".into()
}
fn processes() -> String {
    if let Ok(rd) = fs::read_dir("/proc") { let n = rd.flatten().filter(|e| e.file_name().to_string_lossy().chars().all(|c| c.is_ascii_digit())).count(); return n.to_string(); }
    "?".into()
}
fn init() -> String {
    let c = read("/proc/1/comm").trim().to_string();
    if !c.is_empty() { return c; }
    let cmdline = read("/proc/1/cmdline").replace('\0'," "); let first = cmdline.split_whitespace().next().unwrap_or("?");
    Path::new(first).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or("?".into())
}
fn timezone() -> String {
    if let Ok(tz)=std::env::var("TZ") { if !tz.is_empty() { return tz; } }
    if let Ok(p)=fs::read_link("/etc/localtime") {
        let s=p.to_string_lossy().to_string();
        if let Some(i)=s.find("zoneinfo/") { return s[i+9..].to_string(); }
        return s;
    }
    read("/etc/timezone").trim().to_string().if_empty("?".into())
}
fn date_now() -> String {
    let out = Command::new("date").arg("+%Y-%m-%d %H:%M:%S").output().ok();
    if let Some(o)=out { let t=String::from_utf8_lossy(&o.stdout).trim().to_string(); if !t.is_empty() { return t; } }
    "?".into()
}
fn motherboard() -> String { let v=read("/sys/devices/virtual/dmi/id/board_vendor").trim().to_string(); let n=read("/sys/devices/virtual/dmi/id/board_name").trim().to_string(); let s=format!("{v} {n}").trim().to_string(); if s.is_empty(){"?".into()} else {s} }
fn bios() -> String { let v=read("/sys/devices/virtual/dmi/id/bios_vendor").trim().to_string(); let n=read("/sys/devices/virtual/dmi/id/bios_version").trim().to_string(); let s=format!("{v} {n}").trim().to_string(); if s.is_empty(){"?".into()} else {s} }
fn display() -> String { resolution() }
fn audio() -> String {
    let out=cmd("pactl", &["info"]);
    for line in out.lines() { if let Some(v)=line.strip_prefix("Default Sink:") { let t=v.trim(); if !t.is_empty() { return t.to_string(); } } }
    "?".into()
}
fn bluetooth() -> String {
    let out=cmd("bluetoothctl", &["show"]);
    for line in out.lines() { let l=line.trim(); if let Some(v)=l.strip_prefix("Powered:") { return v.trim().to_string(); } }
    if Path::new("/sys/class/bluetooth").exists() {"off".into()} else {"?".into()}
}
fn power() -> String { let g = read("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor").trim().to_string(); if !g.is_empty(){g} else {battery()} }
fn command_field(field: &str) -> String {
    let parts: Vec<&str> = field.splitn(3, ':').collect();
    if parts.len() != 3 { return "?".into(); }
    let cmdline = parts[2];
    let out = Command::new("sh").args(["-c", cmdline]).output().ok();
    if let Some(o)=out { let s=String::from_utf8_lossy(&o.stdout).trim().to_string(); if !s.is_empty() { return s.lines().next().unwrap_or("?").to_string(); } let e=String::from_utf8_lossy(&o.stderr).trim().to_string(); if !e.is_empty() { return e.lines().next().unwrap_or("?").to_string(); } }
    "?".into()
}
