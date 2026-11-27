<div align="center">

<img src="assets/banner.png" alt="nullfetch" width="100%">

**A tiny system fetch for Linux — pure Python, zero required dependencies.**

[![Release](https://img.shields.io/github/v/release/pybindi/nullfetch?color=478061&label=release)](https://github.com/pybindi/nullfetch/releases)
[![Python](https://img.shields.io/badge/python-3.10%2B-3776AB?logo=python&logoColor=white)](https://www.python.org/)
[![Platform](https://img.shields.io/badge/platform-Linux-FCC624?logo=linux&logoColor=black)](#)
[![Dependencies](https://img.shields.io/badge/dependencies-none-brightgreen)](#)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Check](https://github.com/pybindi/nullfetch/actions/workflows/check.yml/badge.svg)](https://github.com/pybindi/nullfetch/actions/workflows/check.yml)

<img src="assets/demo.gif" alt="nullfetch in a terminal" width="1000">

</div>

## ✨ Features

- 👤 28 info modules: user, host, OS, kernel, arch, uptime, CPU, cores, GPU,
  RAM, swap, disk, mounts, network, local/public IP, battery, temperature,
  resolution, shell, terminal, DE, WM, packages, processes, locale, timezone
  and date
- 🖼 terminal images: built-in PNG decoder, half-block or ASCII rendering,
  Kitty and iTerm2 graphics protocols, plus optional `chafa`, `viu`, `jp2`,
  `img2txt` and `timg`
- ⚡ parallel module collection, cached slow values and `--timing` for profiling
- 🎨 9 themes, named/hex accents and per-field separators
- 📊 customizable bars (`blocks`, `ascii`, `arrows`, `dots`, `lines`) and width
- 🧩 text, JSON, CSV, Markdown and one-line output formats
- 🖥 adapts to narrow terminals, `--ascii` for plain terminals
- 💾 persistent config with modules, image settings and `--set` / `--init-config`
- 📦 standard library only — image converters are optional

## 📥 Installation

### Any Linux

```bash
git clone https://github.com/pybindi/nullfetch.git
cd nullfetch
./install.sh
```

The installer copies `nullfetch.py` to `~/.local/bin/nullfetch`, the logo and
image modules to `~/.local/share/nullfetch`, and installs bash, zsh and fish
completions. To install them separately:

```bash
./scripts/install-completions.sh
```

Use another prefix with `PREFIX=/usr/local sudo ./install.sh`.

Manual install:

```bash
chmod +x nullfetch.py
ln -s "$PWD/nullfetch.py" ~/.local/bin/nullfetch
```

Keep `nullfetch_image.py` and `logos.py` next to the script or in
`$prefix/share/nullfetch/` for image and logo support.

### Arch Linux

A `PKGBUILD` is included:

```bash
git clone https://github.com/pybindi/nullfetch.git
cd nullfetch
makepkg -si
```

Set the release archive checksum in `PKGBUILD` / `.SRCINFO` after tagging a
release.

### Debian / Ubuntu

A prebuilt `.deb` is attached to every release:

```bash
sudo apt install ./nullfetch_2.0.0_all.deb
```

Or build it from source:

```bash
sudo apt install dpkg-dev
./scripts/build-deb.sh
sudo apt install ./dist/nullfetch_2.0.0_all.deb
```

Each release also includes `SHA256SUMS` for verifying the downloaded package:

```bash
sha256sum -c SHA256SUMS
```

## 🚀 Usage

```bash
nullfetch                          # logo + default modules
nullfetch --all                    # every module
nullfetch --only cpu,ram,temperature
nullfetch --list-fields            # all available module names
nullfetch --format markdown        # table output
nullfetch --format csv             # machine-readable table
nullfetch --json                   # machine-readable object
nullfetch --oneline                # status bar
nullfetch --ascii                  # no block characters
nullfetch --theme dracula          # another palette
nullfetch --bar-style dots --separator ' = '
nullfetch --timing                 # show collection time on stderr
```

### Images

```bash
nullfetch --image ~/wallpaper.png
nullfetch --image photo.jpg --image-mode chafa --image-position top
nullfetch --image avatar.png --image-mode blocks --image-width 30
nullfetch --image logo.png --image-mode ascii --color never
nullfetch --image logo.png --image-mode kitty
```

`--image-mode auto` (the default) chooses the best renderer for your terminal:

| Mode | Requirements | Notes |
| :--- | :--- | :--- |
| `auto` | — | Kitty → iTerm2 → chafa → built-in PNG renderer |
| `blocks` | PNG | Two pixels per character, truecolor or 256 colors |
| `ascii` | PNG | Brightness ramp, works without colors |
| `kitty` | Kitty terminal, PNG | Native graphics protocol |
| `iterm` | iTerm2 / WezTerm | Inline image protocol, PNG/JPEG/GIF/WebP |
| `chafa` / `sixel` | [chafa](https://hpjansson.org/chafa/) | Best quality, many formats |
| `viu` / `jp2` / `img2txt` / `timg` | respective tool | Fallback converters |
| `--image-position` | `side`, `top`, `bottom` | Where to place the image |
| `--image-width`, `--image-height` | numbers | Size in terminal cells |
| `--image-bg` | color name or `#rrggbb` | Behind transparent pixels |

The built-in decoder supports non-interlaced PNG images (greyscale, palette,
RGB, RGBA, 1/2/4/8/16-bit samples). Interlaced PNG and other formats need one
of the optional converters or the iTerm2 protocol. Rendered images are cached
under `$XDG_CACHE_HOME/nullfetch` and refreshed when the file changes; use
`--no-cache` to bypass the cache.

### Modules

Default modules: `user host os kernel cpu gpu ram disk`.

Additional modules: `arch uptime cpu_cores swap disks network local_ip
public_ip battery temperature resolution shell terminal de wm packages
processes locale timezone date`.

```bash
nullfetch --all
nullfetch --only uptime,shell,de,wm,packages
nullfetch --all --no-network
```

Slow modules (`public_ip`, `packages`, `resolution`, temperature and battery)
are collected in parallel with the rest and cached with a short TTL. Set the
TTL with `--cache-ttl N` or disable caching with `--no-cache`. `public_ip`
queries `api.ipify.org` with a 3 second timeout and only when requested.

### Configuration

Save the current appearance and module selection:

```bash
nullfetch --logo cat --accent orange --theme neon --save-config
nullfetch --only os,cpu,ram --bar-style dots --save-config
nullfetch --show-config
nullfetch --reset-config
```

Start from an example config or edit a single key:

```bash
nullfetch --init-config
nullfetch --set image=$HOME/avatar.png
nullfetch --set modules=os,cpu,ram,temperature
nullfetch --set theme=gruvbox
```

The config lives at `~/.config/nullfetch/config.json` (or
`$XDG_CONFIG_HOME/nullfetch/config.json`). Use `--config FILE` for a different
file. Command line options always override saved values. Available keys:
`logo`, `logo-file`, `accent`, `theme`, `color`, `image`, `image-mode`,
`image-width`, `image-height`, `image-position`, `image-bg`, `modules`,
`separator`, `bar-style`, `bar-width`, `format`, `cache-ttl`, `ascii`,
`compact`, `all`, `no-cache`, `no-network`, `timing`.

Colors accept names (`black red green yellow blue magenta cyan white gray
grey orange pink purple tux`), distribution names (`void`, `arch`, `debian`,
`ubuntu`, `fedora`, `opensuse`, `alpine`, `gentoo`, `nixos`, `manjaro`, `mint`,
`kali`, `raspbian`, `pop`), `#rrggbb` and `#rgb`.

Themes: `default mono light neon dark nord gruvbox dracula tokyo solarized
matrix`.

### Options

| Option | Description |
| :--- | :--- |
| `-h`, `--help` | Show help and exit |
| `--version` | Show version and exit |
| `--no-logo` | Hide the ASCII logo |
| `--logo NAME` | `auto`, `none` or a name from `logos.py` |
| `--logo-file FILE` | Custom ASCII art file |
| `--image FILE` | Image to display |
| `--image-mode MODE` | Image renderer (see table above) |
| `--image-width N`, `--image-height N` | Image size in cells |
| `--image-position P` | `side`, `top` or `bottom` |
| `--image-bg COLOR` | Background for transparent pixels |
| `--accent COLOR` | Accent color |
| `--color MODE` | `auto`, `always` or `never` |
| `--theme NAME` | Visual theme |
| `--compact` | Tighter output |
| `--all` | Show every module |
| `--list-fields` | List module names |
| `--list-image-modes` | List image renderers |
| `--separator TEXT` | Text between label and value |
| `--bar-width N` | Width of usage bars |
| `--bar-style STYLE` | `blocks`, `ascii`, `arrows`, `dots`, `lines` |
| `--format FORMAT` | `text`, `json`, `csv`, `markdown`, `oneline` |
| `--no-cache`, `--cache-ttl N` | Control slow-value cache |
| `--no-network` | Skip network modules |
| `--timing` | Print collection time to stderr |
| `--save-config`, `--show-config` | Save or show defaults |
| `--reset-config`, `--init-config` | Remove defaults or write an example |
| `--set KEY=VALUE` | Change one config key |
| `--config FILE` | Alternate config file |
| `--no-color` | Disable colors (also honors `NO_COLOR`) |
| `--ascii` | ASCII-only output |
| `--width N` | Output width in columns |
| `--oneline` | One compact line |
| `--list-logos` | List available logos |
| `-o`, `--only FIELDS` | Show only these modules |
| `--json` | Print JSON |

### Shell completion

```bash
# bash
source scripts/completions/nullfetch.bash

# zsh
source scripts/completions/_nullfetch

# fish
source scripts/completions/nullfetch.fish
```

Colors automatically turn off when output is redirected. Use `--color always`
to force them or `--color never` to disable them.

## ⚡ Speed

Modules are gathered concurrently, repeated `/etc/os-release` reads are
memoized, and slow values are cached. Measure your machine:

```bash
time nullfetch >/dev/null
nullfetch --timing
```

For the smallest possible startup use a short module list:

```bash
nullfetch --only os,kernel,cpu,ram --no-logo
```

## 🖼 Example output

```text
      ________________         USER: matuha
     /                \        HOST: void
    /                  \       OS: Void Linux
   /                    \      kernel: 6.18.60_1
  /       ________       \     CPU: Intel(R) Core(TM) i7-4600U CPU @ 2.10GHz
 /_______|        |_______\    GPU: Intel Corporation Haswell-ULT Integrated Graphics Controller
  _________  V0ID   ________    RAM: 2.4 / 7.7 GiB  ███░░░░░░░░░  31%
  \       |________|       /    disk: 31.2 / 237.4 GiB ██░░░░░░░░░  13%
  \                      /
   \                    /
    \                  /
     \________________/
```

## 🤖 JSON output

```bash
nullfetch --json
```

```json
{
  "user": "matuha",
  "host": "void",
  "os": "Void Linux",
  "kernel": "6.18.60_1",
  "cpu": "Intel(R) Core(TM) i7-4600U CPU @ 2.10GHz",
  "gpu": "Intel Corporation Haswell-ULT Integrated Graphics Controller",
  "ram": { "used": 2.4, "total": 7.7, "percent": 31 },
  "disk": { "used": 31.2, "total": 237.4, "percent": 13 }
}
```

`--format csv` and `--format markdown` use the same module selection.

## 🎨 Custom logo

[`logos.py`](logos.py) collects ASCII art. Pick one and paste it into the
`ART` string in `nullfetch.py`, or use `--logo-file FILE` without editing code:

```bash
nullfetch --logo-file ~/art.txt
```

The logo is colored with your distribution color unless `--accent` or a theme
overrides it.

## 🗂 Project structure

```text
nullfetch/
├── nullfetch.py        # CLI, modules and rendering
├── nullfetch_image.py  # PNG decoder and terminal image protocols
├── pyproject.toml      # packaging and lint/test config
├── logos.py            # ASCII art collection
├── nullfetch.1         # man page
├── install.sh          # installer for ~/.local
├── assets/
│   ├── banner.png
│   ├── demo.gif
│   ├── demo.svg
│   └── logos.png
├── scripts/
│   ├── build-deb.sh
│   └── completions/
├── tests/
├── debian/
├── PKGBUILD
└── README.md
```

## 🧩 Requirements

- Python **3.10+**
- Linux (`platform.freedesktop_os_release`)
- optional: `pciutils` for GPU detection
- optional: `chafa`, `viu`, `jp2`, `img2txt` or `timg` for other image formats

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Tests are run with:

```bash
python -m pytest -q
```

## 📄 License

[MIT](LICENSE) — free to use, modify and share.
