<div align="center">

<img src="assets/banner.png" alt="nullfetch" width="100%">

**A tiny system fetch for Linux — pure Python, zero dependencies.**

[![Release](https://img.shields.io/github/v/release/pybindi/nullfetch?color=478061&label=release)](https://github.com/pybindi/nullfetch/releases)
[![Python](https://img.shields.io/badge/python-3.10%2B-3776AB?logo=python&logoColor=white)](https://www.python.org/)
[![Platform](https://img.shields.io/badge/platform-Linux-FCC624?logo=linux&logoColor=black)](#)
[![Dependencies](https://img.shields.io/badge/dependencies-none-brightgreen)](#)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Check](https://github.com/pybindi/nullfetch/actions/workflows/check.yml/badge.svg)](https://github.com/pybindi/nullfetch/actions/workflows/check.yml)

<img src="assets/demo.gif" alt="nullfetch in a terminal" width="720">

</div>

## ✨ Features

- 👤 current user and hostname
- 🐧 Linux distribution and kernel version
- 🖥️ CPU model and GPU
- 🧠 RAM usage
- ⌨️ terminal type
- 🎨 ASCII logo with bold labels
- ❓ `--help` and `--version`
- 🙈 `--no-logo` for a text-only output
- 📦 standard library only — nothing to install

## 📥 Installation

### Any Linux

```bash
git clone https://github.com/pybindi/nullfetch.git
cd nullfetch

chmod +x nullfetch.py
ln -s "$PWD/nullfetch.py" ~/.local/bin/nullfetch
```

Make sure `~/.local/bin` is in your `PATH`:

```bash
echo "$PATH" | grep -q "$HOME/.local/bin" && echo OK
```

### Arch Linux

A `PKGBUILD` is included, so you can build and install the package with `pacman`:

```bash
git clone https://github.com/pybindi/nullfetch.git
cd nullfetch
makepkg -si
```

`pacman` pulls `python` automatically. `pciutils` is optional and only needed
for GPU detection.

### Debian / Ubuntu

Debian packaging lives in `debian/`:

```bash
sudo apt install debhelper devscripts
dpkg-buildpackage -us -uc -b
sudo apt install ../nullfetch_1.1.0_all.deb
```

A prebuilt `.deb` is attached to every release.

## 🚀 Usage

```bash
nullfetch            # logo + system info
nullfetch --no-logo  # text only
```

### Options

| Option | Description |
| :--- | :--- |
| `-h`, `--help` | Show help and exit |
| `--version` | Show version and exit |
| `--no-logo` | Hide the ASCII logo |

## 🖼 Example output

```text
--------------------------
        .--.
       |o_o |
       |:_/ |
      //   \ \
     (|     | )
    /'\_   _/`\
    \___)=(___/

USER: matuha
HOST: void
OS: Void Linux
kernel: 6.18.60_1
terminal: xterm-256color
CPU: Intel(R) Core(TM) i7-4600U CPU @ 2.10GHz
GPU: Intel Corporation Haswell-ULT Integrated Graphics Controller
RAM: 3 / 7 GB
--------------------------
```

## 🎨 Custom logo

[`logos.py`](logos.py) collects a set of ASCII art. Pick one and paste it into
`printlogo()`:

```python
def printlogo():
    print(r"""        .--.
       |o_o |
       |:_/ |
      //   \ \
     (|     | )
    /'\_   _/`\
    \___)=(___/""")
```

> `r"""..."""` is a raw multi-line string, so backslashes in the art do not
> need to be escaped.

## 🗂 Project structure

```text
nullfetch/
├── nullfetch.py        # the fetch itself
├── logos.py            # ASCII art collection
├── assets/
│   ├── banner.png      # README header
│   ├── demo.gif        # animated preview
│   └── demo.svg        # static preview
├── debian/             # Debian packaging
├── PKGBUILD            # Arch Linux package
├── .github/workflows/  # syntax check on push
├── LICENSE
└── README.md
```

## 🧩 Requirements

- Python **3.10+**
- Linux (`platform.freedesktop_os_release`)

## 📄 License

[MIT](LICENSE) — free to use, modify and share.
