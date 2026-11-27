use serde_json::{json, Value as J};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub enum Value {
    Text(String),
    Size { used: f64, total: f64, percent: i64 },
    Info { text: String, fields: std::collections::BTreeMap<String, String> },
    None,
}

pub fn value_to_json(v: &Value) -> J {
    match v {
        Value::Text(s) => json!(s),
        Value::Size { used, total, percent } => json!({"used":used,"total":total,"percent":percent}),
        Value::Info { text, fields } => {
            let mut m = serde_json::Map::new();
            m.insert("text".into(), json!(text));
            for (k,val) in fields { m.insert(k.clone(), json!(val)); }
            J::Object(m)
        }
        Value::None => json!("?"),
    }
}
pub fn value_from_json(v: &J) -> Value {
    if let Some(s) = v.as_str() { return Value::Text(s.to_string()); }
    if let Some(o) = v.as_object() {
        let used = o.get("used").and_then(|x| x.as_f64());
        let total = o.get("total").and_then(|x| x.as_f64());
        let percent = o.get("percent").and_then(|x| x.as_i64());
        if let (Some(u),Some(t),Some(p)) = (used,total,percent) {
            if o.len()==3 { return Value::Size{used:u,total:t,percent:p}; }
        }
        if let Some(text) = o.get("text").and_then(|x| x.as_str()) {
            let mut fields = std::collections::BTreeMap::new();
            for (k,val) in o { if k=="text" || k=="used"||k=="total"||k=="percent" { continue; } if let Some(s)=val.as_str(){ fields.insert(k.clone(), s.to_string()); } else { fields.insert(k.clone(), val.to_string()); } }
            if let (Some(u),Some(t),Some(p)) = (used,total,percent) { fields.insert("used".into(), u.to_string()); fields.insert("total".into(), t.to_string()); fields.insert("percent".into(), p.to_string()); }
            return Value::Info { text: text.to_string(), fields };
        }
        if let (Some(u),Some(t),Some(p)) = (used,total,percent) { return Value::Size{used:u,total:t,percent:p}; }
    }
    Value::Text(v.to_string())
}

pub fn value_text(v: &Value) -> String {
    match v {
        Value::Text(s) => s.clone(),
        Value::Size{used,total,percent} => format!("{used}/{total} GiB ({percent}%)"),
        Value::Info{text,..} => text.clone(),
        Value::None => "?".into(),
    }
}

pub fn value_field<'a>(v: &'a Value, key: &str) -> Option<String> {
    match v {
        Value::Text(s) => if key=="text"||key=="value" { Some(s.clone()) } else { None },
        Value::Size{used,total,percent} => match key { "used"=>Some(used.to_string()),"total"=>Some(total.to_string()),"percent"=>Some(percent.to_string()),"text"|"value"=>Some(format!("{used}/{total} GiB ({percent}%)")), _=>None },
        Value::Info{text,fields} => if key=="text"||key=="value" { Some(text.clone()) } else { fields.get(key).cloned() },
        Value::None => if key=="text"||key=="value" { Some("?".into()) } else { None },
    }
}

trait StrExt { fn if_empty(self, d:String)->String; }
impl StrExt for String { fn if_empty(self,d:String)->String { let t=self.trim(); if t.is_empty(){d} else {t.to_string()} } }
fn read(path: impl AsRef<Path>) -> String { fs::read_to_string(path).unwrap_or_default() }
fn cmd(prog: &str, args: &[&str]) -> String {
    Command::new(prog).args(args).output().ok().map(|o| {
        let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if s.is_empty() { String::from_utf8_lossy(&o.stderr).trim().to_string() } else { s }
    }).unwrap_or_default()
}
fn has_cmd(name:&str)->bool { Command::new("sh").args(["-c",&format!("command -v {name} >/dev/null 2>&1")]).status().map(|s|s.success()).unwrap_or(false) }

pub fn get(field: &str) -> Value {
    if field.starts_with("command:") { return Value::Text(command_field(field)); }
    let v = match field {
        "title" => { let u=std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or("?".into()); let h=hostname(); info_simple("title", format!("{u}@{h}")) },
        "break"|"separator" => info_simple("text", String::new()),
        "custom" => Value::Text(String::new()),
        "user" => info_simple("name", std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "?".into())),
        "host" => info_simple("name", hostname()),
        "os" => info_os(),
        "kernel" => info_simple("release", kernel()),
        "arch" => info_simple("arch", std::env::consts::ARCH.to_string()),
        "uptime" => info_simple("uptime", uptime()),
        "boot_time" => info_simple("time", boot_time()),
        "loadavg" => info_simple("load", loadavg()),
        "cpu" => info_cpu(),
        "cpu_cores" => info_simple("cores", std::thread::available_parallelism().map(|n| n.get().to_string()).unwrap_or("?".into())),
        "gpu" => info_gpu(),
        "ram" => ram(),
        "swap" => swap(),
        "disk" => disk_root(),
        "disks" => info_simple("disks", disks()),
        "network" => info_simple("network", network()),
        "local_ip" => info_simple("ip", local_ip()),
        "public_ip" => info_simple("ip", public_ip()),
        "battery" => info_simple("battery", battery()),
        "temperature" => info_simple("temp", temperature()),
        "resolution" => info_simple("resolution", resolution()),
        "shell" => info_shell(),
        "terminal" => info_terminal(),
        "de" => info_simple("de", de()),
        "wm" => info_simple("wm", wm()),
        "packages" => info_packages(),
        "processes" => info_simple("count", processes()),
        "init" => info_simple("name", init()),
        "locale" => info_simple("locale", locale()),
        "timezone" => info_simple("tz", timezone()),
        "date" => info_simple("date", date_now()),
        "motherboard" => info_simple("name", motherboard()),
        "bios" => info_simple("version", bios()),
        "display" => info_simple("resolution", resolution()),
        "audio" => info_simple("device", audio()),
        "bluetooth" => info_simple("state", bluetooth()),
        "power" => info_simple("state", power()),
        _ => Value::Text("?".into()),
    };
    match &v {
        Value::Text(s) if s.trim().is_empty() && field!="break" && field!="separator" && field!="custom" => Value::Text("?".into()),
        Value::Info{text,..} if text.trim().is_empty() && field!="break" && field!="separator" && field!="custom" => Value::Text("?".into()),
        _ => v,
    }
}

fn info_simple(key:&str, val:String) -> Value {
    let mut f = std::collections::BTreeMap::new();
    let t = if val.trim().is_empty() { "?".into() } else { val.trim().to_string() };
    f.insert(key.into(), t.clone());
    f.insert("text".into(), t.clone());
    Value::Info { text: t, fields: f }
}

fn hostname() -> String {
    for f in ["/etc/hostname"] { let t=read(f).trim().to_string(); if !t.is_empty() { return t; } }
    let o = cmd("hostname", &[]); if !o.is_empty() { return o.lines().next().unwrap_or("?").to_string(); }
    std::env::var("COMPUTERNAME").unwrap_or("?".into())
}

fn os_name() -> String { info_os_text() }
fn info_os_text() -> String {
    if cfg!(target_os="linux") {
        if let Ok(s)=fs::read_to_string("/etc/os-release") { for line in s.lines() { if let Some(v)=line.strip_prefix("PRETTY_NAME=") { return v.trim_matches('"').to_string(); } } }
    }
    if cfg!(target_os="macos") { let name=cmd("sw_vers",&["-productName"]); let ver=cmd("sw_vers",&["-productVersion"]); if !name.is_empty(){ return if ver.is_empty(){name} else {format!("{name} {ver}")}; } return "macOS".into(); }
    if cfg!(target_os="windows") { let name=cmd("powershell",&["-NoProfile","-Command","(Get-CimInstance Win32_OperatingSystem).Caption"]); if !name.is_empty(){ return name.lines().next().unwrap_or("Windows").to_string(); } return format!("Windows {}", std::env::consts::ARCH); }
    if cfg!(target_os="freebsd") { let v=cmd("freebsd-version",&[]); return if v.is_empty(){"FreeBSD".into()} else {format!("FreeBSD {v}")}; }
    std::env::consts::OS.to_string()
}
fn info_os() -> Value {
    let mut f=std::collections::BTreeMap::new();
    let text=info_os_text();
    f.insert("name".into(), text.clone()); f.insert("id".into(), detect_distro_id()); f.insert("arch".into(), std::env::consts::ARCH.to_string());
    if cfg!(target_os="linux") {
        if let Ok(s)=fs::read_to_string("/etc/os-release") {
            for line in s.lines() {
                if let Some(v)=line.strip_prefix("VERSION_ID=") { f.insert("version".into(), v.trim_matches('"').to_string()); }
                if let Some(v)=line.strip_prefix("ID_LIKE=") { f.insert("like".into(), v.trim_matches('"').to_string()); }
            }
        }
    }
    if cfg!(target_os="macos") { f.insert("version".into(), cmd("sw_vers",&["-productVersion"])); }
    Value::Info { text, fields:f }
}

pub fn detect_distro_id() -> String {
    if cfg!(target_os="linux") {
        if let Ok(s) = fs::read_to_string("/etc/os-release") {
            let mut id=String::new(); let mut like=String::new();
            for line in s.lines() {
                if let Some(v)=line.strip_prefix("ID=") { id=v.trim_matches('"').to_lowercase(); }
                if let Some(v)=line.strip_prefix("ID_LIKE=") { like=v.trim_matches('"').to_lowercase(); }
            }
            if !id.is_empty() { return id; }
            if let Some(first)=like.split_whitespace().next() { return first.to_string(); }
        }
    }
    std::env::consts::OS.to_string()
}

fn kernel() -> String {
    if cfg!(target_os="linux") { return read("/proc/sys/kernel/osrelease").trim().to_string().if_empty(cmd("uname", &["-r"])); }
    if cfg!(target_os="windows") { let v=cmd("powershell", &["-NoProfile","-Command","[System.Environment]::OSVersion.Version.ToString()"]); return if v.is_empty(){cmd("cmd",&["/c","ver"])} else {v}; }
    cmd("uname", &["-r"]).if_empty("?".into())
}

fn uptime() -> String {
    if cfg!(target_os="linux") {
        let s = read("/proc/uptime"); let secs: f64 = s.split_whitespace().next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        if secs>0.0 { return fmt_dur(secs as u64); }
    }
    if cfg!(target_os="macos") {
        let out = cmd("sysctl", &["-n","kern.boottime"]);
        if let Some(ts)=out.split("sec = ").nth(1).and_then(|s| s.split(',').next()).and_then(|n| n.trim().parse::<i64>().ok()) {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(ts);
            if now>ts { return fmt_dur((now-ts) as u64); }
        }
    }
    if cfg!(target_os="windows") {
        let out = cmd("powershell", &["-NoProfile","-Command","(Get-Date) - (gcim Win32_OperatingSystem).LastBootUpTime | ForEach-Object { [int]$_.TotalSeconds }"]);
        if let Ok(secs)=out.trim().parse::<u64>() { return fmt_dur(secs); }
    }
    let out = cmd("uptime", &[]); if !out.is_empty() { return out; }
    "?".into()
}

fn fmt_dur(secs: u64) -> String {
    let d = secs / 86400; let h = (secs % 86400) / 3600; let m = (secs % 3600) / 60;
    let mut out = vec![]; if d>0 {out.push(format!("{d}d"));} if h>0 {out.push(format!("{h}h"));} out.push(format!("{m}m")); out.join(" ")
}

fn boot_time() -> String {
    if cfg!(target_os="linux") {
        let s = read("/proc/stat");
        for line in s.lines() {
            if let Some(v) = line.strip_prefix("btime ") {
                if let Ok(ts) = v.trim().parse::<i64>() { return format_epoch(ts); }
            }
        }
    }
    if cfg!(target_os="macos") {
        let out = cmd("sysctl", &["-n","kern.boottime"]);
        if let Some(ts)=out.split("sec = ").nth(1).and_then(|s| s.split(',').next()).and_then(|n| n.trim().parse::<i64>().ok()) { return format_epoch(ts); }
    }
    if cfg!(target_os="windows") {
        let out = cmd("powershell", &["-NoProfile","-Command","(gcim Win32_OperatingSystem).LastBootUpTime.ToString('yyyy-MM-dd HH:mm')"]);
        if !out.is_empty() { return out.lines().next().unwrap_or("?").to_string(); }
    }
    "?".into()
}

fn format_epoch(ts:i64)->String {
    #[cfg(unix)]
    unsafe {
        unsafe extern "C" { fn localtime(t: *const i64) -> *mut tm; }
        #[repr(C)] struct tm { tm_sec:i32,tm_min:i32,tm_hour:i32,tm_mday:i32,tm_mon:i32,tm_year:i32,tm_wday:i32,tm_yday:i32,tm_isdst:i32,tm_gmtoff:i64,tm_zone:*const i8 }
        let p = localtime(&ts as *const i64);
        if !p.is_null() { let t=&*p; return format!("{:04}-{:02}-{:02} {:02}:{:02}", t.tm_year+1900, t.tm_mon+1, t.tm_mday, t.tm_hour, t.tm_min); }
    }
    ts.to_string()
}

fn loadavg() -> String {
    if cfg!(target_os="linux") { let s=read("/proc/loadavg"); let p:Vec<&str>=s.split_whitespace().collect(); if p.len()>=3 { return format!("{} {} {}", p[0],p[1],p[2]); } }
    if cfg!(unix) { let out=cmd("sysctl", &["-n","vm.loadavg"]); if !out.is_empty() { return out.replace(['{','}'], "").trim().to_string(); } }
    "?".into()
}

fn cpu() -> String { match info_cpu() { Value::Info{text,..}=>text, Value::Text(s)=>s, _=>"?".into() } }
fn info_cpu() -> Value {
    let mut f=std::collections::BTreeMap::new();
    let mut name = String::new();
    if cfg!(target_os="linux") {
        let s = read("/proc/cpuinfo");
        for line in s.lines() { if line.to_lowercase().starts_with("model name") { if let Some((_,v))=line.split_once(':') { name=v.trim().to_string(); break; } } }
        if let Some(line)=s.lines().find(|l| l.to_lowercase().starts_with("cpu(s)") || l.to_lowercase().starts_with("cpu cores")) { }
        let cores = std::thread::available_parallelism().map(|n| n.get().to_string()).unwrap_or("?".into());
        f.insert("cores".into(), cores);
    }
    if cfg!(target_os="macos") { name = cmd("sysctl", &["-n","machdep.cpu.brand_string"]); f.insert("cores".into(), cmd("sysctl", &["-n","hw.logicalcpu"])); }
    if cfg!(target_os="windows") { name = cmd("powershell",&["-NoProfile","-Command","(Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name)"]); f.insert("cores".into(), cmd("powershell",&["-NoProfile","-Command","(Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors"])); }
    if name.is_empty() { name = std::env::consts::ARCH.to_string(); }
    f.insert("name".into(), name.clone());
    Value::Info { text:name, fields:f }
}

fn gpu() -> String { match info_gpu() { Value::Info{text,..}=>text, Value::Text(s)=>s, _=>"?".into() } }
fn info_gpu() -> Value {
    let mut names: Vec<String> = vec![];
    if cfg!(target_os="linux") {
        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            for e in entries.flatten() {
                let name=e.file_name().to_string_lossy().to_string(); if !name.starts_with("card")||name.contains('-'){continue;}
                let dev=e.path().join("device"); let vendor=read(dev.join("vendor")).trim().to_string(); let device=read(dev.join("device")).trim().to_string();
                if vendor.is_empty()||device.is_empty(){continue;}
                let label=pci_lookup(&vendor,&device).unwrap_or_else(|| format!("{vendor}:{device}"));
                if !names.contains(&label){names.push(label);}
            }
        }
        if names.is_empty() { let out=cmd("sh",&["-c","lspci 2>/dev/null | grep -Ei 'vga|3d|display' | head -n1 | sed 's/.*: //' "]); if !out.is_empty(){ names.push(out.lines().next().unwrap_or("?").to_string()); } }
    }
    if cfg!(target_os="macos") { let o=cmd("sh",&["-c","system_profiler SPDisplaysDataType 2>/dev/null | awk -F': ' '/Chipset Model/{print $2; exit}'"]); if !o.is_empty(){ names.push(o); } }
    if cfg!(target_os="windows") { let o=cmd("powershell",&["-NoProfile","-Command","(Get-CimInstance Win32_VideoController | Select-Object -First 1 -ExpandProperty Name)"]); if !o.is_empty(){ names.push(o); } }
    let text = if names.is_empty(){"?".into()} else {names.join(", ")};
    let mut f=std::collections::BTreeMap::new(); f.insert("name".into(), text.clone()); f.insert("count".into(), names.len().to_string());
    Value::Info{ text, fields:f }
}
fn pci_lookup(vendor: &str, device: &str) -> Option<String> {
    let v = vendor.trim_start_matches("0x").to_lowercase();
    let d = device.trim_start_matches("0x").to_lowercase();
    let db = read("/usr/share/hwdata/pci.ids");
    if db.is_empty() { return None; }
    let mut in_vendor = false;
    for line in db.lines() {
        if line.starts_with('#') || line.trim().is_empty() { continue; }
        if !line.starts_with('\t') { in_vendor = line.starts_with(&v); continue; }
        if in_vendor && line.starts_with('\t') && !line.starts_with("\t\t") {
            let l = line.trim_start();
            if l.starts_with(&d) { let name = l[d.len()..].trim(); if !name.is_empty() { return Some(name.to_string()); } }
        }
    }
    None
}

fn meminfo_kb() -> std::collections::HashMap<String,u64> {
    let mut m=std::collections::HashMap::new();
    for line in read("/proc/meminfo").lines() {
        if let Some((k,v)) = line.split_once(':') { let n = v.split_whitespace().next().and_then(|x| x.parse::<u64>().ok()).unwrap_or(0); m.insert(k.trim().to_string(), n); }
    }
    m
}
fn size_kb(used_kb: u64, total_kb: u64) -> Value {
    let total = (total_kb as f64 / 1024.0 / 1024.0 * 10.0).round()/10.0;
    let used = (used_kb as f64 / 1024.0 / 1024.0 * 10.0).round()/10.0;
    let pct = if total_kb>0 { ((used_kb as f64/total_kb as f64)*100.0).round() as i64 } else {0};
    Value::Size{used,total,percent:pct}
}
fn ram() -> Value {
    if cfg!(target_os="linux") {
        let m=meminfo_kb(); let t=m.get("MemTotal").copied().unwrap_or(0); let a=m.get("MemAvailable").copied().unwrap_or_else(|| m.get("MemFree").copied().unwrap_or(0));
        if t>0 { return size_kb(t.saturating_sub(a),t); }
    }
    if cfg!(target_os="macos") {
        let total = cmd("sysctl", &["-n","hw.memsize"]).trim().parse::<u64>().unwrap_or(0) / 1024;
        let vm = cmd("vm_stat", &[]); let mut page=4096u64; let mut free=0u64; let mut inactive=0u64; let mut spec=0u64;
        for line in vm.lines() {
            if line.starts_with("page size of") { page = line.split_whitespace().nth(7).and_then(|n| n.parse().ok()).unwrap_or(4096); }
            if let Some((k,v))=line.split_once(':') { let n=v.trim().trim_end_matches('.').replace('.',""); let n=n.parse::<u64>().unwrap_or(0); match k.trim() {"Pages free"=>free=n,"Pages inactive"=>inactive=n,"Pages speculative"=>spec=n,_=>{}} }
        }
        let avail = (free+inactive+spec).saturating_mul(page)/1024;
        if total>0 { return size_kb(total.saturating_sub(avail), total); }
    }
    if cfg!(target_os="windows") {
        let out = cmd("powershell", &["-NoProfile","-Command","$os=gcim Win32_OperatingSystem; \"$($os.TotalVisibleMemorySize) $($os.FreePhysicalMemory)\""]);
        let p:Vec<u64>=out.split_whitespace().filter_map(|n| n.parse().ok()).collect();
        if p.len()==2 { return size_kb(p[0].saturating_sub(p[1]), p[0]); }
    }
    Value::Text("?".into())
}
fn swap() -> Value {
    if cfg!(target_os="linux") { let m=meminfo_kb(); let t=m.get("SwapTotal").copied().unwrap_or(0); let f=m.get("SwapFree").copied().unwrap_or(0); if t==0 {return Value::Text("none".into())} else {return size_kb(t.saturating_sub(f),t)} }
    if cfg!(target_os="macos") { let out=cmd("sysctl",&["-n","vm.swapusage"]); if !out.is_empty() { return Value::Text(out); } }
    if cfg!(target_os="windows") { let out=cmd("powershell",&["-NoProfile","-Command","$os=gcim Win32_OperatingSystem; \"$($os.TotalVirtualMemorySize-$os.FreeVirtualMemory) $($os.TotalVirtualMemorySize)\""]); let p:Vec<u64>=out.split_whitespace().filter_map(|n|n.parse().ok()).collect(); if p.len()==2 { return size_kb(p[0],p[1]); } }
    Value::Text("?".into())
}

fn disk_root() -> Value { disk_path(if cfg!(windows) {"C:\\"} else {"/"}) }
fn disk_path(path:&str) -> Value {
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::mem::MaybeUninit;
        unsafe extern "C" { fn statvfs(path:*const i8, buf:*mut Statvfs)->i32; }
        #[repr(C)] struct Statvfs { f_bsize:u64,f_frsize:u64,f_blocks:u64,f_bfree:u64,f_bavail:u64,f_files:u64,f_ffree:u64,f_favail:u64,f_fsid:u64,f_flag:u64,f_namemax:u64,__f_spare:[i32;6] }
        if let Ok(c)=CString::new(path) { unsafe { let mut buf=MaybeUninit::<Statvfs>::uninit(); if statvfs(c.as_ptr(), buf.as_mut_ptr())==0 { let b=buf.assume_init(); let total=b.f_blocks.saturating_mul(b.f_frsize)/1024; let avail=b.f_bavail.saturating_mul(b.f_frsize)/1024; return size_kb(total.saturating_sub(avail), total); } } }
    }
    if cfg!(target_os="windows") {
        let drive = path.chars().next().unwrap_or('C');
        let out = cmd("powershell", &["-NoProfile","-Command",&format!("$d=Get-PSDrive -Name {drive}; \"$($d.Used/1KB) $(($d.Used+$d.Free)/1KB)\"")]);
        let p:Vec<u64>=out.split_whitespace().filter_map(|n| n.replace(',', ".").parse::<f64>().ok().map(|x| x as u64)).collect();
        if p.len()==2 { return size_kb(p[0],p[1]); }
    }
    let out = cmd("df", &["-Pk", path]);
    if let Some(line)=out.lines().nth(1) { let p:Vec<&str>=line.split_whitespace().collect(); if p.len()>=4 { let t=p[1].parse().unwrap_or(0); let u=p[2].parse().unwrap_or(0); return size_kb(u,t); } }
    Value::Text("?".into())
}

fn disks() -> String {
    if cfg!(target_os="linux") {
        let m = read("/proc/mounts"); let mut out=vec![];
        for line in m.lines() {
            let p: Vec<&str>=line.split_whitespace().collect(); if p.len()<3 { continue; }
            let fs=p[2]; let mp=p[1];
            if matches!(fs,"tmpfs"|"devtmpfs"|"squashfs"|"overlay"|"proc"|"sysfs"|"cgroup2"|"efivarfs"|"debugfs"|"tracefs") { continue; }
            if !p[0].starts_with('/') { continue; }
            if let Value::Size{used,total,percent}=disk_path(mp) { out.push(format!("{mp}: {used}/{total} GiB ({percent}%)")); }
            if out.len()>=5 { break; }
        }
        return if out.is_empty(){"?".into()} else {out.join(", ")};
    }
    if cfg!(target_os="macos") {
        let out=cmd("df", &["-H"]); let mut lines=vec![];
        for line in out.lines().skip(1) { let p:Vec<&str>=line.split_whitespace().collect(); if p.len()>=6 { let mp=p.last().unwrap_or(&""); if mp.starts_with("/Volumes") || *mp=="/" { lines.push(format!("{mp}: {} / {}", p.get(2).unwrap_or(&"?"), p.get(1).unwrap_or(&"?"))); } } if lines.len()>=5 {break;} }
        return if lines.is_empty(){"?".into()} else {lines.join(", ")};
    }
    if cfg!(target_os="windows") {
        let out=cmd("powershell",&["-NoProfile","-Command","Get-PSDrive -PSProvider FileSystem | ForEach-Object { '{0}: {1:N1}/{2:N1} GiB' -f $_.Name, ($_.Used/1GB), (($_.Used+$_.Free)/1GB) }"]);
        let lines:Vec<String>=out.lines().map(|s|s.trim().to_string()).filter(|s|!s.is_empty()).take(5).collect();
        return if lines.is_empty(){"?".into()} else {lines.join(", ")};
    }
    "?".into()
}

fn network() -> String { let ip=local_ip(); let iface=default_iface().unwrap_or_default(); if !iface.is_empty() && ip!="?" { format!("{iface} ({ip})") } else if ip!="?" { ip } else { "?".into() } }
fn default_iface() -> Option<String> {
    if cfg!(target_os="linux") {
        let s = read("/proc/net/route");
        for line in s.lines().skip(1) { let p: Vec<&str>=line.split_whitespace().collect(); if p.len()>=3 && p[1]=="00000000" && p[2]!="00000000" { return Some(p[0].to_string()); } }
    }
    if cfg!(target_os="macos") { let o=cmd("sh",&["-c","route -n get default 2>/dev/null | awk '/interface:/{print $2; exit}'"]); if !o.is_empty() { return Some(o); } }
    None
}
fn local_ip() -> String {
    if cfg!(target_os="linux") {
        let out = cmd("ip", &["-4","route","get","1.1.1.1"]);
        for tok in out.split_whitespace().collect::<Vec<_>>().windows(2) { if tok[0]=="src" { return tok[1].to_string(); } }
    }
    if cfg!(target_os="macos") { let o=cmd("ipconfig",&["getifaddr","en0"]); if !o.is_empty() { return o; } }
    if cfg!(target_os="windows") { let o=cmd("powershell",&["-NoProfile","-Command","(Get-NetIPAddress -AddressFamily IPv4 | Where-Object {$_.IPAddress -notlike '169.*' -and $_.IPAddress -ne '127.0.0.1'} | Select-Object -First 1 -ExpandProperty IPAddress)"]); if !o.is_empty() { return o; } }
    "?".into()
}
fn public_ip() -> String {
    for (prog,args) in [("curl", vec!["-fsS","--max-time","2","https://ifconfig.me"]), ("wget", vec!["-qO-","--timeout=2","https://ifconfig.me"])] {
        let out = cmd(prog, &args.iter().map(|s|*s).collect::<Vec<_>>());
        let t = out.trim().to_string();
        if !t.is_empty() && t.len()<100 { return t; }
    }
    "?".into()
}

fn battery() -> String {
    if cfg!(target_os="linux") {
        let base = Path::new("/sys/class/power_supply");
        if let Ok(entries) = fs::read_dir(base) {
            for e in entries.flatten() {
                let n=e.file_name().to_string_lossy().to_string(); if !n.starts_with("BAT") { continue; }
                let cap=read(e.path().join("capacity")).trim().to_string(); let status=read(e.path().join("status")).trim().to_string();
                if !cap.is_empty() { return if status.is_empty(){format!("{cap}%")} else {format!("{cap}% ({status})")}; }
            }
        }
    }
    if cfg!(target_os="macos") { let o=cmd("pmset",&["-g","batt"]); if o.contains('%') { return o.lines().find(|l| l.contains('%')).unwrap_or("?").trim().to_string(); } }
    if cfg!(target_os="windows") { let o=cmd("powershell",&["-NoProfile","-Command","$b=Get-CimInstance Win32_Battery; if($b){ '{0}% ({1})' -f $b.EstimatedChargeRemaining, $b.BatteryStatus }"]); if !o.is_empty() { return o; } }
    "?".into()
}
fn temperature() -> String {
    if cfg!(target_os="linux") {
        if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
            for e in entries.flatten() { let t=read(e.path().join("temp")).trim().to_string(); if let Ok(v)=t.parse::<f64>() { if v>1000.0 { return format!("{:.1}°C", v/1000.0);} else if v>0.0 && v<200.0 { return format!("{v:.1}°C"); } } }
        }
        let out = cmd("sh", &["-c","sensors 2>/dev/null | awk '/Package id 0|Tctl|temp1/ {gsub(/[+°C]/,\"\",$0); for(i=1;i<=NF;i++) if($i ~ /[0-9]+\\.[0-9]/){print $i; exit}}'"]);
        if !out.is_empty() { return format!("{}°C", out.lines().next().unwrap_or("?")); }
    }
    if cfg!(target_os="macos") { let o=cmd("sh",&["-c","istats cpu temp 2>/dev/null | awk '{print $3; exit}'"]); if !o.is_empty() { return o; } }
    "?".into()
}
fn resolution() -> String {
    if cfg!(target_os="linux") {
        if let Ok(entries)=fs::read_dir("/sys/class/drm") { for e in entries.flatten() { let n=e.file_name().to_string_lossy().to_string(); if !n.contains('-') { continue; } let modes=read(e.path().join("modes")); if let Some(first)=modes.lines().next() { let t=first.trim(); if !t.is_empty() { return t.to_string(); } } } }
        let out=cmd("sh",&["-c","xrandr --current 2>/dev/null | awk '/\\*/{print $1; exit}'"]); if !out.is_empty() { return out; }
    }
    if cfg!(target_os="macos") { let o=cmd("sh",&["-c","system_profiler SPDisplaysDataType 2>/dev/null | awk '/Resolution/{print $2\"x\"$4; exit}'"]); if !o.is_empty() { return o; } }
    if cfg!(target_os="windows") { let o=cmd("powershell",&["-NoProfile","-Command","$v=Get-CimInstance Win32_VideoController | Select-Object -First 1; \"$($v.CurrentHorizontalResolution)x$($v.CurrentVerticalResolution)\""]); if !o.is_empty() && o!="x" { return o; } }
    "?".into()
}
fn shell() -> String { match info_shell() { Value::Info{text,..}=>text, Value::Text(s)=>s, _=>"?".into() } }
fn info_shell() -> Value {
    let path = std::env::var("SHELL").or_else(|_| std::env::var("COMSPEC")).unwrap_or("?".into());
    let name = Path::new(&path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.clone());
    let mut f=std::collections::BTreeMap::new(); f.insert("name".into(), name.clone()); f.insert("path".into(), path);
    Value::Info{ text:name, fields:f }
}
fn terminal() -> String { match info_terminal() { Value::Info{text,..}=>text, Value::Text(s)=>s, _=>"?".into() } }
fn info_terminal() -> Value {
    let name = std::env::var("TERM_PROGRAM").or_else(|_| std::env::var("TERMINAL_EMULATOR")).or_else(|_| std::env::var("WT_SESSION")).or_else(|_| std::env::var("TERM")).unwrap_or("?".into());
    let mut f=std::collections::BTreeMap::new(); f.insert("name".into(), name.clone());
    if let Ok(v)=std::env::var("TERM_PROGRAM_VERSION") { f.insert("version".into(), v); }
    Value::Info{ text:name, fields:f }
}
fn de() -> String { if cfg!(target_os="macos") { return "Aqua".into(); } if cfg!(target_os="windows") { return "Windows Shell".into(); } std::env::var("XDG_CURRENT_DESKTOP").or_else(|_| std::env::var("DESKTOP_SESSION")).unwrap_or("?".into()) }
fn wm() -> String {
    if cfg!(target_os="macos") { return "Quartz Compositor".into(); }
    if cfg!(target_os="windows") { return "DWM".into(); }
    if std::env::var("WAYLAND_DISPLAY").ok().filter(|s| !s.is_empty()).is_some() { return std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "Wayland".into()); }
    std::env::var("XDG_SESSION_DESKTOP").or_else(|_| std::env::var("XDG_CURRENT_DESKTOP")).unwrap_or_else(|_| std::env::var("XDG_SESSION_TYPE").unwrap_or("?".into()))
}
fn packages() -> String { match info_packages() { Value::Info{text,..}=>text, Value::Text(s)=>s, _=>"?".into() } }
fn info_packages() -> Value {
    let mut managers: Vec<(String,i64)> = vec![];
    if cfg!(target_os="linux") {
        if let Ok(rd)=fs::read_dir("/var/lib/pacman/local") { let n=rd.flatten().filter(|e| e.file_type().map(|t|t.is_dir()).unwrap_or(false)).count() as i64; if n>1 { managers.push(("pacman".into(), n-1)); } }
        if let Ok(s)=fs::read_to_string("/var/lib/dpkg/status") { let n=s.lines().filter(|l| l.starts_with("Package: ")).count() as i64; if n>0 { managers.push(("dpkg".into(), n)); } }
        for (name,cmdline) in [("rpm","rpm -qa 2>/dev/null | wc -l"),("xbps","xbps-query -l 2>/dev/null | wc -l"),("apk","apk info 2>/dev/null | wc -l"),("flatpak","flatpak list 2>/dev/null | wc -l"),("snap","snap list 2>/dev/null | tail -n +2 | wc -l")] {
            let out=cmd("sh",&["-c",cmdline]); if let Ok(n)=out.trim().parse::<i64>() { if n>0 { managers.push((name.into(), n)); } }
        }
    }
    if cfg!(target_os="macos") { let b=cmd("sh",&["-c","brew list --formula 2>/dev/null | wc -l"]); if let Ok(n)=b.trim().parse::<i64>() { if n>0 { managers.push(("brew".into(), n)); } } }
    if cfg!(target_os="windows") {
        let c=cmd("powershell",&["-NoProfile","-Command","(choco list --local-only -r 2>$null | Measure-Object).Count"]); if let Ok(n)=c.trim().parse::<i64>() { if n>0 { managers.push(("choco".into(), n)); } }
        let s=cmd("powershell",&["-NoProfile","-Command","(scoop list 2>$null | Measure-Object).Count"]); if let Ok(n)=s.trim().parse::<i64>() { if n>2 { managers.push(("scoop".into(), n-2)); } }
        let w=cmd("powershell",&["-NoProfile","-Command","(winget list 2>$null | Measure-Object).Count"]); if let Ok(n)=w.trim().parse::<i64>() { if n>0 { managers.push(("winget".into(), n)); } }
    }
    let text = if managers.is_empty() { "?".into() } else { managers.iter().map(|(n,c)| format!("{c} ({n})")).collect::<Vec<_>>().join(", ") };
    let mut f=std::collections::BTreeMap::new(); f.insert("text".into(), text.clone()); f.insert("count".into(), managers.iter().map(|(_,c)|*c).sum::<i64>().to_string());
    f.insert("managers".into(), managers.iter().map(|(n,_)|n.clone()).collect::<Vec<_>>().join(","));
    Value::Info{ text, fields:f }
}
fn processes() -> String {
    if cfg!(target_os="linux") { if let Ok(rd)=fs::read_dir("/proc") { let n=rd.flatten().filter(|e| e.file_name().to_string_lossy().chars().all(|c| c.is_ascii_digit())).count(); return n.to_string(); } }
    if cfg!(target_os="macos") { let o=cmd("sh",&["-c","ps -A | wc -l"]); if let Ok(n)=o.trim().parse::<i64>() { return (n-1).max(0).to_string(); } }
    if cfg!(target_os="windows") { let o=cmd("powershell",&["-NoProfile","-Command","(Get-Process | Measure-Object).Count"]); if !o.is_empty() { return o; } }
    "?".into()
}
fn init() -> String {
    if cfg!(target_os="linux") { let c=read("/proc/1/comm").trim().to_string(); if !c.is_empty() { return c; } let cmdline=read("/proc/1/cmdline").replace('\0'," "); let first=cmdline.split_whitespace().next().unwrap_or("?"); return Path::new(first).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or("?".into()); }
    if cfg!(target_os="macos") { return "launchd".into(); }
    if cfg!(target_os="windows") { return "services.exe".into(); }
    "?".into()
}
fn locale() -> String { std::env::var("LANG").or_else(|_| std::env::var("LC_ALL")).or_else(|_| std::env::var("LC_CTYPE")).unwrap_or_else(|_| if cfg!(target_os="windows"){ cmd("powershell",&["-NoProfile","-Command","(Get-Culture).Name"]) } else {"?".into()}) }
fn timezone() -> String {
    if let Ok(tz)=std::env::var("TZ") { if !tz.is_empty() { return tz; } }
    if cfg!(unix) { if let Ok(p)=fs::read_link("/etc/localtime") { let s=p.to_string_lossy().to_string(); if let Some(i)=s.find("zoneinfo/") { return s[i+9..].to_string(); } return s; } let t=read("/etc/timezone").trim().to_string(); if !t.is_empty() { return t; } }
    if cfg!(target_os="windows") { let o=cmd("powershell",&["-NoProfile","-Command","(Get-TimeZone).Id"]); if !o.is_empty() { return o; } }
    "?".into()
}
fn date_now() -> String { let out=cmd("date", &["+%Y-%m-%d %H:%M:%S"]); if !out.is_empty() { return out; } if cfg!(target_os="windows") { let o=cmd("powershell",&["-NoProfile","-Command","Get-Date -Format 'yyyy-MM-dd HH:mm:ss'"]); if !o.is_empty() { return o; } } SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or("?".into()) }
fn motherboard() -> String {
    if cfg!(target_os="linux") { let v=read("/sys/devices/virtual/dmi/id/board_vendor").trim().to_string(); let n=read("/sys/devices/virtual/dmi/id/board_name").trim().to_string(); return format!("{v} {n}").trim().to_string().if_empty("?".into()); }
    if cfg!(target_os="windows") { let o=cmd("powershell",&["-NoProfile","-Command","$b=Get-CimInstance Win32_BaseBoard; \"$($b.Manufacturer) $($b.Product)\""]); if !o.trim().is_empty() { return o; } }
    "?".into()
}
fn bios() -> String {
    if cfg!(target_os="linux") { let v=read("/sys/devices/virtual/dmi/id/bios_vendor").trim().to_string(); let n=read("/sys/devices/virtual/dmi/id/bios_version").trim().to_string(); return format!("{v} {n}").trim().to_string().if_empty("?".into()); }
    if cfg!(target_os="windows") { let o=cmd("powershell",&["-NoProfile","-Command","$b=Get-CimInstance Win32_BIOS; \"$($b.Manufacturer) $($b.SMBIOSBIOSVersion)\""]); if !o.trim().is_empty() { return o; } }
    "?".into()
}
fn audio() -> String {
    if cfg!(target_os="linux") { let out=cmd("pactl",&["info"]); for line in out.lines() { if let Some(v)=line.strip_prefix("Default Sink:") { let t=v.trim(); if !t.is_empty() { return t.to_string(); } } } }
    if cfg!(target_os="macos") { let o=cmd("sh",&["-c","system_profiler SPAudioDataType 2>/dev/null | awk -F': ' '/Default Output Device: Yes/{found=1} found&&/Name:/{print $2; exit}'"]); if !o.is_empty() { return o; } }
    if cfg!(target_os="windows") { let o=cmd("powershell",&["-NoProfile","-Command","(Get-CimInstance Win32_SoundDevice | Select-Object -First 1 -ExpandProperty Name)"]); if !o.is_empty() { return o; } }
    "?".into()
}
fn bluetooth() -> String {
    if cfg!(target_os="linux") { let out=cmd("bluetoothctl",&["show"]); for line in out.lines() { let l=line.trim(); if let Some(v)=l.strip_prefix("Powered:") { return v.trim().to_string(); } } if Path::new("/sys/class/bluetooth").exists() { return "off".into(); } }
    if cfg!(target_os="macos") { let o=cmd("sh",&["-c","system_profiler SPBluetoothDataType 2>/dev/null | awk -F': ' '/State:/{print $2; exit}'"]); if !o.is_empty() { return o; } }
    if cfg!(target_os="windows") { let o=cmd("powershell",&["-NoProfile","-Command","(Get-PnpDevice -Class Bluetooth -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty Status)"]); if !o.is_empty() { return o; } }
    "?".into()
}
fn power() -> String {
    if cfg!(target_os="linux") { let g=read("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor").trim().to_string(); if !g.is_empty(){ return g; } }
    if cfg!(target_os="macos") { let o=cmd("pmset",&["-g","therm"]); if !o.is_empty() { return o.lines().next().unwrap_or("?").to_string(); } }
    battery()
}
fn command_field(field: &str) -> String {
    let parts: Vec<&str> = field.splitn(3, ':').collect(); if parts.len()!=3 { return "?".into(); }
    let cmdline = parts[2];
    let out = if cfg!(target_os="windows") { Command::new("cmd").args(["/C", cmdline]).output().ok() } else { Command::new("sh").args(["-c", cmdline]).output().ok() };
    if let Some(o)=out {
        let s=String::from_utf8_lossy(&o.stdout).trim().to_string(); if !s.is_empty() { return s.lines().next().unwrap_or("?").to_string(); }
        let e=String::from_utf8_lossy(&o.stderr).trim().to_string(); if !e.is_empty() { return e.lines().next().unwrap_or("?").to_string(); }
    }
    "?".into()
}
