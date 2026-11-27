import os
import struct
import subprocess
import sys
import zlib
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "nullfetch.py"

sys.path.insert(0, str(ROOT))
import nullfetch_image as image  # noqa: E402

ENV = {key: value for key, value in os.environ.items() if key != "NO_COLOR"}
ENV["TERM"] = "xterm-256color"
ENV["COLORTERM"] = "truecolor"


def chunk(kind, data):
    return (
        struct.pack(">I", len(data))
        + kind
        + data
        + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)
    )


def png_bytes(width, height, depth, color, raw, extra=b""):
    header = struct.pack(">IIBBBBB", width, height, depth, color, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + extra
        + chunk(b"IDAT", zlib.compress(raw))
        + chunk(b"IEND", b"")
    )


def gradient_png(width=8, height=4):
    rows = []
    for y in range(height):
        row = bytearray([0])
        for x in range(width):
            row += bytes(
                (
                    x * 255 // max(1, width - 1),
                    y * 255 // max(1, height - 1),
                    100,
                    255,
                )
            )
        rows.append(bytes(row))
    return png_bytes(width, height, 8, 6, b"".join(rows))


@pytest.fixture
def png_file(tmp_path):
    path = tmp_path / "picture.png"
    path.write_bytes(gradient_png())
    return path


def run(*args):
    return subprocess.run(
        [sys.executable, str(SCRIPT), *args],
        capture_output=True,
        text=True,
        env=ENV,
    )


def test_decode_rgba_png():
    width, height, rows = image.decode_png(gradient_png(2, 1))
    assert (width, height) == (2, 1)
    assert rows[0][:4] == b"\x00\x00\x64\xff"
    assert rows[0][4:8] == b"\xff\x00\x64\xff"


def test_decode_palette_with_alpha():
    extra = chunk(b"PLTE", bytes((255, 0, 0, 0, 255, 0)))
    extra += chunk(b"tRNS", bytes((128, 255)))
    data = png_bytes(2, 1, 8, 3, b"\x00\x00\x01", extra)
    _, _, rows = image.decode_png(data)
    assert rows[0][:4] == b"\xff\x00\x00\x80"
    assert rows[0][4:8] == b"\x00\xff\x00\xff"


def test_decode_one_bit_gray():
    _, _, rows = image.decode_png(png_bytes(8, 1, 1, 0, b"\x00\xaa"))
    assert rows[0][:4] == b"\xff\xff\xff\xff"
    assert rows[0][4:8] == b"\x00\x00\x00\xff"


def test_invalid_png_is_rejected(tmp_path):
    path = tmp_path / "bad.png"
    path.write_bytes(b"not a png")
    with pytest.raises(image.ImageError):
        image.render_image(path, mode="blocks")


def test_render_modes(png_file):
    width, height, source = image.decode_png(png_file.read_bytes())
    resized = image.resize_rgba(source, width, height, 6, 2)
    blocks = image.render_blocks(resized, 6, 1, truecolor=True)
    assert "\u2580" in blocks[0]
    assert "\033[48;2;" in blocks[0]
    ascii_lines = image.render_ascii(resized, 6, 2, use_color=False)
    assert all(character in image.RAMP for line in ascii_lines for character in line)
    assert "\033" not in ascii_lines[0]


def test_render_image_uses_cache(tmp_path, png_file, monkeypatch):
    cache = tmp_path / "cache"
    lines, mode = image.render_image(
        png_file, mode="blocks", columns=8, lines=2, cache_dir=cache
    )
    assert mode == "blocks"
    assert len(lines) == 2
    assert list(cache.glob("image-*.json"))

    def forbidden(*args, **kwargs):
        raise AssertionError("cache was not used")

    monkeypatch.setattr(image, "decode_png", forbidden)
    cached, _ = image.render_image(
        png_file, mode="blocks", columns=8, lines=2, cache_dir=cache
    )
    assert cached == lines


def test_terminal_protocols():
    payload = image._kitty_payload(b"pngdata", 10, 5)
    assert payload.startswith("\033_Ga=T,f=100")
    assert "r=5,c=10" in payload
    assert payload.endswith("\033\\")
    inline = image._iterm_payload(b"pngdata", "cat.png", 10, 5)
    assert "\033]1337;File=inline=1" in inline
    assert "width=10columns" in inline


def test_unknown_mode(png_file):
    with pytest.raises(image.ImageError, match="unknown image mode"):
        image.render_image(png_file, mode="nope")


def test_external_output_strips_colors_when_disabled(png_file, monkeypatch):
    features = image.terminal_features()
    features["converters"] = {name: name == "chafa" for name in features["converters"]}
    features["iterm"] = False
    monkeypatch.setattr(image, "terminal_features", lambda: features)
    monkeypatch.setattr(image, "_external_render", lambda *args: ["\033[31mX\033[0m"])
    lines, mode = image.render_image(
        png_file,
        mode="chafa",
        columns=4,
        lines=1,
        use_color=False,
        cache_dir=None,
    )
    assert (lines, mode) == (["X"], "chafa")


def test_cli_ascii_image_has_no_escape(png_file):
    result = run(
        "--image",
        str(png_file),
        "--image-mode",
        "ascii",
        "--color",
        "never",
        "--no-logo",
        "--only",
        "user",
    )
    assert result.returncode == 0
    assert "\033" not in result.stdout
    assert "USER:" in result.stdout
    assert any(char in result.stdout for char in image.RAMP)


def test_cli_block_image_uses_half_blocks(png_file):
    result = run(
        "--image",
        str(png_file),
        "--image-mode",
        "blocks",
        "--color",
        "always",
        "--no-logo",
        "--only",
        "user",
    )
    assert result.returncode == 0
    assert "\u2580" in result.stdout
    assert "\033[38;2;" in result.stdout


def test_cli_unknown_image_mode(png_file):
    result = run("--image", str(png_file), "--image-mode", "nope")
    assert result.returncode == 2
    assert "unknown image mode" in result.stderr
