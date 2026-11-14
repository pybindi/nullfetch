import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "nullfetch.py"

sys.path.insert(0, str(ROOT))
import nullfetch  # noqa: E402

ENV = {key: value for key, value in os.environ.items() if key != "NO_COLOR"}
ENV["TERM"] = "xterm-256color"
ENV["COLORTERM"] = "truecolor"


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
    assert "--list-logos" in result.stdout


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


def test_color_modes():
    assert "\033[" in run("--color", "always").stdout
    assert "\033[" not in run("--color", "never").stdout


def test_color_mode_invalid():
    result = run("--color", "loud")
    assert result.returncode == 2
    assert "invalid color mode" in result.stderr


def test_themes():
    assert "\033[" not in run("--theme", "mono").stdout
    assert "38;2;255;80;220" in run(
        "--theme", "neon", "--color", "always"
    ).stdout


def test_theme_invalid():
    result = run("--theme", "sunset")
    assert result.returncode == 2
    assert "unknown theme" in result.stderr


def test_compact_output():
    out = run("--compact", "--no-color").stdout
    assert "\n\n" not in out


def test_color_by_default():
    out = run().stdout
    assert "\033[" not in out


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
    line = nullfetch.format_size({"used": 1.0, "total": 4.0, "percent": 25}, False)
    assert "1.0 / 4.0 GiB" in line
    assert "25%" in line
    assert line.count("█") + line.count("░") == nullfetch.BAR_WIDTH


def test_main_without_argv(monkeypatch, capsys):
    monkeypatch.setattr("sys.argv", ["nullfetch", "--version"])
    assert nullfetch.main() == 0
    assert capsys.readouterr().out.strip() == f"nullfetch {nullfetch.VERSION}"


def test_ascii_bars():
    out = run("--ascii", "--no-color", "--logo", "none").stdout
    assert "#" in out
    assert "-" in out
    assert "█" not in out


def test_width_option():
    out = run("--width", "50", "--no-color").stdout
    assert max(len(line) for line in out.splitlines()) <= 50


def test_very_narrow_width_option():
    out = run("--width", "20", "--no-color").stdout
    assert max(len(line) for line in out.splitlines()) <= 20


def test_width_invalid():
    assert run("--width", "5").returncode == 2


def test_oneline():
    out = run("--oneline", "--no-color").stdout.strip()
    assert len(out.splitlines()) == 1
    assert "@" in out
    assert "·" in out


def test_oneline_ascii():
    out = run("--oneline", "--ascii", "--no-color").stdout
    assert "·" not in out
    assert " | " in out


@pytest.mark.parametrize("field", ["user", "host"])
def test_oneline_single_identity_field(field):
    out = run("--oneline", "--only", field, "--no-color").stdout.strip()
    assert out


def test_oneline_labels_sizes():
    out = run("--oneline", "--only", "ram,disk", "--no-color").stdout
    assert "RAM " in out
    assert "disk " in out


def test_list_logos():
    result = run("--list-logos")
    assert result.returncode == 0
    assert "arch" in result.stdout
    assert "void" in result.stdout


def test_accent():
    out = run(
        "--accent", "#ff0000", "--logo", "linux", "--color", "always"
    ).stdout
    assert "38;2;255;0;0" in out


def test_accent_invalid():
    result = run("--accent", "red")
    assert result.returncode == 2
    assert "invalid color" in result.stderr


def test_logo_file(tmp_path):
    path = tmp_path / "art.txt"
    path.write_text("(o.o)\n")
    out = run("--logo-file", str(path), "--no-color").stdout
    assert "(o.o)" in out


def test_logo_file_missing():
    result = run("--logo-file", "/nonexistent/art.txt")
    assert result.returncode == 2
    assert "cannot read" in result.stderr


def test_logo_file_limits(tmp_path):
    path = tmp_path / "huge.txt"
    path.write_text("x" * 100_001)
    result = run("--logo-file", str(path))
    assert result.returncode == 2
    assert "too large" in result.stderr


def test_distro_logos_present():
    for name in ("fedora", "opensuse", "alpine", "nixos", "gentoo", "mint"):
        assert name in nullfetch.LOGOS


@pytest.mark.parametrize(
    "args",
    [
        ["--version"],
        ["--help"],
        ["--no-color", "--logo", "none"],
        ["--json"],
        ["--oneline"],
        ["--ascii", "--no-color"],
        ["--width", "60"],
        ["--accent", "#00ff00"],
        ["--only", "user"],
        ["--logo", "cat", "--no-color"],
        ["--logo", "arch", "--no-color"],
    ],
)
def test_main_success(args, capsys):
    assert nullfetch.main(args) == 0
    assert capsys.readouterr().out


def test_main_json(capsys):
    assert nullfetch.main(["--json", "--only", "user"]) == 0
    data = json.loads(capsys.readouterr().out)
    assert set(data) == {"user"}


def test_main_errors(capsys):
    for args in (["--only"], ["--only", "nope"], ["--logo"], ["--width", "abc"]):
        with pytest.raises(SystemExit):
            nullfetch.main(args)
        capsys.readouterr()


def test_system_getters():
    assert nullfetch.get_user()
    assert nullfetch.get_host()
    assert nullfetch.get_os()
    assert nullfetch.get_kernel()
    assert nullfetch.get_cpu()
    assert nullfetch.read("/proc/uptime")
    assert nullfetch.os_release()


def test_size_values():
    ram = nullfetch.get_ram()
    assert set(ram) == {"used", "total", "percent"}
    assert 0 <= ram["percent"] <= 100

    disk = nullfetch.get_disk()
    assert set(disk) == {"used", "total", "percent"}
    assert nullfetch.size_value(0, 0)["percent"] == 0


def test_get_ram_uses_fractional_gib(monkeypatch):
    meminfo = "MemTotal:       2097152 kB\nMemAvailable:   1048576 kB\n"
    monkeypatch.setattr(nullfetch, "read", lambda path: meminfo)
    value = nullfetch.get_ram()
    assert value == {"used": 1.0, "total": 2.0, "percent": 50}


def test_get_gpu_lists_multiple(monkeypatch):
    output = """00:02.0 VGA compatible controller: Intel GPU (rev 01)
01:00.0 3D controller: NVIDIA GPU (rev 02)
"""
    monkeypatch.setattr(nullfetch.shutil, "which", lambda name: "/usr/bin/lspci")
    monkeypatch.setattr(
        nullfetch.subprocess,
        "run",
        lambda *args, **kwargs: type("Result", (), {"stdout": output})(),
    )
    assert nullfetch.get_gpu() == "Intel GPU, NVIDIA GPU"


def test_distro_color_and_fg():
    assert len(nullfetch.distro_color()) == 3
    assert nullfetch.fg((1, 2, 3)).startswith("\033[38;")
