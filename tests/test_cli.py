import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "nullfetch.py"

sys.path.insert(0, str(ROOT))
import nullfetch  # noqa: E402

ENV = {key: value for key, value in os.environ.items() if key != "NO_COLOR"}
ENV["TERM"] = "xterm-256color"


def run(*args):
    return subprocess.run(
        [sys.executable, str(SCRIPT), *args],
        capture_output=True,
        text=True,
        env=ENV,
    )


def test_version():
    result = run("--version")
    assert result.returncode == 0
    assert result.stdout.strip() == f"nullfetch {nullfetch.VERSION}"


def test_help():
    result = run("--help")
    assert result.returncode == 0
    assert "--no-logo" in result.stdout
    assert "--json" in result.stdout


def test_unknown_option():
    result = run("--nope")
    assert result.returncode == 2
    assert "unknown option" in result.stderr


def test_default_output_has_logo():
    out = run("--logo", "linux", "--no-color").stdout
    assert "o_o" in out
    assert "USER:" in out


def test_logo_none():
    out = run("--logo", "none", "--no-color").stdout
    assert "o_o" not in out
    assert "USER:" in out


def test_logo_unknown():
    result = run("--logo", "nope")
    assert result.returncode == 2
    assert "unknown logo" in result.stderr


def test_logo_requires_value():
    result = run("--logo")
    assert result.returncode == 2


def test_no_logo():
    out = run("--no-logo", "--no-color").stdout
    assert "o_o" not in out
    assert "USER:" in out


def test_no_color():
    out = run("--no-color").stdout
    assert "\033" not in out


def test_color_by_default():
    out = run().stdout
    assert "\033[" in out


def test_json():
    result = run("--json")
    assert result.returncode == 0
    data = json.loads(result.stdout)
    assert set(data) == set(nullfetch.FIELDS)
    for key, value in data.items():
        if key in ("ram", "disk"):
            assert set(value) == {"used", "total", "percent"}
            assert 0 <= value["percent"] <= 100
        else:
            assert isinstance(value, str) and value


def test_only():
    out = run("--only", "cpu,ram", "--no-color").stdout
    assert "CPU:" in out
    assert "RAM:" in out
    assert "HOST:" not in out


def test_only_equals_form():
    out = run("--only=cpu", "--no-color").stdout
    assert "CPU:" in out
    assert "RAM:" not in out


def test_only_unknown_field():
    result = run("--only", "nope")
    assert result.returncode == 2
    assert "unknown field" in result.stderr


def test_only_requires_value():
    result = run("--only")
    assert result.returncode == 2


def test_render_side_by_side():
    art = nullfetch.ART.splitlines()
    out = nullfetch.render({"user": "u", "host": "h"}, art, False, width=120)
    lines = out.splitlines()
    assert lines[0].startswith("        .--.")
    assert "USER: u" in lines[0]
    assert "HOST: h" in lines[1]


def test_render_stacked_when_narrow():
    art = nullfetch.ART.splitlines()
    out = nullfetch.render({"user": "u"}, art, False, width=40)
    lines = out.splitlines()
    assert lines[0].startswith("        .--.")
    assert "USER: u" in lines
    assert "" in lines


def test_render_without_logo():
    out = nullfetch.render({"user": "u"}, [], False)
    assert out == "USER: u"


def test_long_value_truncated():
    out = nullfetch.render({"gpu": "x" * 200}, [], False, width=40)
    assert len(out) <= 40
    assert out.endswith("…")


def test_rgb_to_256():
    assert nullfetch.rgb_to_256(0, 0, 0) == 16
    assert nullfetch.rgb_to_256(255, 255, 255) == 231
    assert 16 <= nullfetch.rgb_to_256(71, 128, 97) <= 231


def test_format_size():
    line = nullfetch.format_size({"used": 1, "total": 4, "percent": 25}, False)
    assert "1 / 4 GB" in line
    assert "25%" in line
    assert line.count("█") + line.count("░") == nullfetch.BAR_WIDTH


def test_main_without_argv(monkeypatch, capsys):
    monkeypatch.setattr("sys.argv", ["nullfetch", "--version"])
    assert nullfetch.main() == 0
    assert capsys.readouterr().out.strip() == f"nullfetch {nullfetch.VERSION}"
