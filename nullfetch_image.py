#!/usr/bin/env python3
"""Image rendering for nullfetch: built-in PNG decoder and terminal protocols.

The module is dependency-free.  External converters (chafa, viu, jp2, img2txt,
timg) are used only when available and only when they produce a better result
than the built-in renderer.
"""

from __future__ import annotations

import base64
import hashlib
import json
import os
import re
import shutil
import struct
import subprocess
import zlib
from pathlib import Path

PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"
JPEG_SIGNATURE = b"\xff\xd8\xff"
GIF_SIGNATURE = b"GIF8"
WEBP_SIGNATURE = b"RIFF"

MAX_IMAGE_BYTES = 32 * 1024 * 1024
MAX_IMAGE_PIXELS = 64 * 1024 * 1024
MAX_EXTERNAL_OUTPUT = 4 * 1024 * 1024
EXTERNAL_TIMEOUT = 5

RAMP = "@%#*+=-:. "
RESET = "\033[0m"
ANSI_PATTERN = re.compile(
    r"\033\][^\007\033]*(?:\007|\033\\)|"
    r"\033_[^\007\033]*\007|"
    r"\033\[[0-9;?]*[ -/]*[@-~]"
)


class ImageError(Exception):
    """Raised when an image cannot be loaded or rendered."""


def _sample_type(data: bytes) -> str:
    if data.startswith(PNG_SIGNATURE):
        return "png"
    if data.startswith(JPEG_SIGNATURE):
        return "jpeg"
    if data.startswith(GIF_SIGNATURE):
        return "gif"
    if data.startswith(WEBP_SIGNATURE) and data[8:12] == b"WEBP":
        return "webp"
    return "unknown"


def read_file(path) -> bytes:
    """Read an image file with sane size limits."""
    file = Path(path).expanduser()
    if not file.is_file():
        raise ImageError(f"image file not found: {file}")
    size = file.stat().st_size
    if size == 0:
        raise ImageError(f"image file is empty: {file}")
    if size > MAX_IMAGE_BYTES:
        raise ImageError("image file is too large (maximum: 32 MiB)")
    try:
        return file.read_bytes()
    except OSError as error:
        raise ImageError(f"cannot read image: {error}") from error


def _paeth(a: int, b: int, c: int) -> int:
    p = a + b - c
    pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
    if pa <= pb and pa <= pc:
        return a
    return b if pb <= pc else c


def _samples(row: bytes, width: int, channels: int, depth: int) -> list[int]:
    if depth == 8:
        return list(row)
    if depth == 16:
        return list(row[::2])
    mask = (1 << depth) - 1
    per_byte = 8 // depth
    return [
        (row[i // per_byte] >> (8 - depth - (i % per_byte) * depth)) & mask
        for i in range(width * channels)
    ]


def decode_png(data: bytes):
    """Decode a PNG into (width, height, rows) of packed RGBA bytes.

    Supports greyscale, RGB, palette and alpha images with 1, 2, 4, 8 or 16
    bits per sample.  Interlaced images are rejected; external converters can
    still display them.
    """
    if not data.startswith(PNG_SIGNATURE):
        raise ImageError("not a PNG image")

    pos = 8
    header = None
    palette = b""
    transparency = b""
    compressed = []
    while pos + 12 <= len(data):
        length = struct.unpack_from(">I", data, pos)[0]
        kind = data[pos + 4 : pos + 8]
        if length > len(data) - pos - 12:
            raise ImageError("truncated PNG chunk")
        chunk = data[pos + 8 : pos + 8 + length]
        pos += length + 12
        if kind == b"IHDR":
            header = struct.unpack(">IIBBBBB", chunk)
        elif kind == b"PLTE":
            palette = chunk
        elif kind == b"tRNS":
            transparency = chunk
        elif kind == b"IDAT":
            compressed.append(chunk)
        elif kind == b"IEND":
            break

    if header is None or not compressed:
        raise ImageError("invalid PNG image")
    width, height, depth, color, compression, filtering, interlace = header
    if not width or not height or width * height > MAX_IMAGE_PIXELS:
        raise ImageError("image dimensions are too large")
    if compression or filtering or interlace:
        raise ImageError("unsupported PNG encoding (interlaced or compressed)")
    if depth not in (1, 2, 4, 8, 16):
        raise ImageError("unsupported PNG bit depth")
    if color not in (0, 2, 3, 4, 6):
        raise ImageError("unsupported PNG color type")
    if color == 3 and not palette:
        raise ImageError("PNG palette is missing")

    channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[color]
    stride = (width * channels * depth + 7) // 8
    pixel_bytes = max(1, channels * depth // 8)

    try:
        decompressor = zlib.decompressobj()
        raw = decompressor.decompress(b"".join(compressed), MAX_IMAGE_PIXELS * 4)
    except zlib.error as error:
        raise ImageError(f"cannot decompress PNG: {error}") from error

    rows = []
    previous = bytearray(stride)
    pos = 0
    for _ in range(height):
        if pos + stride + 1 > len(raw):
            raise ImageError("truncated PNG image data")
        filter_type = raw[pos]
        pos += 1
        row = bytearray(raw[pos : pos + stride])
        pos += stride
        if filter_type == 1:
            for i in range(pixel_bytes, stride):
                row[i] = (row[i] + row[i - pixel_bytes]) & 255
        elif filter_type == 2:
            for i in range(stride):
                row[i] = (row[i] + previous[i]) & 255
        elif filter_type == 3:
            for i in range(stride):
                left = row[i - pixel_bytes] if i >= pixel_bytes else 0
                row[i] = (row[i] + ((left + previous[i]) >> 1)) & 255
        elif filter_type == 4:
            for i in range(stride):
                left = row[i - pixel_bytes] if i >= pixel_bytes else 0
                up_left = previous[i - pixel_bytes] if i >= pixel_bytes else 0
                row[i] = (row[i] + _paeth(left, previous[i], up_left)) & 255
        elif filter_type != 0:
            raise ImageError("unknown PNG filter type")
        rows.append(row)
        previous = row

    maximum = (1 << depth) - 1
    output = []
    for row in rows:
        values = _samples(row, width, channels, depth)
        pixels = bytearray(width * 4)
        for x in range(width):
            sample = values[x * channels : (x + 1) * channels]
            if color == 0:
                gray = sample[0] * 255 // maximum
                alpha = 255
                if (
                    len(transparency) >= 2
                    and int.from_bytes(transparency[:2], "big") == sample[0]
                ):
                    alpha = 0
                rgba = (gray, gray, gray, alpha)
            elif color == 2:
                red, green, blue = (value * 255 // maximum for value in sample[:3])
                alpha = 255
                if len(transparency) >= 6 and sample[:3] == list(
                    transparency[i] * 255 // maximum for i in (0, 2, 4)
                ):
                    alpha = 0
                rgba = (red, green, blue, alpha)
            elif color == 3:
                index = sample[0]
                if index * 3 + 2 >= len(palette):
                    raise ImageError("invalid PNG palette index")
                rgba = (
                    palette[index * 3],
                    palette[index * 3 + 1],
                    palette[index * 3 + 2],
                    transparency[index] if index < len(transparency) else 255,
                )
            elif color == 4:
                gray = sample[0] * 255 // maximum
                rgba = (gray, gray, gray, sample[1] * 255 // maximum)
            else:
                rgba = tuple(value * 255 // maximum for value in sample[:4])
            pixels[x * 4 : x * 4 + 4] = bytes(rgba)
        output.append(pixels)
    return width, height, output


def resize_rgba(rows, width: int, height: int, columns: int, lines: int):
    """Box-filter a RGBA image down to a terminal sized grid."""
    columns = max(1, columns)
    lines = max(1, lines)
    result = []
    for line in range(lines):
        top = line * height // lines
        bottom = max(top + 1, (line + 1) * height // lines)
        out_row = bytearray(columns * 4)
        for column in range(columns):
            left = column * width // columns
            right = max(left + 1, (column + 1) * width // columns)
            total = [0, 0, 0, 0]
            samples = 0
            for y in range(top, min(bottom, height), max(1, (bottom - top) // 3)):
                source = rows[y]
                for x in range(left, min(right, width), max(1, (right - left) // 3)):
                    pixel = source[x * 4 : x * 4 + 4]
                    total[0] += pixel[0]
                    total[1] += pixel[1]
                    total[2] += pixel[2]
                    total[3] += pixel[3]
                    samples += 1
            if samples:
                out_row[column * 4 : column * 4 + 4] = bytes(
                    value // samples for value in total
                )
        result.append(out_row)
    return result


def _color_code(rgb, truecolor: bool, background: bool = False) -> str:
    red, green, blue = rgb
    if truecolor:
        kind = 48 if background else 38
        return f"\033[{kind};2;{red};{green};{blue}m"
    if red == green == blue:
        if red < 8:
            index = 16
        elif red > 248:
            index = 231
        else:
            index = 232 + round((red - 8) / 247 * 24)
    else:
        index = (
            16
            + 36 * round(red / 255 * 5)
            + 6 * round(green / 255 * 5)
            + round(blue / 255 * 5)
        )
    kind = 48 if background else 38
    return f"\033[{kind};5;{index}m"


def _blend(pixel, background):
    alpha = pixel[3] / 255
    return tuple(
        round(pixel[channel] * alpha + background[channel] * (1 - alpha))
        for channel in range(3)
    )


def render_blocks(
    rows, columns: int, line_count: int, truecolor=True, background=(0, 0, 0)
):
    """Render using half-block characters, two pixel rows per terminal line."""
    output = []
    for line in range(line_count):
        top_row = rows[2 * line] if 2 * line < len(rows) else rows[-1]
        bottom_row = rows[2 * line + 1] if 2 * line + 1 < len(rows) else top_row
        pieces = []
        column = 0
        while column < columns:
            upper = _blend(top_row[column * 4 : column * 4 + 4], background)
            lower = _blend(bottom_row[column * 4 : column * 4 + 4], background)
            end = column + 1
            while end < columns:
                next_upper = _blend(top_row[end * 4 : end * 4 + 4], background)
                next_lower = _blend(bottom_row[end * 4 : end * 4 + 4], background)
                if (next_upper, next_lower) != (upper, lower):
                    break
                end += 1
            pieces.append(
                _color_code(upper, truecolor)
                + _color_code(lower, truecolor, background=True)
                + "\u2580" * (end - column)
            )
            column = end
        output.append("".join(pieces) + RESET)
    return output


def render_ascii(rows, columns: int, line_count: int, use_color=False, truecolor=True):
    """Render using an ASCII brightness ramp."""
    output = []
    for line in range(line_count):
        row = rows[line] if line < len(rows) else rows[-1]
        pieces = []
        column = 0
        while column < columns:
            pixel = row[column * 4 : column * 4 + 4]
            luminance = (299 * pixel[0] + 587 * pixel[1] + 114 * pixel[2]) // 1000
            index = luminance * (len(RAMP) - 1) // 255
            character = RAMP[index]
            color = tuple(pixel[:3])
            end = column + 1
            while end < columns:
                other = row[end * 4 : end * 4 + 4]
                other_luma = (299 * other[0] + 587 * other[1] + 114 * other[2]) // 1000
                other_index = other_luma * (len(RAMP) - 1) // 255
                if RAMP[other_index] != character:
                    break
                if use_color and tuple(other[:3]) != color:
                    break
                end += 1
            prefix = _color_code(color, truecolor) if use_color else ""
            pieces.append(prefix + character * (end - column))
            column = end
        output.append("".join(pieces) + (RESET if use_color else ""))
    return output


def terminal_features() -> dict:
    term = os.environ.get("TERM", "")
    program = os.environ.get("TERM_PROGRAM", "")
    return {
        "kitty": bool(os.environ.get("KITTY_WINDOW_ID")) or "kitty" in term.lower(),
        "iterm": bool(os.environ.get("ITERM_SESSION_ID"))
        or program.lower() in ("iterm.app", "wezterm", "kitty"),
        "sixel": "sixel" in term.lower(),
        "truecolor": "truecolor" in os.environ.get("COLORTERM", "").lower()
        or "24bit" in os.environ.get("COLORTERM", "").lower()
        or "truecolor" in term.lower()
        or "direct" in term.lower(),
        "converters": {
            name: bool(shutil.which(name))
            for name in ("chafa", "viu", "jp2", "img2txt", "timg")
        },
    }


def _kitty_payload(data: bytes, columns: int, lines: int) -> str:
    encoded = base64.b64encode(data).decode("ascii")
    header = f"a=T,f=100,C=1,r={lines},c={columns},"
    chunks = [encoded[i : i + 4092] for i in range(0, len(encoded), 4092)]
    parts = []
    for index, chunk in enumerate(chunks):
        more = "1" if index < len(chunks) - 1 else "0"
        parts.append(f"\033_G{header}m={more};{chunk}\033\\")
    return "".join(parts)


def _iterm_payload(data: bytes, name: str, columns: int, lines: int) -> str:
    encoded = base64.b64encode(data).decode("ascii")
    encoded_name = base64.b64encode(name.encode("utf-8", "replace")).decode("ascii")
    return (
        "\033]1337;File=inline=1;name="
        + encoded_name
        + f";width={columns}columns;height={lines}lines;preserveAspectRatio=1:"
        + encoded
        + "\007"
    )


def _external_commands(
    converter: str, path: str, columns: int, lines: int
) -> list[str]:
    if converter == "chafa":
        return [
            "chafa",
            f"--size={columns}x{lines}",
            "--colors=full",
            "--format=symbols",
            path,
        ]
    if converter == "viu":
        return ["viu", "-w", str(columns), "-h", str(lines), path]
    if converter == "jp2":
        return ["jp2", "-w", str(columns), path]
    if converter == "img2txt":
        return ["img2txt", "-W", str(columns), "-H", str(lines), "-f", "utf8", path]
    if converter == "timg":
        return ["timg", f"-g={columns}x{lines}", path]
    raise ImageError(f"unknown image converter: {converter}")


def _external_render(converter: str, path: str, columns: int, lines: int) -> list[str]:
    command = _external_commands(converter, path, columns, lines)
    try:
        result = subprocess.run(
            command,
            capture_output=True,
            timeout=EXTERNAL_TIMEOUT,
            check=False,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise ImageError(f"{converter} failed: {error}") from error
    if result.returncode != 0:
        raise ImageError(f"{converter} exited with status {result.returncode}")
    if len(result.stdout) > MAX_EXTERNAL_OUTPUT:
        raise ImageError(f"{converter} produced too much output")
    text = result.stdout.decode("utf-8", "replace").rstrip("\n")
    return text.splitlines() or [""]


def _cache_path(cache_dir, key: str) -> Path:
    return Path(cache_dir) / f"image-{key}.json"


def _cache_key(path, mode, columns, lines, truecolor, use_color, background) -> str:
    file = Path(path).expanduser()
    try:
        stat = file.stat()
        identity = f"{file}:{stat.st_size}:{stat.st_mtime_ns}"
    except OSError:
        identity = str(file)
    payload = (
        f"{identity}|{mode}|{columns}x{lines}|{truecolor}|{use_color}|{background}"
    )
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()[:24]


def _read_cache(cache_dir, key):
    if not cache_dir:
        return None
    try:
        data = json.loads(_cache_path(cache_dir, key).read_text(encoding="utf-8"))
        if isinstance(data, dict) and isinstance(data.get("lines"), list):
            return data
    except (OSError, ValueError):
        pass
    return None


def _write_cache(cache_dir, key, data):
    if not cache_dir:
        return
    try:
        directory = Path(cache_dir)
        directory.mkdir(parents=True, exist_ok=True)
        temporary = directory / f".image-{key}.tmp"
        temporary.write_text(json.dumps(data), encoding="utf-8")
        os.replace(temporary, _cache_path(directory, key))
    except OSError:
        pass


def _candidates(mode: str, kind: str, features: dict):
    converters = features["converters"]
    if mode != "auto":
        return [mode]
    candidates = []
    if kind == "png":
        if features["kitty"]:
            candidates.append("kitty")
        if features["iterm"]:
            candidates.append("iterm")
        candidates.append("blocks")
    else:
        if features["iterm"]:
            candidates.append("iterm")
        for name in ("chafa", "viu", "jp2", "timg", "img2txt"):
            if converters.get(name):
                candidates.append(name)
        if kind == "png":
            candidates.append("blocks")
    if features["sixel"] and converters.get("chafa"):
        candidates.insert(0, "chafa")
    return candidates


def render_image(
    path,
    *,
    mode="auto",
    columns=40,
    lines=20,
    truecolor=True,
    use_color=True,
    background=(0, 0, 0),
    cache_dir=None,
    allow_external=True,
    name=None,
):
    """Render an image as terminal lines.

    Returns a ``(lines, mode_used)`` tuple.  ``mode`` may be ``auto``,
    ``blocks``, ``ascii``, ``kitty``, ``iterm``, ``chafa``, ``viu``, ``jp2``,
    ``img2txt`` or ``timg``.
    """
    valid_modes = {
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
    }
    if mode not in valid_modes:
        raise ImageError(
            f"unknown image mode: {mode} "
            "(auto, blocks, ascii, kitty, iterm, chafa, viu, jp2, img2txt, timg)"
        )
    columns = max(1, min(columns, 1000))
    lines = max(1, min(lines, 500))
    features = terminal_features()
    if not allow_external and mode not in ("auto", "blocks", "ascii", "kitty", "iterm"):
        raise ImageError(f"external image converter disabled: {mode}")

    file = Path(path).expanduser()
    data = read_file(file)
    kind = _sample_type(data)
    candidates = _candidates("chafa" if mode == "sixel" else mode, kind, features)
    errors = []

    for candidate in candidates:
        if candidate in ("kitty", "iterm"):
            if candidate == "kitty" and kind != "png":
                errors.append("kitty protocol requires a PNG image")
                continue
            if candidate == "iterm" and kind == "unknown":
                errors.append("iTerm2 protocol does not recognize this format")
                continue
            payload = (
                _kitty_payload(data, columns, lines)
                if candidate == "kitty"
                else _iterm_payload(data, name or file.name, columns, lines)
            )
            return [payload], candidate

        if candidate in ("chafa", "viu", "jp2", "img2txt", "timg"):
            if not allow_external or not features["converters"].get(candidate):
                errors.append(f"{candidate} is not available")
                continue
            key = _cache_key(
                file, candidate, columns, lines, truecolor, use_color, background
            )
            cached = _read_cache(cache_dir, key)
            if cached is not None and cached.get("mode") == candidate:
                return cached["lines"], candidate
            try:
                rendered = _external_render(candidate, str(file), columns, lines)
            except ImageError as error:
                errors.append(str(error))
                continue
            if not use_color:
                rendered = [ANSI_PATTERN.sub("", line) for line in rendered]
            _write_cache(
                cache_dir,
                key,
                {"mode": candidate, "lines": rendered},
            )
            return rendered, candidate

        if candidate in ("blocks", "ascii"):
            if kind != "png":
                errors.append("built-in rendering requires a PNG image")
                continue
            key = _cache_key(
                file, candidate, columns, lines, truecolor, use_color, background
            )
            cached = _read_cache(cache_dir, key)
            if cached is not None and cached.get("mode") == candidate:
                return cached["lines"], candidate
            try:
                width, height, source = decode_png(data)
                resized = resize_rgba(source, width, height, columns, lines * 2)
                if candidate == "blocks":
                    rendered = render_blocks(
                        resized, columns, lines, truecolor, background
                    )
                else:
                    rendered = render_ascii(
                        resized, columns, lines, use_color, truecolor
                    )
            except ImageError as error:
                errors.append(str(error))
                continue
            _write_cache(
                cache_dir,
                key,
                {"mode": candidate, "lines": rendered},
            )
            return rendered, candidate

    raise ImageError("; ".join(errors) if errors else "cannot render image")
