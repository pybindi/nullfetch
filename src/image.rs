use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use image::imageops::FilterType;
use image::{DynamicImage, RgbaImage};
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct RenderedImage {
    pub lines: Vec<String>,
    pub width: usize,
}

fn term_size() -> (usize, usize) {
    if let Ok(out) = Command::new("stty").args(["size"]).output() {
        let s = String::from_utf8_lossy(&out.stdout);
        let mut it = s.split_whitespace();
        let rows = it.next().and_then(|x| x.parse().ok()).unwrap_or(24usize);
        let cols = it.next().and_then(|x| x.parse().ok()).unwrap_or(80usize);
        return (cols, rows);
    }
    (80,24)
}

fn load_image(path: &str) -> Result<DynamicImage, String> {
    let p = if path == "-" { return Err("stdin image not supported yet".into()) } else { Path::new(path) };
    let bytes = fs::read(p).map_err(|e| format!("cannot read image {}: {e}", p.display()))?;
    image::load_from_memory(&bytes).map_err(|e| format!("cannot decode image {}: {e}", p.display()))
}

fn rgba_rows(img: &RgbaImage) -> Vec<Vec<[u8;4]>> {
    let (w,h)=img.dimensions();
    let mut rows = Vec::with_capacity(h as usize);
    for y in 0..h {
        let mut row = Vec::with_capacity(w as usize);
        for x in 0..w {
            let p = img.get_pixel(x,y).0;
            row.push(p);
        }
        rows.push(row);
    }
    rows
}

fn blend(p: [u8;4], bg: [u8;3]) -> [u8;3] {
    let a = p[3] as f32 / 255.0;
    let inv = 1.0 - a;
    [
        (p[0] as f32*a + bg[0] as f32*inv) as u8,
        (p[1] as f32*a + bg[1] as f32*inv) as u8,
        (p[2] as f32*a + bg[2] as f32*inv) as u8,
    ]
}

fn ansi_fg(rgb:[u8;3]) -> String { format!("\x1b[38;2;{};{};{}m", rgb[0],rgb[1],rgb[2]) }
fn ansi_bg(rgb:[u8;3]) -> String { format!("\x1b[48;2;{};{};{}m", rgb[0],rgb[1],rgb[2]) }

pub fn render_blocks(path:&str, cols:usize, lines:usize, bg: [u8;3]) -> Result<RenderedImage, String> {
    let img = load_image(path)?;
    let target_w = (cols.max(1)) as u32;
    let target_h = (lines.max(1) * 2) as u32;
    let resized = img.resize_exact(target_w, target_h, FilterType::Triangle).to_rgba8();
    let rows = rgba_rows(&resized);
    let mut out = Vec::new();
    let mut y = 0usize;
    while y + 1 < rows.len() {
        let mut line = String::new();
        for x in 0..rows[y].len() {
            let top = blend(rows[y][x], bg);
            let bot = blend(rows[y+1][x], bg);
            line.push_str(&ansi_fg(top));
            line.push_str(&ansi_bg(bot));
            line.push('▀');
        }
        line.push_str("\x1b[0m");
        out.push(line);
        y += 2;
    }
    if rows.len() % 2 == 1 {
        let mut line = String::new();
        let last = rows.len()-1;
        for x in 0..rows[last].len() {
            let top = blend(rows[last][x], bg);
            line.push_str(&ansi_fg(top));
            line.push_str(&ansi_bg(bg));
            line.push('▀');
        }
        line.push_str("\x1b[0m");
        out.push(line);
    }
    Ok(RenderedImage { width: cols, lines: out })
}

pub fn render_ascii(path:&str, cols:usize, lines:usize, bg:[u8;3]) -> Result<RenderedImage,String> {
    let img = load_image(path)?;
    let resized = img.resize_exact(cols.max(1) as u32, lines.max(1) as u32, FilterType::Triangle).to_rgba8();
    let rows = rgba_rows(&resized);
    let chars = [' ','.',':','-','=','+','*','#','%','@'];
    let mut out=Vec::new();
    for row in rows {
        let mut line = String::new();
        for p in row {
            let rgb = blend(p,bg);
            let lum = (0.2126*rgb[0] as f32 + 0.7152*rgb[1] as f32 + 0.0722*rgb[2] as f32) as usize;
            let idx = (lum * (chars.len()-1) / 255).min(chars.len()-1);
            line.push(chars[idx]);
        }
        out.push(line);
    }
    Ok(RenderedImage{width:cols, lines:out})
}

fn encode_png_bytes(path:&str, cols:usize, lines:usize) -> Result<Vec<u8>,String> {
    let img = load_image(path)?;
    let resized = img.resize_exact(cols.max(1) as u32, (lines.max(1)*2) as u32, FilterType::Triangle);
    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    resized.write_to(&mut cursor, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    Ok(buf)
}

pub fn render_kitty(path:&str, cols:usize, lines:usize) -> Result<RenderedImage,String> {
    let bytes = encode_png_bytes(path, cols, lines)?;
    let b64 = B64.encode(bytes);
    let mut out = Vec::new();
    let chunks: Vec<&str> = b64.as_bytes().chunks(4096).map(|c| std::str::from_utf8(c).unwrap_or("")).collect();
    for (i,chunk) in chunks.iter().enumerate() {
        let m = if i == chunks.len()-1 { 0 } else { 1 };
        if i == 0 {
            out.push(format!("\x1b_Ga=T,f=100,c={cols},r={lines},m={m};{chunk}\x1b\\"));
        } else {
            out.push(format!("\x1b_Gm={m};{chunk}\x1b\\"));
        }
    }
    // kitty places cursor; add blank lines for layout
    let mut lines_out = vec![out.join("")];
    for _ in 1..lines { lines_out.push(String::new()); }
    Ok(RenderedImage{width:cols, lines:lines_out})
}

pub fn render_iterm(path:&str, cols:usize, lines:usize) -> Result<RenderedImage,String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let b64 = B64.encode(&bytes);
    let name = B64.encode(Path::new(path).file_name().unwrap_or_default().to_string_lossy().as_bytes());
    let line = format!("\x1b]1337;File=name={name};size={};inline=1;width={cols};height={lines};preserveAspectRatio=1:{b64}\x07", bytes.len());
    let mut out = vec![line];
    for _ in 1..lines { out.push(String::new()); }
    Ok(RenderedImage{width:cols, lines:out})
}

fn which(cmd:&str) -> bool { Command::new("sh").args(["-c", &format!("command -v {cmd} >/dev/null 2>&1")]).status().map(|s| s.success()).unwrap_or(false) }

pub fn render_external(mode:&str, path:&str, cols:usize, lines:usize) -> Result<RenderedImage,String> {
    let (cmd, args): (&str, Vec<String>) = match mode {
        "chafa" => ("chafa", vec![format!("--size={cols}x{lines}"), "--format=symbols".into(), path.into()]),
        "viu" => ("viu", vec!["-w".into(), cols.to_string(), "-h".into(), lines.to_string(), path.into()]),
        "jp2"|"jp2a" => ("jp2a", vec![format!("--width={cols}"), format!("--height={lines}"), path.into()]),
        "img2txt" => ("img2txt", vec![format!("--width={cols}"), format!("--height={lines}"), path.into()]),
        "timg" => ("timg", vec![format!("-g{}x{}", cols, lines), path.into()]),
        _ => return Err("unsupported external mode".into()),
    };
    if !which(cmd) { return Err(format!("{cmd} not found")); }
    let out = Command::new(cmd).args(&args).output().map_err(|e| e.to_string())?;
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    let lines_out: Vec<String> = s.lines().map(|l| l.to_string()).collect();
    Ok(RenderedImage{width:cols, lines:lines_out})
}

pub fn render(mode:&str, path:&str, cols:usize, lines:usize, bg: [u8;3]) -> Result<RenderedImage,String> {
    match mode {
        "blocks" => render_blocks(path,cols,lines,bg),
        "ascii" => render_ascii(path,cols,lines,bg),
        "kitty" => render_kitty(path,cols,lines),
        "iterm" => render_iterm(path,cols,lines),
        "chafa"|"viu"|"jp2"|"img2txt"|"timg" => render_external(mode,path,cols,lines),
        "auto" => {
            if std::env::var("KITTY_WINDOW_ID").is_ok() { return render_kitty(path,cols,lines); }
            if std::env::var("TERM_PROGRAM").ok().as_deref()==Some("iTerm.app") { return render_iterm(path,cols,lines); }
            for m in ["chafa","viu","jp2a","img2txt","timg"] { let mode_name = if m=="jp2a" {"jp2"} else {m}; if which(m) { return render_external(mode_name,path,cols,lines); } }
            render_blocks(path,cols,lines,bg)
        }
        _ => render_blocks(path,cols,lines,bg),
    }
}

pub fn image_dims(cols: Option<usize>, lines: Option<usize>) -> (usize, usize) {
    let (tw, _th) = term_size();
    let c = cols.unwrap_or_else(|| ((tw as f32*0.35) as usize).clamp(16,48));
    let l = lines.unwrap_or_else(|| ((c as f32*0.5) as usize).clamp(8,24));
    (c.max(4), l.max(2))
}
