#!/usr/bin/env python3
"""nullfetch - a tiny system fetch for Linux."""

import getpass
import importlib.util
import json
import os
import platform
import shutil
import subprocess
import sys
from itertools import zip_longest
from pathlib import Path

VERSION = "1.7.0"

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


def _load_logos():
    try:
        from logos import LOGOS

        return dict(LOGOS)
    except Exception:
        pass

    here = Path(__file__).resolve().parent
    prefix = here.parent
    for path in (
        here / "logos.py",
        prefix / "share" / "nullfetch" / "logos.py",
        Path("/usr/share/nullfetch/logos.py"),
        Path("/usr/local/share/nullfetch/logos.py"),
    ):
        if not path.is_file():
            continue
        try:
            spec = importlib.util.spec_from_file_location("_nullfetch_logos", path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            return dict(module.LOGOS)
        except Exception:
            continue
    return {}


LOGOS = _load_logos()
LOGO_NAMES = ", ".join(sorted(LOGOS))

HELP = f"""nullfetch {VERSION} - PC info

Usage:
  nullfetch [options]

Options:
  -h, --help          show this help
      --version       show version
      --no-logo       hide the ASCII logo
      --logo NAME     logo: auto, none, or one of:
                      {LOGO_NAMES}
      --logo-file F   use a custom ASCII art file
      --accent COLOR  accent color, e.g. #ff0000
      --color MODE    color mode: auto, always, or never
      --no-color      alias for --color never
      --theme NAME    theme: default, mono, or neon
      --compact       tighter visual layout
      --ascii         ASCII-only bars and ellipsis
      --width N       output width in columns
      --oneline       one compact line for status bars
      --list-logos    list available logo names
  -o, --only FIELDS   show only these fields, comma separated
      --json          print system info as JSON

Fields:
  {" ".join(FIELDS)}

Examples:
  nullfetch --only cpu,ram
  nullfetch --logo arch
  nullfetch --oneline --only ram,disk
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
    names = []
    if shutil.which("lspci"):
        try:
            out = subprocess.run(
                ["lspci"], capture_output=True, text=True, timeout=5
            ).stdout
        except (OSError, subprocess.SubprocessError):
            out = ""
        for line in out.splitlines():
            if "VGA compatible controller" in line or "3D controller" in line:
                name = line.split(": ", 1)[-1].split(" (rev")[0]
                if name not in names:
                    names.append(name)
    if not names:
        for path in sorted(Path("/sys/class/drm").glob("card*/device")):
            for line in read(path / "uevent").splitlines():
                if line.startswith("DRIVER="):
                    driver = line.split("=", 1)[1]
                    if driver not in names:
                        names.append(driver)
                    break
    return ", ".join(names) or "?"


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

    total = round(total_kb / 1024 / 1024, 1)
    used = round((total_kb - available_kb) / 1024 / 1024, 1)
    return size_value(used, total)


def get_disk():
    try:
        st = os.statvfs("/")
    except OSError:
        return "?"

    gb = 1024**3
    total = st.f_blocks * st.f_frsize
    free = st.f_bavail * st.f_frsize
    return size_value(round((total - free) / gb, 1), round(total / gb, 1))


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
    return bool(sys.stdout.isatty()) and not os.environ.get("NO_COLOR")


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


# --- logos ----------------------------------------------------------------


def logo_lines(name):
    if name == "none":
        return []

    if name != "auto":
        if name not in LOGOS:
            available = LOGO_NAMES or "none"
            fail(f"unknown logo: {name} (available: {available})")
        return LOGOS[name]

    release = os_release()
    names = [release.get("ID", "").lower()]
    names += release.get("ID_LIKE", "").lower().split()
    for key in names:
        if key in LOGOS:
            return LOGOS[key]

    return LOGOS.get("linux", ART.splitlines())


# --- output ---------------------------------------------------------------


def format_size(
    value, use_color, accent=None, ascii_mode=False, max_width=None
):
    text = f"{value['used']} / {value['total']} GiB"
    percent = value["percent"]
    bar_width = BAR_WIDTH
    if max_width is not None:
        available = max_width - len(text) - 8
        bar_width = max(3, min(BAR_WIDTH, available))
    filled = round(percent / 100 * bar_width)
    empty = bar_width - filled

    full, blank = ("#", "-") if ascii_mode else ("█", "░")

    if use_color:
        bar = f"{fg(accent or distro_color())}{full * filled}{RESET}{blank * empty}"
    else:
        bar = full * filled + blank * empty

    result = f"{text:<16}{bar} {percent:>3}%"
    if max_width is not None and len(result) > max_width:
        compact = f"{value['used']}/{value['total']} GiB {bar} {percent}%"
        result = compact[:max_width]
    return result


def info_lines(data, use_color, max_width=None, accent=None, ascii_mode=False):
    lines = []
    for key, value in data.items():
        label = LABELS[key]

        if isinstance(value, dict):
            room = max_width - len(label) - 2 if max_width else None
            text = format_size(value, use_color, accent, ascii_mode, room)
        else:
            text = value
            if max_width:
                room = max_width - len(label) - 2
                if room > 1 and len(text) > room:
                    ellipsis = "..." if ascii_mode else "…"
                    text = text[: room - len(ellipsis)] + ellipsis

        if use_color:
            lines.append(f"{BOLD}{label}:{RESET} {text}")
        else:
            lines.append(f"{label}: {text}")
    return lines


def paint_art(art, use_color, width=0, accent=None, max_width=None):
    prefix = fg(accent or distro_color()) if use_color else ""
    suffix = RESET if use_color else ""

    lines = []
    for line in art:
        if max_width:
            line = line[:max_width]
        if width:
            line = line.ljust(width)
        lines.append(prefix + line + suffix)
    return lines


def render(
    data, art, use_color, width=None, accent=None, ascii_mode=False, compact=False
):
    if width is None:
        width = shutil.get_terminal_size((80, 24)).columns

    if not art:
        return "\n".join(info_lines(data, use_color, width, accent, ascii_mode))

    art_width = max(len(line) for line in art) + 4

    if width < art_width + 40:
        lines = info_lines(data, use_color, width, accent, ascii_mode)
        separator = [] if compact or ascii_mode else [""]
        return "\n".join(
            paint_art(art, use_color, accent=accent, max_width=width)
            + separator
            + lines
        )

    lines = info_lines(data, use_color, width - art_width, accent, ascii_mode)
    art = paint_art(art, use_color, art_width, accent)

    rows = []
    for left, right in zip_longest(art, lines, fillvalue=""):
        if not left:
            left = " " * art_width
        rows.append((left + right).rstrip())
    return "\n".join(rows)


def oneline(data, ascii_mode=False):
    parts = []
    if "user" in data and "host" in data:
        parts.append(f"{data['user']}@{data['host']}")

    for key, value in data.items():
        if key in ("user", "host"):
            if not ("user" in data and "host" in data):
                parts.append(str(value))
            continue
        if isinstance(value, dict):
            parts.append(
                f"{LABELS[key]} {value['used']}/{value['total']} GiB "
                f"{value['percent']}%"
            )
        else:
            parts.append(str(value))

    separator = " | " if ascii_mode else " · "
    return separator.join(parts)


def parse_color(text):
    if len(text) == 7 and text.startswith("#"):
        try:
            return tuple(int(text[i : i + 2], 16) for i in (1, 3, 5))
        except ValueError:
            pass
    fail(f"invalid color: {text} (expected #rrggbb)")


def read_logo_file(path):
    try:
        text = Path(path).read_text(encoding="utf-8", errors="replace")
    except OSError:
        fail(f"cannot read logo file: {path}")

    if len(text) > 100_000:
        fail("logo file is too large (maximum: 100 KiB)")
    lines = text.rstrip("\n").splitlines()
    if len(lines) > 100:
        fail("logo file has too many lines (maximum: 100)")
    if any(len(line) > 2_000 for line in lines):
        fail("logo file has an overlong line (maximum: 2000 characters)")
    if not lines:
        fail(f"logo file is empty: {path}")
    return lines


# --- command line ---------------------------------------------------------


def fail(message):
    print(f"nullfetch: {message}", file=sys.stderr)
    print("try 'nullfetch --help'", file=sys.stderr)
    sys.exit(2)


def parse_width(text):
    try:
        width = int(text)
    except ValueError:
        fail(f"invalid width: {text}")
    if width < 20:
        fail(f"width must be at least 20: {text}")
    return width


def parse_args(argv):
    opts = {
        "help": False,
        "version": False,
        "logo": True,
        "logo_name": "auto",
        "logo_file": None,
        "accent": None,
        "color": color_enabled(),
        "color_mode": "auto",
        "theme": "default",
        "compact": False,
        "ascii": False,
        "width": None,
        "oneline": False,
        "list_logos": False,
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
        elif arg == "--logo":
            i += 1
            if i == len(argv):
                fail(f"{arg} needs a value")
            opts["logo_name"] = argv[i]
        elif arg.startswith("--logo="):
            opts["logo_name"] = arg.split("=", 1)[1]
        elif arg == "--logo-file":
            i += 1
            if i == len(argv):
                fail(f"{arg} needs a value")
            opts["logo_file"] = argv[i]
        elif arg.startswith("--logo-file="):
            opts["logo_file"] = arg.split("=", 1)[1]
        elif arg == "--accent":
            i += 1
            if i == len(argv):
                fail(f"{arg} needs a value")
            opts["accent"] = argv[i]
        elif arg.startswith("--accent="):
            opts["accent"] = arg.split("=", 1)[1]
        elif arg == "--ascii":
            opts["ascii"] = True
        elif arg == "--width":
            i += 1
            if i == len(argv):
                fail(f"{arg} needs a value")
            opts["width"] = parse_width(argv[i])
        elif arg.startswith("--width="):
            opts["width"] = parse_width(arg.split("=", 1)[1])
        elif arg == "--oneline":
            opts["oneline"] = True
        elif arg == "--compact":
            opts["compact"] = True
        elif arg == "--list-logos":
            opts["list_logos"] = True
        elif arg == "--no-color":
            opts["color"] = False
            opts["color_mode"] = "never"
        elif arg == "--color":
            i += 1
            if i == len(argv):
                fail(f"{arg} needs a value")
            mode = argv[i]
            if mode not in ("auto", "always", "never"):
                fail(f"invalid color mode: {mode}")
            opts["color_mode"] = mode
            opts["color"] = mode == "always" or (
                mode == "auto" and color_enabled()
            )
        elif arg.startswith("--color="):
            mode = arg.split("=", 1)[1]
            if mode not in ("auto", "always", "never"):
                fail(f"invalid color mode: {mode}")
            opts["color_mode"] = mode
            opts["color"] = mode == "always" or (
                mode == "auto" and color_enabled()
            )
        elif arg == "--theme":
            i += 1
            if i == len(argv):
                fail(f"{arg} needs a value")
            opts["theme"] = argv[i]
        elif arg.startswith("--theme="):
            opts["theme"] = arg.split("=", 1)[1]
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

    if opts["theme"] not in ("default", "mono", "neon"):
        fail(f"unknown theme: {opts['theme']} (use default, mono, or neon)")

    if opts["help"]:
        print(HELP)
        return 0

    if opts["version"]:
        print(f"nullfetch {VERSION}")
        return 0

    if opts["list_logos"]:
        print("\n".join(sorted(LOGOS)))
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
    elif opts["oneline"]:
        fields = ["user", "host", "os", "kernel", "ram", "disk"]

    data = collect(fields)

    if opts["json"]:
        print(json.dumps(data, indent=2, ensure_ascii=False))
        return 0

    if opts["oneline"]:
        print(oneline(data, opts["ascii"]))
        return 0

    accent = parse_color(opts["accent"]) if opts["accent"] else None
    if opts["theme"] == "mono":
        opts["color"] = False
    elif opts["theme"] == "neon" and accent is None:
        accent = (255, 80, 220)

    if not opts["logo"]:
        art = []
    elif opts["logo_file"]:
        art = read_logo_file(opts["logo_file"])
    else:
        art = logo_lines(opts["logo_name"])

    print(
        render(
            data,
            art,
            opts["color"],
            width=opts["width"] or (60 if opts["compact"] else None),
            accent=accent,
            ascii_mode=opts["ascii"],
            compact=opts["compact"],
        )
    )
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except BrokenPipeError:
        sys.exit(0)
