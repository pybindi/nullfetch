<div align="center">

# nullfetch

**A tiny system fetch for Linux — pure Python, zero dependencies.**

[![Python](https://img.shields.io/badge/python-3.10%2B-3776AB?logo=python&logoColor=white)](https://www.python.org/)
[![Platform](https://img.shields.io/badge/platform-Linux-FCC624?logo=linux&logoColor=black)](#)
[![No dependencies](https://img.shields.io/badge/dependencies-none-brightgreen)](#)
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
- ❓ built-in `--help` and `--version`
- 🙈 `--no-logo` for a text-only output
- 📦 standard library only — nothing to install

## 📦 Installation

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

A `PKGBUILD` for the `nullfetch` package is included, so you can build and
install it with `pacman`:

```bash
git clone https://github.com/pybindi/nullfetch.git
cd nullfetch
makepkg -si
```

`pacman` will pull the dependency (`python`) automatically. `pciutils` is
optional and only needed for GPU detection.

Once published to the AUR, it can be installed with an AUR helper:

```bash
yay -S nullfetch
```

### Debian / Ubuntu

Debian packaging lives in `debian/`. On a Debian-based system:

```bash
sudo apt install debhelper devscripts
dpkg-buildpackage -us -uc -b
sudo apt install ../nullfetch_1.0.0_all.deb
```

A prebuilt package is also available in the releases page.

## 🚀 Usage

Run from the project directory:

```bash
python nullfetch.py
```

Or as a command from anywhere:

```bash
nullfetch
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

[`logos.py`](logos.py) contains a collection of ASCII art. Pick one and paste
it into `printlogo()`:

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

> `r"""..."""` is a raw multi-line string, so backslashes in the art
> do not need to be escaped.

## 🗂 Project structure

```text
nullfetch/
├── nullfetch.py        # the fetch itself
├── logos.py            # ASCII art collection
├── assets/
│   └── demo.svg        # README preview
├── .github/workflows/  # syntax check on push
├── LICENSE
└── README.md
```

## 🧩 Requirements

- Python **3.10+**
- Linux (`platform.freedesktop_os_release`)

## 📄 License

[MIT](LICENSE) — free to use, modify and share.
