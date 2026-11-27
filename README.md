<div align="center">

<img src="assets/banner.png" alt="nullfetch banner" width="100%">

# nullfetch

**Fast system fetch, written in Rust.**  
Linux, macOS, Windows. Zero runtime dependencies. Logos, images, presets, command modules, script-friendly output.

<p>
  <a href="https://github.com/pybindi/nullfetch/releases"><img src="https://img.shields.io/github/v/release/pybindi/nullfetch?style=for-the-badge&color=478061&label=release" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-2ea44f?style=for-the-badge" alt="MIT License"></a>
  <a href="#"><img src="https://img.shields.io/badge/runtime_deps-zero-brightgreen?style=for-the-badge" alt="Zero runtime deps"></a>
</p>

</div>

---

## What it is

`nullfetch` prints system info in your terminal, fast.  
v3 is Rust-native and aimed at quick startup with practical modules.

## Benchmarks (Linux reference machine, warmup, `NO_COLOR=1`)

| Command | Median | Min |
|---|---:|---:|
| `nullfetch --no-color --logo none` (8 default fields) | 4.1 ms | 3.8 ms |
| `nullfetch --only os,kernel,cpu,ram --no-color --logo none` | 1.1 ms | 1.0 ms |
| `fastfetch --logo none` | 8.0 ms | 7.3 ms |

Results depend on hardware, OS, cache, and selected modules.

## Install

### Cargo (all platforms with Rust)

```bash
cargo install --git https://github.com/pybindi/nullfetch.git
```

### From source

```bash
git clone https://github.com/pybindi/nullfetch.git
cd nullfetch
cargo build --release
```

Linux/macOS:

```bash
install -Dm755 target/release/nullfetch ~/.local/bin/nullfetch
# or
./install.sh
```

Windows (PowerShell):

```powershell
cargo build --release
Copy-Item .\target\release\nullfetch.exe $env:USERPROFILE\.local\bin\nullfetch.exe
```

Or run `nullfetch.bat` after build.

### Arch Linux (PKGBUILD)

```bash
git clone https://github.com/pybindi/nullfetch.git
cd nullfetch
makepkg -si
```

### Debian/Ubuntu (.deb)

```bash
./scripts/build-deb.sh
sudo dpkg -i dist/nullfetch_*_*.deb
```

### Void Linux (XBPS template)

Template is in `packaging/xbps/template`. Copy it into your `void-packages/srcpkgs/nullfetch/template`, set checksum, then:

```bash
./xbps-src pkg nullfetch
sudo xbps-install -R hostdir/binpkgs nullfetch
```

### Homebrew (formula template)

Template: `packaging/homebrew/nullfetch.rb`.

```bash
brew install --build-from-source ./packaging/homebrew/nullfetch.rb
```

### Scoop (Windows, template)

Template: `packaging/scoop/nullfetch.json`.

```powershell
scoop install ./packaging/scoop/nullfetch.json
```

### Winget / Chocolatey

Submission templates are in `packaging/winget/` and `packaging/chocolatey/`. Replace version/checksum before publishing.

### Prebuilt binaries

Release workflow builds archives for:
- Linux x86_64/aarch64
- macOS x86_64/aarch64
- Windows x86_64

Download from GitHub Releases.

## Usage

```bash
nullfetch
nullfetch --all
nullfetch --only cpu,ram,temperature
nullfetch --preset dev
nullfetch --format json
nullfetch --format markdown --only os,cpu,ram
nullfetch --oneline
nullfetch --doctor
nullfetch --timing
```

### Command modules

```bash
nullfetch --only os,command:git:git\ branch\ --show-current
```

## Modules

**Default:** `user host os kernel cpu gpu ram disk`

**Extra:** `arch uptime boot_time loadavg cpu_cores swap disks network local_ip public_ip battery temperature resolution shell terminal de wm packages processes init locale timezone date motherboard bios display audio bluetooth power`

## Config

Config path: `~/.config/nullfetch/config.json` / `config.jsonc` (or `$XDG_CONFIG_HOME/...` on Unix; `%APPDATA%\nullfetch\...` on Windows).

```bash
nullfetch --gen-config
nullfetch --show-config
nullfetch --set theme=gruvbox
nullfetch --reset-config
```

JSONC is supported. Modules can be strings or objects:

```jsonc
{
  "modules": [
    "os",
    { "type": "cpu", "key": "CPU", "format": "{value}", "keyColor": "#7aa2f7" },
    { "type": "command:git:git branch --show-current", "key": "Git" }
  ]
}
```

## Output formats

`text`, `json`, `csv`, `markdown`, `oneline`.

## Images / logos

```bash
nullfetch --logo arch
nullfetch --image ~/wallpaper.png --image-mode blocks
nullfetch --list-image-modes
```

Modes: `auto blocks ascii kitty iterm chafa viu jp2 img2txt timg sixel` (external modes use installed tools when available).

## Development

```bash
cargo build --release
cargo test
python3 -m pytest -q --ignore=nullfetch --ignore=target
```

## License

MIT. See [LICENSE](LICENSE).
