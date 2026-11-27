use crate::modules::Value;

pub fn color_enabled() -> bool {
    if std::env::var_os("NO_COLOR").is_some() { return false; }
    match std::env::var("TERM").ok().as_deref() {
        None | Some("") | Some("dumb") => false,
        _ => true,
    }
}

fn ansi_rgb(r:u8,g:u8,b:u8) -> String { format!("\x1b[38;2;{r};{g};{b}m") }
fn ansi_bg(r:u8,g:u8,b:u8) -> String { format!("\x1b[48;2;{r};{g};{b}m") }
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

pub fn theme_accent(theme: &str) -> Option<(u8,u8,u8)> {
    match theme {
        "default" => None,
        "mono" => Some((200,200,200)),
        "light" => Some((23,78,166)),
        "neon" => Some((255,80,220)),
        "dark" => Some((120,170,255)),
        "nord" => Some((136,192,208)),
        "gruvbox" => Some((250,189,47)),
        "dracula" => Some((189,147,249)),
        "tokyo" => Some((122,162,247)),
        "solarized" => Some((38,139,210)),
        "matrix" => Some((0,255,65)),
        _ => None,
    }
}

pub fn parse_hex_color(s: &str) -> Option<(u8,u8,u8)> {
    let s = s.trim();
    let hex = s.strip_prefix('#')?;
    if hex.len()==6 {
        let r=u8::from_str_radix(&hex[0..2],16).ok()?;
        let g=u8::from_str_radix(&hex[2..4],16).ok()?;
        let b=u8::from_str_radix(&hex[4..6],16).ok()?;
        return Some((r,g,b));
    }
    if hex.len()==3 {
        let r=u8::from_str_radix(&hex[0..1].repeat(2),16).ok()?;
        let g=u8::from_str_radix(&hex[1..2].repeat(2),16).ok()?;
        let b=u8::from_str_radix(&hex[2..3].repeat(2),16).ok()?;
        return Some((r,g,b));
    }
    None
}

pub fn named_color(s: &str) -> Option<(u8,u8,u8)> {
    match s.to_lowercase().as_str() {
        "blue"=>Some((80,140,255)),"cyan"=>Some((0,220,220)),"green"=>Some((80,200,120)),"yellow"=>Some((240,200,80)),
        "orange"=>Some((255,140,0)),"red"=>Some((255,80,80)),"magenta"|"pink"=>Some((255,100,200)),"white"=>Some((230,230,230)),
        "arch"=>Some((23,147,209)),"ubuntu"=>Some((233,84,32)),"debian"=>Some((215,10,83)),"fedora"=>Some((60,110,180)),
        "nixos"=>Some((126,186,228)),"mint"=>Some((135,207,62)),"gentoo"=>Some((170,170,255)),"void"=>Some((71,128,97)),
        "tux"=>Some((255,200,0)),
        _=>None
    }
}

pub fn resolve_accent(accent: Option<&str>, theme: &str, logo: &str) -> Option<(u8,u8,u8)> {
    if let Some(a)=accent {
        return parse_hex_color(a).or_else(|| named_color(a));
    }
    if let Some(t)=theme_accent(theme) { return Some(t); }
    let dc = crate::logos::distro_colors();
    if let Some(c)=dc.get(logo) { return named_color(c); }
    None
}

pub fn bar(used: f64, total: f64, width: usize, style: &str, ascii: bool) -> String {
    let pct = if total>0.0 { (used/total).clamp(0.0,1.0) } else { 0.0 };
    let filled = ((pct * width as f64).round() as usize).min(width);
    let empty = width - filled;
    let (f,e) = match (style, ascii) {
        ("ascii",_) | (_,true) => ("#", "-"),
        ("arrows",_) => (">", "."),
        ("dots",_) => ("●", "○"),
        ("lines",_) => ("━", "─"),
        _ => ("█", "░"),
    };
    format!("{}{}", f.repeat(filled), e.repeat(empty))
}

pub fn format_value(v: &Value, opts_bar_width: usize, bar_style: &str, ascii: bool) -> String {
    match v {
        Value::Size{used,total,percent} => {
            let b = bar(*used,*total,opts_bar_width,bar_style,ascii);
            format!("{used}/{total} GiB   {b}  {percent}%")
        }
        Value::Text(s) => s.clone(),
        Value::Info{text, ..} => text.clone(),
        Value::None => "?".into(),
    }
}

pub fn render_info_lines_spec(specs: &[crate::ModuleSpec], data: &std::collections::BTreeMap<String,Value>, opts: &crate::Opts, accent: Option<(u8,u8,u8)>) -> Vec<String> {
    let mut max_label = 0usize;
    for s in specs { if s.is_builtin() { max_label = max_label.max(s.display_key().len()); } }
    let mut out = vec![];
    let ac = accent.map(|(r,g,b)| ansi_rgb(r,g,b));
    for s in specs {
        if s.kind == "break" { out.push(String::new()); continue; }
        if s.kind == "separator" { out.push(if opts.ascii { "-".repeat(max_label.max(8)) } else { "─".repeat(max_label.max(8)) }); continue; }
        if s.kind == "title" || s.kind == "custom" {
            let v = data.get(&s.kind).cloned().unwrap_or(Value::Text(String::new()));
            let txt = if let Some(t)=&s.format { crate::format_with_template(t, &s.kind, &v) } else { crate::modules::value_text(&v) };
            out.push(txt);
            continue;
        }
        let v = data.get(&s.kind).cloned().unwrap_or(Value::Text("?".into()));
        let val = if let Some(t)=&s.format { crate::format_with_template(t, &s.kind, &v) } else { format_value(&v, opts.bar_width, &opts.bar_style, opts.ascii) };
        let lbl = s.display_key();
        let key_ac = s.key_color.as_deref().and_then(|c| parse_hex_color(c).or_else(|| named_color(c))).map(|(r,g,b)| ansi_rgb(r,g,b)).or_else(|| ac.clone());
        let line = if opts.color {
            let ac = key_ac.as_deref().unwrap_or(BOLD);
            if opts.compact { format!("{ac}{BOLD}{lbl}{RESET}{}{val}", opts.separator) }
            else { format!("{ac}{BOLD}{:>w$}{RESET}{}{val}", lbl, opts.separator, w=max_label) }
        } else {
            if opts.compact { format!("{lbl}{}{val}", opts.separator) }
            else { format!("{:>w$}{}{val}", lbl, opts.separator, w=max_label) }
        };
        out.push(line);
    }
    out
}

pub fn visible_len(s: &str) -> usize {
    let mut n=0; let mut chars=s.chars().peekable();
    while let Some(c)=chars.next() {
        if c=='\x1b' {
            if chars.peek()==Some(&'[') { chars.next(); while let Some(&x)=chars.peek() { chars.next(); if x.is_ascii_alphabetic() { break; } } }
        } else { n+=1; }
    }
    n
}

pub fn pad_visible(s: &str, width: usize) -> String {
    let v = visible_len(s);
    if v >= width { s.to_string() } else { format!("{s}{}", " ".repeat(width-v)) }
}
