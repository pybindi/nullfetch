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
    out = run("--no-color").stdout
    assert "o_o" in out
    assert "USER:" in out


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
    assert all(isinstance(value, str) and value for value in data.values())


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
    out = nullfetch.render({"user": "u", "host": "h"}, True, False)
    lines = out.splitlines()
    assert len(lines) >= 7
    assert lines[0].startswith("        .--.")
    assert "USER: u" in lines[0]
    assert "HOST: h" in lines[1]


def test_render_without_logo():
    out = nullfetch.render({"user": "u"}, False, False)
    assert out == "USER: u"


def test_rgb_to_256():
    assert nullfetch.rgb_to_256(0, 0, 0) == 16
    assert nullfetch.rgb_to_256(255, 255, 255) == 231
    assert 16 <= nullfetch.rgb_to_256(71, 128, 97) <= 231
