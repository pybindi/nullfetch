<div align="center">

<img src="assets/banner.png" alt="nullfetch banner" width="100%">

# nullfetch

**Fast system fetch for Linux, written in Rust.**  
Zero runtime dependencies. Logos, images, presets, command modules, script-friendly output.

<p>
  <a href="https://github.com/pybindi/nullfetch/releases"><img src="https://img.shields.io/github/v/release/pybindi/nullfetch?style=for-the-badge&color=478061&label=release" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-2ea44f?style=for-the-badge" alt="MIT License"></a>
  <a href="#"><img src="https://img.shields.io/badge/platform-Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black" alt="Linux"></a>
  <a href="#"><img src="https://img.shields.io/badge/runtime_deps-zero-brightgreen?style=for-the-badge" alt="Zero runtime deps"></a>
</p>

</div>

---

## What it is

`nullfetch` prints system info in your terminal, fast.  
Version 3 is a Rust rewrite focused on startup speed and practical output.

## Benchmarks (this machine, warmup, `NO_COLOR=1`)

| Command | Median | Min |
|---|---:|---:|
| `nullfetch --no-color --logo none` (8 default fields) | 4.1 ms | 3.8 ms |
| `nullfetch --only os,kernel,cpu,ram --no-color --logo none` | 1.1 ms | 1.0 ms |
| `fastfetch --logo none` | 8.0 ms | 7.3 ms |

Results depend on hardware, OS, cache, and selected modules.

## Install

### From source

```bash
git clone https://github.com/pybindi/nullfetch.git
cd nullfetch
cargo build --release
install -Dm755 target/release/nullfetch ~/.local/bin/nullfetch
```

Or use:

```bash
./install.sh
```

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

In config: `command:<name>:<cmd>`.

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

CLI flags override config.

## Output formats

`text`, `json`, `csv`, `markdown`, `oneline`.

Default `--json` keeps the original 8 fields for compatibility unless fields are selected explicitly.

## Images / logos

```bash
nullfetch --logo arch
nullfetch --image ~/wallpaper.png --image-mode blocks
nullfetch --image ~/wallpaper.png --image-mode kitty
nullfetch --list-image-modes
```

Built-in image modes include `blocks`, `ascii`, `kitty`, `iterm`; external converters are used when available (`chafa`, `viu`, `jp2a`, `img2txt`, `timg`).

## Project layout

```text
.
├── src/            # Rust source
├── tests/          # CLI smoke tests
├── scripts/        # completions, packaging helpers
├── debian/         # Debian packaging
├── PKGBUILD        # Arch packaging
└── nullfetch.1     # man page
```

## Development

```bash
cargo build --release
cargo test
python3 -m pytest -q --ignore=nullfetch --ignore=target
```

## License

MIT. See [LICENSE](LICENSE).
