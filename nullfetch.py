#!/usr/bin/env python3
"""nullfetch - a tiny system fetch for Linux."""

import getpass
import importlib.util
import io
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time
from functools import lru_cache
from itertools import zip_longest
from pathlib import Path

VERSION = "2.0.0"

BOLD = "\033[1m"
RESET = "\033[0m"
BAR_WIDTH = 12

# The original fields stay the default so that --json and old scripts keep
# working exactly as before.  ALL_FIELDS adds the extended modules.
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

ALL_FIELDS = [
    "user",
    "host",
    "os",
    "kernel",
    "arch",
    "uptime",
    "cpu",
    "cpu_cores",
    "gpu",
    "ram",
    "swap",
    "disk",
    "disks",
    "network",
    "local_ip",
    "public_ip",
    "battery",
    "temperature",
    "resolution",
    "shell",
    "terminal",
    "de",
    "wm",
    "packages",
    "processes",
    "locale",
    "timezone",
    "date",
]
EXTRA_FIELDS = [name for name in ALL_FIELDS if name not in FIELDS]
SIZE_FIELDS = {"ram", "disk", "swap"}

LABELS = {
    "user": "USER",
    "host": "HOST",
    "os": "OS",
    "kernel": "kernel",
    "arch": "arch",
    "uptime": "uptime",
    "cpu": "CPU",
    "cpu_cores": "cores",
    "gpu": "GPU",
    "ram": "RAM",
    "swap": "swap",
    "disk": "disk",
    "disks": "disks",
    "network": "network",
    "local_ip": "local ip",
    "public_ip": "public ip",
    "battery": "battery",
    "temperature": "temp",
    "resolution": "resolution",
    "shell": "shell",
    "terminal": "terminal",
    "de": "DE",
    "wm": "WM",
    "packages": "packages",
    "processes": "processes",
    "locale": "locale",
    "timezone": "timezone",
    "date": "date",
}

BAR_STYLES = {
    "blocks": ("\u2588", "\u2591"),
    "ascii": ("#", "-"),
    "arrows": ("\u25b6", "\u2500"),
    "dots": ("\u25cf", "\u25cb"),
    "lines": ("\u2501", "\u2500"),
}

SLOW_FIELDS = {
    "public_ip": 3600,
    "packages": 300,
    "resolution": 120,
    "temperature": 30,
    "battery": 30,
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

NAMED_COLORS = {
    "black": (0, 0, 0),
    "red": (220, 50, 47),
    "green": (64, 160, 43),
    "yellow": (202, 169, 0),
    "blue": (38, 109, 211),
    "magenta": (163, 90, 190),
    "cyan": (17, 150, 155),
    "white": (255, 255, 255),
    "gray": (120, 120, 120),
    "grey": (120, 120, 120),
    "orange": (255, 136, 0),
    "pink": (255, 105, 180),
    "purple": (140, 80, 200),
    "tux": (30, 58, 138),
}

THEME_ACCENTS = {
    "light": (23, 78, 166),
    "neon": (255, 80, 220),
    "dark": (110, 180, 255),
    "nord": (143, 188, 187),
    "gruvbox": (215, 153, 70),
    "dracula": (189, 147, 249),
    "tokyo": (122, 162, 247),
    "solarized": (181, 137, 0),
    "matrix": (0, 255, 65),
}
THEME_NAMES = ("default", "mono") + tuple(THEME_ACCENTS)

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
THEME_LIST = ", ".join(THEME_NAMES)
IMAGE_MODES = (
    "auto",
    "blocks",
    "ascii",
    "kitty",
    "iterm",
    "chafa",
    "viu",
    "jp2",
    "img2txt",
    "timg",
    "sixel",
)

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
      --image FILE    show an image (PNG, JPEG, GIF, WebP) in the terminal
      --image-mode M  auto, blocks, ascii, kitty, iterm, chafa, viu, jp2,
                      img2txt, timg or sixel
      --image-width N terminal columns for the image
      --image-height N terminal lines for the image
      --image-position P  side, top or bottom
      --image-bg C    background color behind transparent pixels
      --accent COLOR  accent color: blue, tux, #ff0000, ...
      --color MODE    color mode: auto, always, or never
      --no-color      alias for --color never
      --theme NAME    theme: {THEME_LIST}
      --compact       tighter visual layout
      --all           show every available module
      --list-fields   list available info fields
      --list-image-modes  list image rendering modes
      --separator S   text between a label and its value (default ": ")
      --bar-width N   width of RAM/disk bars (default {BAR_WIDTH})
      --bar-style S   blocks, ascii, arrows, dots or lines
      --format F      text, json, csv, markdown or oneline
      --no-cache      ignore cached slow values
      --cache-ttl N   cache lifetime in seconds (default 300)
      --timing        print collection time to stderr
      --save-config   save appearance and modules as defaults
      --show-config   show saved defaults
      --reset-config  remove saved defaults
      --init-config   write an example config file
      --set K=V       change one value in the saved config
      --config FILE   use an alternate config file
      --ascii         ASCII-only bars and ellipsis
      --width N       output width in columns
      --oneline       one compact line for status bars
      --list-logos    list available logo names
  -o, --only FIELDS   show only these fields, comma separated
      --json          print system info as JSON

Fields:
  {" ".join(FIELDS)} (default)
  {" ".join(EXTRA_FIELDS)} (extra: --all or --only)

Examples:
  nullfetch --only cpu,ram
  nullfetch --all --compact
  nullfetch --logo arch
  nullfetch --image ~/wallpaper.png --image-mode blocks
  nullfetch --image photo.jpg --image-mode chafa --image-position top
  nullfetch --oneline --only ram,disk
  nullfetch --logo cat --accent '#ff8800' --save-config
  nullfetch --format markdown --only os,cpu,ram
  nullfetch --reset-config
  nullfetch --no-logo --json
"""


# --- reading system information -------------------------------------------


def read(path):
    try:
        return Path(path).read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def cache_dir():
    base = os.environ.get("XDG_CACHE_HOME")
    return (Path(base) if base else Path.home() / ".cache") / "nullfetch"


def cache_read(name, ttl):
    if not ttl:
        return None
    try:
        data = json.loads((cache_dir() / f"{name}.json").read_text(encoding="utf-8"))
        if time.time() - float(data["time"]) <= ttl:
            return data["value"]
    except (OSError, ValueError, KeyError, TypeError):
        pass
    return None


def cache_write(name, value):
    path = cache_dir() / f"{name}.json"
    temporary = None
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", dir=path.parent, delete=False
        ) as stream:
            temporary = Path(stream.name)
            json.dump({"time": time.time(), "value": value}, stream)
        os.replace(temporary, path)
    except (OSError, TypeError, ValueError):
        pass
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def command(argv, timeout=2):
    try:
        result = subprocess.run(
            argv,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )
        return result.stdout.strip()
    except (OSError, subprocess.SubprocessError):
        return ""


@lru_cache(maxsize=1)
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


def get_arch():
    return platform.machine() or "?"


def get_uptime():
    parts = read("/proc/uptime").split()
    if not parts:
        return "?"
    try:
        seconds = int(float(parts[0]))
    except ValueError:
        return "?"
    days, remainder = divmod(seconds, 86400)
    hours, remainder = divmod(remainder, 3600)
    output = []
    if days:
        output.append(f"{days}d")
    if hours:
        output.append(f"{hours}h")
    output.append(f"{remainder // 60}m")
    return " ".join(output)


def get_cpu_cores():
    return str(os.cpu_count() or "?")


def get_shell():
    return Path(os.environ.get("SHELL") or "?").name or "?"


def get_terminal():
    for key in ("TERM_PROGRAM", "TERMINAL_PROG", "TERM"):
        value = os.environ.get(key)
        if value:
            return value
    return read(f"/proc/{os.getppid()}/comm").strip() or "?"


def get_de():
    return (
        os.environ.get("XDG_CURRENT_DESKTOP")
        or os.environ.get("DESKTOP_SESSION")
        or "?"
    )


def get_wm():
    text = command(["wmctrl", "-m"], timeout=1)
    for line in text.splitlines():
        if line.lower().startswith("name:"):
            return line.split(":", 1)[1].strip() or "?"
    return os.environ.get("WM_NAME") or get_de()


def get_packages():
    managers = (
        ("pacman", ("pacman", "-Qq")),
        ("dpkg", ("dpkg-query", "-f", "${binary:Package}\\n", "-W")),
        ("apk", ("apk", "info")),
        ("rpm", ("rpm", "-qa")),
        ("flatpak", ("flatpak", "list", "--app")),
    )
    counts = []
    for name, argv in managers:
        if not shutil.which(argv[0]):
            continue
        output = command(argv, timeout=3)
        if output:
            counts.append(f"{name}: {len(output.splitlines())}")
    return ", ".join(counts) or "?"


def get_processes():
    try:
        return str(sum(entry.name.isdigit() for entry in os.scandir("/proc")))
    except OSError:
        return "?"


def get_battery():
    for path in sorted(Path("/sys/class/power_supply").glob("BAT*")):
        capacity = read(path / "capacity").strip()
        if not capacity:
            continue
        status = read(path / "status").strip()
        return f"{capacity}% ({status})" if status else f"{capacity}%"
    return "?"


def get_local_ip():
    import socket

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        sock.connect(("8.8.8.8", 80))
        return sock.getsockname()[0]
    except OSError:
        return "?"
    finally:
        sock.close()


def get_public_ip():
    cached = cache_read("public-ip", 3600)
    if cached is not None:
        return cached
    try:
        import urllib.request

        with urllib.request.urlopen("https://api.ipify.org", timeout=3) as response:
            value = response.read(64).decode("utf-8", "replace").strip()
    except Exception:
        return "?"
    if value:
        cache_write("public-ip", value)
    return value or "?"


def get_temperature():
    readings = []
    for zone in sorted(Path("/sys/class/thermal").glob("thermal_zone*")):
        text = read(zone / "temp").strip()
        if text.isdigit():
            readings.append((int(text) / 1000, read(zone / "type").strip() or "zone"))
    for chip in sorted(Path("/sys/class/hwmon").glob("hwmon*")):
        name = read(chip / "name").strip() or "sensor"
        for sensor in sorted(chip.glob("temp*_input")):
            text = read(sensor).strip()
            if text.isdigit():
                readings.append((int(text) / 1000, name))
    if not readings:
        return "?"
    value, name = max(readings)
    return f"{value:.0f}\u00b0C ({name})"


def get_resolution():
    output = command(["xrandr", "--current"], timeout=2)
    for line in output.splitlines():
        if " connected" not in line:
            continue
        match = re.search(r"(\d{3,5})x(\d{3,5})\+", line)
        if match:
            return f"{match.group(1)}x{match.group(2)}"
    output = command(["xdpyinfo"], timeout=2)
    match = re.search(r"dimensions:\s+(\d+x\d+)", output)
    return match.group(1) if match else "?"


def get_swap():
    values = {}
    for line in read("/proc/meminfo").splitlines():
        if ":" in line:
            key, value = line.split(":", 1)
            values[key] = value.split()
    total_kb = int(values.get("SwapTotal", [0])[0])
    if not total_kb:
        return "none"
    free_kb = int(values.get("SwapFree", [0])[0])
    return size_value(
        round((total_kb - free_kb) / 1024 / 1024, 1),
        round(total_kb / 1024 / 1024, 1),
    )


def get_disks():
    entries = []
    seen = set()
    for line in read("/proc/mounts").splitlines():
        parts = line.split()
        if len(parts) < 2 or not parts[0].startswith("/dev/"):
            continue
        mount = parts[1]
        if mount in seen or mount.startswith(("/proc", "/sys", "/dev")):
            continue
        try:
            st = os.statvfs(mount)
        except OSError:
            continue
        gb = 1024**3
        total = st.f_blocks * st.f_frsize
        free = st.f_bavail * st.f_frsize
        if not total:
            continue
        value = size_value(round((total - free) / gb, 1), round(total / gb, 1))
        entries.append(
            f"{mount}: {value['used']}/{value['total']} GiB ({value['percent']}%)"
        )
        seen.add(mount)
        if len(entries) == 5:
            break
    return ", ".join(entries) or "?"


def get_network():
    for line in read("/proc/net/route").splitlines()[1:]:
        parts = line.split()
        if len(parts) > 1 and parts[1] == "00000000":
            return f"{parts[0]} ({get_local_ip()})"
    return get_local_ip() or "?"


def get_locale():
    return os.environ.get("LANG") or os.environ.get("LC_ALL") or "?"


def get_timezone():
    if os.environ.get("TZ"):
        return os.environ["TZ"]
    try:
        target = str(Path("/etc/localtime").resolve())
        return target.split("/zoneinfo/")[-1] if "/zoneinfo/" in target else target
    except OSError:
        return "?"


def get_date():
    from datetime import datetime

    return datetime.now().strftime("%Y-%m-%d %H:%M")


GETTERS = {
    "user": get_user,
    "host": get_host,
    "os": get_os,
    "kernel": get_kernel,
    "arch": get_arch,
    "uptime": get_uptime,
    "cpu": get_cpu,
    "cpu_cores": get_cpu_cores,
    "gpu": get_gpu,
    "ram": get_ram,
    "swap": get_swap,
    "disk": get_disk,
    "disks": get_disks,
    "network": get_network,
    "local_ip": get_local_ip,
    "public_ip": get_public_ip,
    "battery": get_battery,
    "temperature": get_temperature,
    "resolution": get_resolution,
    "shell": get_shell,
    "terminal": get_terminal,
    "de": get_de,
    "wm": get_wm,
    "packages": get_packages,
    "processes": get_processes,
    "locale": get_locale,
    "timezone": get_timezone,
    "date": get_date,
}


def collect(fields, cache_ttl=300, use_cache=True):
    """Gather modules in parallel without importing heavyweight libraries."""

    def fetch(key):
        ttl = min(cache_ttl, SLOW_FIELDS.get(key, 0)) if use_cache else 0
        if ttl:
            cached = cache_read(key, ttl)
            if cached is not None:
                return cached
        try:
            value = GETTERS[key]()
        except Exception:
            return "?"
        if ttl and value != "?":
            cache_write(key, value)
        return value

    fields = list(dict.fromkeys(fields))
    if len(fields) < 2:
        return {key: fetch(key) for key in fields}

    data = {}
    lock = threading.Lock()
    workers_count = min(8, len(fields))
    groups = [[] for _ in range(workers_count)]
    for index, key in enumerate(fields):
        groups[index % workers_count].append(key)

    def worker(keys):
        for key in keys:
            value = fetch(key)
            with lock:
                data[key] = value

    threads = [
        threading.Thread(target=worker, args=(group,), daemon=True)
        for group in groups
        if group
    ]
    for thread in threads:
        thread.start()
    deadline = time.monotonic() + 10
    for thread in threads:
        thread.join(max(0.0, deadline - time.monotonic()))
    return {key: data.get(key, "?") for key in fields}


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


@lru_cache(maxsize=1)
def distro_color():
    release = os_release()
    names = [release.get("ID", "").lower()]
    names += release.get("ID_LIKE", "").lower().split()

    for name in names:
        if name in DISTRO_COLORS:
            return DISTRO_COLORS[name]
    return DEFAULT_COLOR


def truecolor_available():
    return os.environ.get("COLORTERM", "").lower() in ("truecolor", "24bit")


def fg(rgb):
    r, g, b = rgb
    if truecolor_available():
        return f"\033[38;2;{r};{g};{b}m"
    return f"\033[38;5;{rgb_to_256(r, g, b)}m"


def bg(rgb):
    r, g, b = rgb
    if truecolor_available():
        return f"\033[48;2;{r};{g};{b}m"
    return f"\033[48;5;{rgb_to_256(r, g, b)}m"


ANSI_PATTERN = re.compile(
    r"\033\][^\007\033]*(?:\007|\033\\)|"
    r"\033_[^\007\033]*\007|"
    r"\033\[[0-9;?]*[ -/]*[@-~]"
)


def visible_len(text):
    return len(ANSI_PATTERN.sub("", text))


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


def load_image_module():
    """Import the optional image renderer from the source or install tree."""
    try:
        import nullfetch_image

        return nullfetch_image
    except ImportError:
        pass

    here = Path(__file__).resolve().parent
    prefix = here.parent
    for path in (
        here / "nullfetch_image.py",
        prefix / "share" / "nullfetch" / "nullfetch_image.py",
        Path("/usr/share/nullfetch/nullfetch_image.py"),
        Path("/usr/local/share/nullfetch/nullfetch_image.py"),
    ):
        if not path.is_file():
            continue
        try:
            spec = importlib.util.spec_from_file_location("_nullfetch_image", path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            return module
        except (OSError, ImportError, SyntaxError):
            continue
    return None


# --- output ---------------------------------------------------------------


def format_size(
    value,
    use_color,
    accent=None,
    ascii_mode=False,
    max_width=None,
    bar_style="blocks",
    bar_width=BAR_WIDTH,
):
    text = f"{value['used']} / {value['total']} GiB"
    percent = value["percent"]
    width = bar_width
    if max_width is not None:
        available = max_width - len(text) - 8
        width = max(3, min(bar_width, available))
    filled = round(percent / 100 * width)
    empty = width - filled

    if ascii_mode:
        full, blank = "#", "-"
    else:
        full, blank = BAR_STYLES.get(bar_style, BAR_STYLES["blocks"])

    if use_color:
        bar = f"{fg(accent or distro_color())}{full * filled}{RESET}{blank * empty}"
    else:
        bar = full * filled + blank * empty

    result = f"{text:<16}{bar} {percent:>3}%"
    if max_width is not None and len(result) > max_width:
        compact = f"{value['used']}/{value['total']} GiB {bar} {percent}%"
        result = compact[:max_width]
    return result


def info_lines(
    data,
    use_color,
    max_width=None,
    accent=None,
    ascii_mode=False,
    separator=": ",
    bar_style="blocks",
    bar_width=BAR_WIDTH,
):
    lines = []
    label_separator = separator.rstrip()
    gap = separator[len(label_separator) :] or " "
    indent = len(label_separator) + len(gap)

    for key, value in data.items():
        label = LABELS.get(key, key)

        if isinstance(value, dict):
            room = max_width - len(label) - indent if max_width else None
            text = format_size(
                value, use_color, accent, ascii_mode, room, bar_style, bar_width
            )
        else:
            if isinstance(value, list):
                text = ", ".join(str(item) for item in value)
            else:
                text = str(value)
            if max_width:
                room = max_width - len(label) - indent
                if room > 1 and len(text) > room:
                    ellipsis = "..." if ascii_mode else "…"
                    text = text[: room - len(ellipsis)] + ellipsis

        if use_color:
            lines.append(f"{BOLD}{label}{label_separator}{RESET}{gap}{text}")
        else:
            lines.append(f"{label}{label_separator}{gap}{text}")
    return lines


def paint_art(art, use_color, width=0, accent=None, max_width=None, precolored=False):
    prefix = "" if precolored or not use_color else fg(accent or distro_color())
    suffix = "" if precolored or not use_color else RESET

    lines = []
    for line in art:
        length = visible_len(line) if precolored else len(line)
        if max_width and not precolored and length > max_width:
            line = line[:max_width]
            length = max_width
        if width and length < width:
            line += " " * (width - length)
        lines.append(prefix + line + suffix)
    return lines


def render(
    data,
    art,
    use_color,
    width=None,
    accent=None,
    ascii_mode=False,
    compact=False,
    separator=": ",
    bar_style="blocks",
    bar_width=BAR_WIDTH,
    precolored_art=False,
):
    if width is None:
        width = shutil.get_terminal_size((80, 24)).columns

    def make_lines(available):
        return info_lines(
            data,
            use_color,
            available,
            accent,
            ascii_mode,
            separator,
            bar_style,
            bar_width,
        )

    if not art:
        return "\n".join(make_lines(width))

    art_width = max(visible_len(line) for line in art) + (2 if precolored_art else 4)

    if width < art_width + 40:
        lines = make_lines(width)
        blank = [] if compact or ascii_mode else [""]
        return "\n".join(
            paint_art(
                art,
                use_color,
                accent=accent,
                max_width=width,
                precolored=precolored_art,
            )
            + blank
            + lines
        )

    lines = make_lines(width - art_width)
    colored_art = paint_art(
        art, use_color, art_width, accent, precolored=precolored_art
    )

    rows = []
    for left, right in zip_longest(colored_art, lines, fillvalue=""):
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
        elif isinstance(value, list):
            parts.append(", ".join(str(item) for item in value))
        else:
            parts.append(str(value))

    separator = " | " if ascii_mode else " · "
    return separator.join(parts)


def csv_output(data):
    import csv

    stream = io.StringIO()
    writer = csv.writer(stream, lineterminator="\n")
    writer.writerow(("field", "value"))
    for key, value in data.items():
        if isinstance(value, dict):
            text = f"{value['used']}/{value['total']} GiB ({value['percent']}%)"
        elif isinstance(value, list):
            text = "; ".join(str(item) for item in value)
        else:
            text = str(value)
        writer.writerow((key, text))
    return stream.getvalue().rstrip("\n")


def markdown_output(data):
    lines = ["| Field | Value |", "| :--- | :--- |"]
    for key, value in data.items():
        if isinstance(value, dict):
            text = f"{value['used']}/{value['total']} GiB ({value['percent']}%)"
        elif isinstance(value, list):
            text = "; ".join(str(item) for item in value)
        else:
            text = str(value).replace("|", "\\|")
        lines.append(f"| {LABELS.get(key, key)} | {text} |")
    return "\n".join(lines)


def parse_color(text):
    name = text.lower()
    if name in NAMED_COLORS:
        return NAMED_COLORS[name]
    if name in DISTRO_COLORS:
        return DISTRO_COLORS[name]
    if text.startswith("#"):
        value = text[1:]
        if len(value) == 3 and all(char in "0123456789abcdefABCDEF" for char in value):
            return tuple(int(char * 2, 16) for char in value)
        if len(value) == 6 and all(char in "0123456789abcdefABCDEF" for char in value):
            return tuple(int(value[i : i + 2], 16) for i in (0, 2, 4))
    fail(f"invalid color: {text} (expected #rrggbb or a color name)")


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


CONFIG_STRING_KEYS = {
    "logo",
    "logo-file",
    "accent",
    "theme",
    "color",
    "image",
    "image-mode",
    "image-position",
    "image-bg",
    "separator",
    "bar-style",
    "format",
}
CONFIG_INT_KEYS = {
    "image-width",
    "image-height",
    "bar-width",
    "cache-ttl",
}
CONFIG_BOOL_KEYS = {
    "ascii",
    "compact",
    "oneline",
    "all",
    "no-cache",
    "no-network",
    "timing",
}
CONFIG_KEYS = CONFIG_STRING_KEYS | CONFIG_INT_KEYS | CONFIG_BOOL_KEYS | {"modules"}
CONFIG_DEFAULTS = {
    "image_mode": "auto",
    "image_position": "side",
    "image_bg": None,
    "bar_style": "blocks",
    "image_width": None,
    "image_height": None,
    "bar_width": BAR_WIDTH,
    "cache_ttl": 300,
}
CONFIG_OVERRIDE = None


def config_path():
    if CONFIG_OVERRIDE is not None:
        return CONFIG_OVERRIDE
    base = os.environ.get("XDG_CONFIG_HOME")
    return (Path(base) if base else Path.home() / ".config") / "nullfetch/config.json"


def load_config(path=None):
    path = path or config_path()
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        return {}
    except (OSError, ValueError) as error:
        fail(f"cannot load config {path}: {error}; use --reset-config")
    if not isinstance(data, dict):
        fail(f"invalid config: {path}; use --reset-config")
    for key, value in data.items():
        if key not in CONFIG_KEYS:
            fail(f"invalid config: {path}; use --reset-config")
        if key == "modules":
            if not isinstance(value, list) or not all(
                isinstance(item, str) and item in ALL_FIELDS for item in value
            ):
                fail(f"invalid config: {path}; use --reset-config")
        elif key in CONFIG_INT_KEYS:
            if not isinstance(value, int) or isinstance(value, bool):
                fail(f"invalid config: {path}; use --reset-config")
        elif key in CONFIG_BOOL_KEYS:
            if not isinstance(value, bool):
                fail(f"invalid config: {path}; use --reset-config")
        elif not isinstance(value, str):
            fail(f"invalid config: {path}; use --reset-config")
    if "theme" in data and data["theme"] not in THEME_NAMES:
        fail(f"invalid config: {path}; use --reset-config")
    if "color" in data and data["color"] not in ("auto", "always", "never"):
        fail(f"invalid config: {path}; use --reset-config")
    return data


def config_args():
    args = []
    for key, value in load_config().items():
        if key == "modules":
            if value:
                args.append(f"--only={','.join(value)}")
        elif key in CONFIG_BOOL_KEYS:
            if value:
                args.append(f"--{key}")
        else:
            args.append(f"--{key}={value}")
    return args


def write_config(path, data):
    temporary = None
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", dir=path.parent, delete=False
        ) as stream:
            temporary = Path(stream.name)
            json.dump(data, stream, indent=2)
            stream.write("\n")
        os.replace(temporary, path)
    except OSError as error:
        fail(f"cannot save config: {error}")
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def show_config():
    path = config_path()
    if not path.exists():
        print(f"no saved defaults: {path}")
        return
    data = load_config(path)
    print(f"config: {path}")
    for key, value in data.items():
        if isinstance(value, list):
            value = ", ".join(value)
        elif isinstance(value, bool):
            value = str(value).lower()
        print(f"{key}: {value}")


def init_config():
    path = config_path()
    if path.exists():
        fail(f"config already exists: {path}; use --set or --reset-config")
    data = {
        "logo": "auto",
        "theme": "default",
        "color": "auto",
        "modules": list(FIELDS),
        "separator": ": ",
        "bar-style": "blocks",
        "bar-width": BAR_WIDTH,
    }
    write_config(path, data)
    print(f"example config written: {path}")


def set_config_value(text):
    if "=" not in text:
        fail(f"--set needs KEY=VALUE: {text}")
    key, value = text.split("=", 1)
    if key not in CONFIG_KEYS:
        fail(f"invalid config key: {key}")
    if key in CONFIG_INT_KEYS:
        value = parse_int(value, key, 0)
    elif key in CONFIG_BOOL_KEYS:
        if value.lower() not in ("true", "false", "yes", "no", "1", "0"):
            fail(f"invalid boolean for {key}: {value}")
        value = value.lower() in ("true", "yes", "1")
    elif key == "modules":
        value = [name.strip() for name in value.split(",") if name.strip()]
        unknown = [name for name in value if name not in ALL_FIELDS]
        if not value or unknown:
            fail(f"invalid modules: {text}")
    if key == "theme" and value not in THEME_NAMES:
        fail(f"unknown theme: {value}")
    if key == "color" and value not in ("auto", "always", "never"):
        fail(f"invalid color mode: {value}")
    if key == "accent":
        parse_color(value)
    path = config_path()
    data = load_config(path)
    data[key] = value
    write_config(path, data)
    print(f"set {key} = {value}")


def save_config(opts):
    data = {
        "logo": opts["logo_name"] if opts["logo"] else "none",
        "theme": opts["theme"],
        "color": opts["color_mode"],
    }
    if opts["accent"]:
        data["accent"] = opts["accent"]
    if opts["logo"] and opts["logo_file"]:
        data["logo-file"] = str(Path(opts["logo_file"]).expanduser().resolve())
    if opts["image"]:
        data["image"] = str(Path(opts["image"]).expanduser().resolve())
    for key, option in (
        ("image-mode", "image_mode"),
        ("image-position", "image_position"),
        ("image-bg", "image_bg"),
        ("bar-style", "bar_style"),
    ):
        if opts[option] != CONFIG_DEFAULTS[option]:
            data[key] = opts[option]
    for key, option in (
        ("image-width", "image_width"),
        ("image-height", "image_height"),
        ("bar-width", "bar_width"),
        ("cache-ttl", "cache_ttl"),
    ):
        if opts[option] != CONFIG_DEFAULTS[option]:
            data[key] = opts[option]
    for key in (
        "ascii",
        "compact",
        "oneline",
        "all",
        "no_cache",
        "no_network",
        "timing",
    ):
        if opts[key]:
            data[key.replace("_", "-")] = True
    if opts["separator"] != ": ":
        data["separator"] = opts["separator"]
    if opts["format"] != "text":
        data["format"] = opts["format"]
    if opts["only"] is not None:
        data["modules"] = [
            name.strip() for name in opts["only"].split(",") if name.strip()
        ]
    elif opts["all"]:
        data["modules"] = list(ALL_FIELDS)
    path = config_path()
    write_config(path, data)
    print(f"saved defaults: {path}")


def fail(message):
    print(f"nullfetch: {message}", file=sys.stderr)
    print("try 'nullfetch --help'", file=sys.stderr)
    sys.exit(2)


def parse_int(text, name, minimum=0):
    try:
        value = int(text)
    except ValueError:
        fail(f"invalid {name}: {text}")
    if value < minimum:
        fail(f"{name} must be at least {minimum}: {text}")
    return value


def parse_width(text):
    try:
        width = int(text)
    except ValueError:
        fail(f"invalid width: {text}")
    if width < 20:
        fail(f"width must be at least 20: {text}")
    return width


def parse_color_mode(text):
    if text not in ("auto", "always", "never"):
        fail(f"invalid color mode: {text}")
    return text


VALUE_OPTIONS = {
    "-o": "only",
    "--logo": "logo_name",
    "--logo-file": "logo_file",
    "--accent": "accent",
    "--width": "width",
    "--color": "color_mode",
    "--theme": "theme",
    "--only": "only",
    "--image": "image",
    "--image-file": "image",
    "--image-mode": "image_mode",
    "--image-width": "image_width",
    "--image-height": "image_height",
    "--image-position": "image_position",
    "--image-bg": "image_bg",
    "--separator": "separator",
    "--bar-width": "bar_width",
    "--bar-style": "bar_style",
    "--format": "format",
    "--cache-ttl": "cache_ttl",
    "--set": "set_value",
    "--config": "config_file",
}
FLAG_OPTIONS = {
    "-h": "help",
    "--help": "help",
    "--version": "version",
    "--save-config": "save_config",
    "--show-config": "show_config",
    "--reset-config": "reset_config",
    "--init-config": "init_config",
    "--ascii": "ascii",
    "--compact": "compact",
    "--oneline": "oneline",
    "--json": "json",
    "--list-logos": "list_logos",
    "--list-fields": "list_fields",
    "--list-image-modes": "list_image_modes",
    "--all": "all",
    "--no-cache": "no_cache",
    "--no-network": "no_network",
    "--timing": "timing",
}


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
        "list_fields": False,
        "list_image_modes": False,
        "json": False,
        "only": None,
        "all": False,
        "image": None,
        "image_mode": "auto",
        "image_width": None,
        "image_height": None,
        "image_position": "side",
        "image_bg": None,
        "separator": ": ",
        "bar_style": "blocks",
        "bar_width": BAR_WIDTH,
        "format": "text",
        "cache_ttl": 300,
        "no_cache": False,
        "no_network": False,
        "timing": False,
        "save_config": False,
        "show_config": False,
        "reset_config": False,
        "init_config": False,
        "set_value": None,
        "config_file": None,
    }

    def assign(key, value, option):
        if key == "width":
            opts[key] = parse_width(value)
        elif key == "color_mode":
            opts[key] = parse_color_mode(value)
            opts["color"] = value == "always" or (value == "auto" and color_enabled())
        elif key == "logo_name" or key == "logo_file":
            opts[key] = value
            opts["logo"] = True
            if key == "logo_name":
                opts["logo_file"] = None
        elif key in ("image_width", "image_height", "bar_width", "cache_ttl"):
            minimum = 0 if key == "cache_ttl" else 1
            opts[key] = parse_int(value, option[2:], minimum)
        elif key == "image_mode" and value not in IMAGE_MODES:
            fail(f"unknown image mode: {value}")
        elif key == "image_position" and value not in ("side", "top", "bottom"):
            fail(f"unknown image position: {value}")
        elif key == "bar_style" and value not in BAR_STYLES:
            fail(f"unknown bar style: {value}")
        elif key == "format" and value not in (
            "text",
            "json",
            "csv",
            "markdown",
            "oneline",
        ):
            fail(f"unknown format: {value}")
        else:
            opts[key] = value

    i = 0
    while i < len(argv):
        arg = argv[i]
        key = None

        for option, target in VALUE_OPTIONS.items():
            if arg == option:
                if i + 1 == len(argv):
                    fail(f"{arg} needs a value")
                value = argv[i + 1]
                i += 1
                key = target
                break
            if arg.startswith(option + "="):
                value = arg.split("=", 1)[1]
                key = target
                break
        if key is not None:
            if key == "config_file":
                global CONFIG_OVERRIDE
                CONFIG_OVERRIDE = Path(value).expanduser()
            assign(key, value, option)
            i += 1
            continue

        if arg == "--no-logo":
            opts["logo"] = False
        elif arg == "--no-color":
            opts["color"] = False
            opts["color_mode"] = "never"
        elif arg in FLAG_OPTIONS:
            opts[FLAG_OPTIONS[arg]] = True
        else:
            fail(f"unknown option: {arg}")
        i += 1

    return opts


def render_image_lines(opts, data):
    """Render the configured image; returns (lines, mode_used)."""
    module = load_image_module()
    if module is None:
        fail("image support unavailable: nullfetch_image.py is missing")

    terminal = shutil.get_terminal_size((80, 24))
    width = opts["width"] or terminal.columns
    position = opts["image_position"]
    columns = opts["image_width"] or (
        max(12, min(width // 2, 48)) if position == "side" else width
    )
    height = opts["image_height"] or max(4, min(terminal.lines - 2, len(data) + 2))

    mode = opts["image_mode"]
    if mode == "auto" and opts["ascii"]:
        mode = "ascii"
    if mode == "blocks" and not opts["color"]:
        mode = "ascii"

    background = parse_color(opts["image_bg"]) if opts["image_bg"] else (0, 0, 0)
    try:
        return module.render_image(
            opts["image"],
            mode=mode,
            columns=columns,
            lines=height,
            truecolor=truecolor_available(),
            use_color=opts["color"],
            background=background,
            cache_dir=None if opts["no_cache"] else cache_dir(),
        )
    except module.ImageError as error:
        fail(f"cannot render image: {error}")
    except Exception as error:
        fail(f"cannot render image: {error}")


def main(argv=None):
    global CONFIG_OVERRIDE
    CONFIG_OVERRIDE = None
    if argv is None:
        argv = sys.argv[1:]

    opts = parse_args(argv)

    if opts["reset_config"]:
        try:
            config_path().unlink(missing_ok=True)
        except OSError as error:
            fail(f"cannot reset config: {error}")
        print("saved defaults reset")
        return 0
    if opts["show_config"]:
        show_config()
        return 0
    if opts["init_config"]:
        init_config()
        return 0
    if opts["set_value"] is not None:
        set_config_value(opts["set_value"])
        return 0

    simple = any(
        opts[key]
        for key in ("help", "version", "list_logos", "list_fields", "list_image_modes")
    )
    if not simple:
        opts = parse_args(config_args() + list(argv))

    cli_only = any(
        item in ("--only", "-o") or item.startswith("--only=") for item in argv
    )
    cli_all = "--all" in argv
    if cli_only and not cli_all:
        opts["all"] = False
    if cli_all and not cli_only:
        opts["only"] = None

    if opts["theme"] not in THEME_NAMES:
        fail(f"unknown theme: {opts['theme']} (use {', '.join(THEME_NAMES)})")

    if opts["help"]:
        print(HELP)
        return 0

    if opts["version"]:
        print(f"nullfetch {VERSION}")
        return 0

    if opts["list_logos"]:
        print("\n".join(sorted(LOGOS)))
        return 0

    if opts["list_fields"]:
        print("\n".join(ALL_FIELDS))
        return 0

    if opts["list_image_modes"]:
        print("\n".join(IMAGE_MODES))
        return 0

    accent = parse_color(opts["accent"]) if opts["accent"] else None
    if opts["image_bg"]:
        parse_color(opts["image_bg"])

    if opts["save_config"]:
        if opts["logo"]:
            if opts["logo_file"]:
                read_logo_file(opts["logo_file"])
            else:
                logo_lines(opts["logo_name"])
        if opts["image"]:
            if not Path(opts["image"]).expanduser().is_file():
                fail(f"image file not found: {opts['image']}")
            if load_image_module() is None:
                fail("image support unavailable: nullfetch_image.py is missing")
        save_config(opts)
        return 0

    if opts["all"] and opts["only"] is not None:
        fail("--all and --only cannot be used together")
    if opts["all"]:
        fields = list(ALL_FIELDS)
    elif opts["only"] is not None:
        fields = [name.strip().lower() for name in opts["only"].split(",")]
        fields = [name for name in fields if name]
        if not fields:
            fail("--only needs at least one field")
        unknown = [name for name in fields if name not in ALL_FIELDS]
        if unknown:
            fail(f"unknown field: {', '.join(unknown)}")
    elif opts["oneline"] or opts["format"] == "oneline":
        fields = ["user", "host", "os", "kernel", "ram", "disk"]
    else:
        fields = list(FIELDS)

    if opts["no_network"]:
        fields = [
            name for name in fields if name not in ("public_ip", "local_ip", "network")
        ]

    started = time.perf_counter()
    data = collect(fields, cache_ttl=opts["cache_ttl"], use_cache=not opts["no_cache"])
    if opts["timing"]:
        elapsed = (time.perf_counter() - started) * 1000
        print(
            f"nullfetch: {len(fields)} fields in {elapsed:.1f} ms",
            file=sys.stderr,
        )

    output_format = opts["format"]
    if opts["json"]:
        output_format = "json"
    elif opts["oneline"] and output_format == "text":
        output_format = "oneline"

    if output_format == "json":
        print(json.dumps(data, indent=2, ensure_ascii=False))
        return 0
    if output_format == "csv":
        print(csv_output(data))
        return 0
    if output_format == "markdown":
        print(markdown_output(data))
        return 0
    if output_format == "oneline":
        print(oneline(data, opts["ascii"]))
        return 0

    if opts["theme"] == "mono":
        opts["color"] = False
    elif accent is None and opts["theme"] in THEME_ACCENTS:
        accent = THEME_ACCENTS[opts["theme"]]

    if not opts["logo"]:
        art = []
    elif opts["logo_file"]:
        art = read_logo_file(opts["logo_file"])
    else:
        art = logo_lines(opts["logo_name"])

    render_options = {
        "width": opts["width"] or (60 if opts["compact"] else None),
        "accent": accent,
        "ascii_mode": opts["ascii"],
        "compact": opts["compact"],
        "separator": opts["separator"],
        "bar_style": opts["bar_style"],
        "bar_width": opts["bar_width"],
    }

    if opts["image"]:
        image_lines, image_mode = render_image_lines(opts, data)
        position = opts["image_position"]
        if image_mode in ("kitty", "iterm"):
            print("\n".join(image_lines))
            print(render(data, [], opts["color"], **render_options))
        elif position == "top":
            print("\n".join(image_lines))
            print(render(data, art, opts["color"], **render_options))
        elif position == "bottom":
            print(render(data, art, opts["color"], **render_options))
            print("\n".join(image_lines))
        else:
            print(
                render(
                    data,
                    image_lines,
                    opts["color"],
                    precolored_art=True,
                    **render_options,
                )
            )
        return 0

    print(render(data, art, opts["color"], **render_options))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except BrokenPipeError:
        sys.exit(0)
