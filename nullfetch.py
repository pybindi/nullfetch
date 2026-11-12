#!/usr/bin/env python3
"""nullfetch - a tiny system fetch for Linux."""

import getpass
import json
import os
import platform
import shutil
import subprocess
import sys
from itertools import zip_longest
from pathlib import Path

VERSION = "1.3.0"

BOLD = "\033[1m"
RESET = "\033[0m"
BAR_WIDTH = 12

FIELDS = [
    "user",
    "host",
    "os",
    "kernel",
    "cpu",
    "gpu",
    "ram",
    "disk",
]

LABELS = {
    "user": "USER",
    "host": "HOST",
    "os": "OS",
    "kernel": "kernel",
    "cpu": "CPU",
    "gpu": "GPU",
    "ram": "RAM",
    "disk": "disk",
}

DISTRO_COLORS = {
    "void": (71, 128, 97),
    "arch": (23, 147, 209),
    "debian": (215, 10, 83),
    "ubuntu": (233, 84, 32),
    "fedora": (41, 65, 122),
    "opensuse": (115, 186, 37),
    "alpine": (13, 89, 123),
    "gentoo": (84, 64, 125),
    "nixos": (82, 119, 183),
    "manjaro": (53, 191, 96),
    "mint": (134, 190, 50),
    "kali": (38, 87, 173),
    "raspbian": (190, 26, 66),
    "pop": (72, 185, 199),
}

DEFAULT_COLOR = (88, 166, 255)

ART = r"""        .--.
       |o_o |
       |:_/ |
      //   \ \
     (|     | )
    /'\_   _/`\
    \___)=(___/"""

HELP = f"""nullfetch {VERSION} - PC info

Usage:
  nullfetch [options]

Options:
  -h, --help          show this help
      --version       show version
      --no-logo       hide the ASCII logo
      --no-color      disable colors
  -o, --only FIELDS   show only these fields, comma separated
      --json          print system info as JSON

Fields:
  {" ".join(FIELDS)}

Examples:
  nullfetch --only cpu,ram
  nullfetch --no-logo --json
"""


# --- reading system information -------------------------------------------


def read(path):
    try:
        return Path(path).read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def os_release():
    try:
        return platform.freedesktop_os_release()
    except OSError:
        return {}


def get_user():
    try:
        return getpass.getuser()
    except (KeyError, OSError):
        return os.environ.get("USER", "?")


def get_host():
    return platform.node() or "?"


def get_os():
    release = os_release()
    return release.get("PRETTY_NAME") or release.get("NAME") or "Linux"


def get_kernel():
    return platform.release()


def get_cpu():
    for line in read("/proc/cpuinfo").splitlines():
        if line.startswith("model name"):
            return line.split(":", 1)[1].strip()
    return "?"


def get_gpu():
    if not shutil.which("lspci"):
        return "?"
    try:
        out = subprocess.run(
            ["lspci"], capture_output=True, text=True, timeout=5
        ).stdout
    except (OSError, subprocess.SubprocessError):
        return "?"
    for line in out.splitlines():
        if "VGA compatible controller" in line or "3D controller" in line:
            return line.split(": ", 1)[1].split(" (rev")[0]
    return "?"


def size_value(used, total):
    percent = round(used / total * 100) if total else 0
    return {"used": used, "total": total, "percent": percent}


def get_ram():
    info = {}
    for line in read("/proc/meminfo").splitlines():
        if ":" in line:
            key, value = line.split(":", 1)
            info[key] = value.split()

    total_kb = int(info.get("MemTotal", [0])[0])
    available_kb = int(info.get("MemAvailable", info.get("MemFree", [0]))[0])
    if not total_kb:
        return "?"

    total = total_kb // 1024 // 1024
    used = (total_kb - available_kb) // 1024 // 1024
    return size_value(used, total)


def get_disk():
    try:
        st = os.statvfs("/")
    except OSError:
        return "?"

    gb = 1024 ** 3
    total = st.f_blocks * st.f_frsize
    free = st.f_bavail * st.f_frsize
    return size_value((total - free) // gb, total // gb)


GETTERS = {
    "user": get_user,
    "host": get_host,
    "os": get_os,
    "kernel": get_kernel,
    "cpu": get_cpu,
    "gpu": get_gpu,
    "ram": get_ram,
    "disk": get_disk,
}


def collect(fields):
    data = {}
    for key in fields:
        try:
            data[key] = GETTERS[key]()
        except Exception:
            data[key] = "?"
    return data


# --- colors ---------------------------------------------------------------


def color_enabled():
    return not os.environ.get("NO_COLOR")


def rgb_to_256(r, g, b):
    if r == g == b:
        if r < 8:
            return 16
        if r > 248:
            return 231
        return 232 + round((r - 8) / 247 * 24)
    return 16 + 36 * round(r / 255 * 5) + 6 * round(g / 255 * 5) + round(b / 255 * 5)


def distro_color():
    release = os_release()
    names = [release.get("ID", "").lower()]
    names += release.get("ID_LIKE", "").lower().split()

    for name in names:
        if name in DISTRO_COLORS:
            return DISTRO_COLORS[name]
    return DEFAULT_COLOR


def fg(rgb):
    r, g, b = rgb
    if os.environ.get("COLORTERM", "").lower() in ("truecolor", "24bit"):
        return f"\033[38;2;{r};{g};{b}m"
    return f"\033[38;5;{rgb_to_256(r, g, b)}m"


# --- output ---------------------------------------------------------------


def format_size(value, use_color):
    text = f"{value['used']} / {value['total']} GB"
    percent = value["percent"]
    filled = round(percent / 100 * BAR_WIDTH)
    empty = BAR_WIDTH - filled

    if use_color:
        bar = f"{fg(distro_color())}{'█' * filled}{RESET}{'░' * empty}"
    else:
        bar = "█" * filled + "░" * empty

    return f"{text:<14}{bar} {percent:>3}%"


def info_lines(data, use_color):
    lines = []
    for key, value in data.items():
        label = LABELS[key]
        if isinstance(value, dict):
            value = format_size(value, use_color)
        if use_color:
            lines.append(f"{BOLD}{label}:{RESET} {value}")
        else:
            lines.append(f"{label}: {value}")
    return lines


def render(data, show_logo, use_color):
    lines = info_lines(data, use_color)
    if not show_logo:
        return "\n".join(lines)

    art = ART.splitlines()
    width = max(len(line) for line in art) + 4

    if use_color:
        prefix = fg(distro_color())
        art = [prefix + line.ljust(width) + RESET for line in art]
    else:
        art = [line.ljust(width) for line in art]

    rows = []
    for left, right in zip_longest(art, lines, fillvalue=""):
        if not left:
            left = " " * width
        rows.append((left + right).rstrip())
    return "\n".join(rows)


# --- command line ---------------------------------------------------------


def fail(message):
    print(f"nullfetch: {message}", file=sys.stderr)
    print("try 'nullfetch --help'", file=sys.stderr)
    sys.exit(2)


def parse_args(argv):
    opts = {
        "help": False,
        "version": False,
        "logo": True,
        "color": color_enabled(),
        "json": False,
        "only": None,
    }

    i = 0
    while i < len(argv):
        arg = argv[i]

        if arg in ("-h", "--help"):
            opts["help"] = True
        elif arg == "--version":
            opts["version"] = True
        elif arg == "--no-logo":
            opts["logo"] = False
        elif arg == "--no-color":
            opts["color"] = False
        elif arg == "--json":
            opts["json"] = True
        elif arg in ("-o", "--only"):
            i += 1
            if i == len(argv):
                fail(f"{arg} needs a value")
            opts["only"] = argv[i]
        elif arg.startswith("--only="):
            opts["only"] = arg.split("=", 1)[1]
        else:
            fail(f"unknown option: {arg}")

        i += 1

    return opts


def main(argv=None):
    if argv is None:
        argv = sys.argv[1:]

    opts = parse_args(argv)

    if opts["help"]:
        print(HELP)
        return 0

    if opts["version"]:
        print(f"nullfetch {VERSION}")
        return 0

    fields = FIELDS
    if opts["only"] is not None:
        fields = [name.strip().lower() for name in opts["only"].split(",")]
        fields = [name for name in fields if name]
        if not fields:
            fail("--only needs at least one field")
        unknown = [name for name in fields if name not in FIELDS]
        if unknown:
            fail(f"unknown field: {', '.join(unknown)}")

    data = collect(fields)

    if opts["json"]:
        print(json.dumps(data, indent=2, ensure_ascii=False))
        return 0

    print(render(data, opts["logo"], opts["color"]))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv[1:]))
    except BrokenPipeError:
        sys.exit(0)
