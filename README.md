<div align="center">

<img src="assets/banner.png" alt="nullfetch banner" width="100%">

# nullfetch

**Small system fetch. Fast Rust core, Python compatibility layer.**  
Linux-first, no required dependencies, terminal images, presets, command modules, script-friendly output.

<p>
  <a href="https://github.com/pybindi/nullfetch/releases"><img src="https://img.shields.io/github/v/release/pybindi/nullfetch?style=for-the-badge&color=478061&label=release" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-2ea44f?style=for-the-badge" alt="MIT License"></a>
  <a href="#"><img src="https://img.shields.io/badge/platform-Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black" alt="Linux"></a>
  <a href="#"><img src="https://img.shields.io/badge/deps-zero-brightgreen?style=for-the-badge" alt="Zero dependencies"></a>
</p>

</div>

---

## What it is

`nullfetch` is a fast system information tool for your terminal.  
Version 3.0 adds a **Rust core** for fast startup and keeps the **Python implementation** for compatibility/fallback.

- **Rust binary**: fast default run, parallel module collection, cheap `/proc`/`/sys` reads.
- **Python CLI (`nullfetch.py`)**: full existing behavior, tests, images/logos/config/features developed in 1.x/2.x.
- **Windows**: `nullfetch.bat` launcher (Python required).

> Honest status: the Rust core is fast and covers the main CLI/modules, but not every Python feature is fully ported yet. For full legacy behavior, use `python3 nullfetch.py ...` or install without cargo.

## Benchmarks (this machine, warmup, `NO_COLOR=1`)

| Command | Median | Min |
|---|---:|---:|
| `nullfetch --no-color --logo none` (Rust, 8 default fields) | 4.1 ms | 3.8 ms |
| `nullfetch --only os,kernel,cpu,ram --no-color --logo none` (Rust) | 1.1 ms | 1.0 ms |
| `fastfetch --logo none` | 8.0 ms | 7.3 ms |

Results depend on hardware, OS, cache, and selected modules.

## Install

### Linux (recommended)

```bash
git clone https://github.com/pybindi/nullfetch.git
cd nullfetch
./install.sh
```

`install.sh` behavior:
- If `cargo` is available: builds and installs `target/release/nullfetch`.
- Otherwise: installs `nullfetch.py` as `nullfetch`.

Make sure `~/.local/bin` is in `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Manual Python run

```bash
python3 nullfetch.py --all
```

### Windows

```bat
git clone https://github.com/pybindi/nullfetch.git
cd nullfetch
nullfetch.bat --all
```

Requires Python 3.10+.

## Usage

```bash
nullfetch                          # default fields
nullfetch --all                    # all modules
nullfetch --only cpu,ram,temperature
nullfetch --preset dev
nullfetch --doctor
nullfetch --timing
nullfetch --format json
nullfetch --format markdown --only os,cpu,ram
nullfetch --oneline
```

### Command modules

You can add custom fields:

```bash
nullfetch --only os,command:git:git\ branch\ --show-current
```

In config `modules`, use `command:<name>:<cmd>`.

## Modules

**Default:** `user host os kernel cpu gpu ram disk`

**Extra:** `arch uptime boot_time loadavg cpu_cores swap disks network local_ip public_ip battery temperature resolution shell terminal de wm packages processes init locale timezone date motherboard bios display audio bluetooth power`

```bash
nullfetch --list-fields
nullfetch --list-presets
```

## Config

Config path: `~/.config/nullfetch/config.json` (or `$XDG_CONFIG_HOME/nullfetch/config.json`).

```bash
nullfetch --init-config
nullfetch --show-config
nullfetch --set theme=gruvbox
nullfetch --set modules=os,cpu,ram,command:git:git\ branch\ --show-current
nullfetch --reset-config
```

CLI flags override config values.

## Output formats

- `text` (default)
- `json`
- `csv`
- `markdown`
- `oneline`

Default `--json` keeps the original 8 fields for compatibility unless you select fields explicitly.

## Images / logos

The Python implementation includes logo rendering and terminal image support (`nullfetch_image.py`): PNG blocks/ascii, Kitty/iTerm2, and optional external converters (`chafa`, `viu`, `jp2a`, `img2txt`, `timg`, `sixel`).

```bash
python3 nullfetch.py --logo arch
python3 nullfetch.py --image ~/wallpaper.png --image-mode blocks
python3 nullfetch.py --list-image-modes
```

Rust core currently focuses on fast data collection; image/logo parity is being ported.

## Project layout

```text
.
├── src/                 # Rust core (v3)
├── nullfetch.py         # Python CLI/compat layer
├── nullfetch_image.py   # Python image rendering
├── logos.py             # ASCII logos
├── nullfetch.bat        # Windows launcher
├── scripts/             # completions, packaging helpers
├── tests/               # Python pytest suite
├── debian/              # Debian packaging
└── PKGBUILD             # Arch packaging
```

## Development

```bash
# Python tests
python3 -m pytest -q

# Rust build
cargo build --release
```

## License

MIT. See [LICENSE](LICENSE).
