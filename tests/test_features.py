import json
import os
import subprocess
import sys
import threading
import time
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "nullfetch.py"

sys.path.insert(0, str(ROOT))
import nullfetch  # noqa: E402

ENV = {key: value for key, value in os.environ.items() if key != "NO_COLOR"}
ENV["TERM"] = "xterm-256color"
ENV["COLORTERM"] = "truecolor"


@pytest.fixture(autouse=True)
def isolated_dirs(monkeypatch, tmp_path):
    config_home = tmp_path / "config"
    cache_home = tmp_path / "cache"
    monkeypatch.setenv("XDG_CONFIG_HOME", str(config_home))
    monkeypatch.setenv("XDG_CACHE_HOME", str(cache_home))
    monkeypatch.setitem(ENV, "XDG_CONFIG_HOME", str(config_home))
    monkeypatch.setitem(ENV, "XDG_CACHE_HOME", str(cache_home))


def run(*args):
    return subprocess.run(
        [sys.executable, str(SCRIPT), *args],
        capture_output=True,
        text=True,
        env=ENV,
    )


def test_all_modules_in_cli():
    result = run("--all", "--no-network", "--no-color", "--logo", "none")
    assert result.returncode == 0
    assert "uptime:" in result.stdout
    assert "shell:" in result.stdout
    assert "packages:" in result.stdout
    assert "date:" in result.stdout


def test_extra_field_with_only():
    out = run("--only", "uptime,shell,arch", "--no-color", "--logo", "none").stdout
    assert "uptime:" in out
    assert "shell:" in out
    assert "arch:" in out
    assert "USER:" not in out


def test_list_fields_and_modes():
    fields = run("--list-fields").stdout.split()
    assert "uptime" in fields
    assert "public_ip" in fields
    modes = run("--list-image-modes").stdout.split()
    assert "blocks" in modes
    assert "kitty" in modes


def test_csv_and_markdown_output():
    csv_text = run("--format", "csv", "--only", "user,ram").stdout
    assert csv_text.splitlines()[0] == "field,value"
    assert "user," in csv_text
    markdown = run("--format", "markdown", "--only", "os").stdout
    assert markdown.startswith("| Field | Value |")
    assert "| OS |" in markdown


def test_custom_separator_and_bar_style():
    out = run(
        "--only",
        "ram",
        "--no-color",
        "--logo",
        "none",
        "--separator",
        " = ",
        "--bar-style",
        "dots",
    ).stdout
    assert "RAM =" in out
    assert "\u25cf" in out or "\u25cb" in out


def test_bar_width_option():
    line = nullfetch.format_size(
        {"used": 1.0, "total": 4.0, "percent": 25},
        False,
        bar_style="ascii",
        bar_width=5,
    )
    assert line.count("#") + line.count("-") == 5


@pytest.mark.parametrize("theme", ["nord", "dracula", "matrix", "solarized"])
def test_new_themes(theme):
    out = run("--theme", theme, "--color", "always", "--logo", "linux").stdout
    assert "\033[38;2;" in out


def test_timing_goes_to_stderr():
    result = run("--only", "user", "--timing")
    assert "fields in" in result.stderr
    assert result.stdout.strip()


def test_init_config_and_set(capsys):
    assert nullfetch.main(["--init-config"]) == 0
    path = nullfetch.config_path()
    data = json.loads(path.read_text())
    assert data["modules"] == list(nullfetch.FIELDS)
    capsys.readouterr()
    assert nullfetch.main(["--set", "theme=dracula"]) == 0
    assert nullfetch.main(["--set", "modules=os,cpu,ram"]) == 0
    data = json.loads(path.read_text())
    assert data["theme"] == "dracula"
    assert data["modules"] == ["os", "cpu", "ram"]
    capsys.readouterr()
    assert nullfetch.main(["--set", "separator= -> "]) == 0
    capsys.readouterr()
    assert nullfetch.main(["--show-config"]) == 0
    assert "separator:  ->" in capsys.readouterr().out


def test_config_modules_are_used(capsys):
    path = nullfetch.config_path()
    path.parent.mkdir(parents=True)
    path.write_text(json.dumps({"modules": ["os", "cpu"], "logo": "none"}))
    assert nullfetch.main(["--no-color"]) == 0
    out = capsys.readouterr().out
    assert "OS:" in out
    assert "CPU:" in out
    assert "USER:" not in out


@pytest.mark.parametrize(
    "data",
    [
        {"modules": ["nope"]},
        {"modules": "os,cpu"},
        {"ascii": "yes"},
        {"bar-width": "wide"},
        {"image-mode": "nope"},
        {"separator": 42},
    ],
)
def test_invalid_config_values(data):
    path = nullfetch.config_path()
    path.parent.mkdir(parents=True)
    path.write_text(json.dumps(data))
    with pytest.raises(SystemExit):
        nullfetch.main([])


def test_save_config_rejects_missing_image():
    with pytest.raises(SystemExit):
        nullfetch.main(["--image", "/nonexistent/picture.png", "--save-config"])
    assert not nullfetch.config_path().exists()


def test_alternate_config_file(tmp_path, capsys):
    path = tmp_path / "other.json"
    assert nullfetch.main(["--config", str(path), "--init-config"]) == 0
    capsys.readouterr()
    assert path.exists()
    assert nullfetch.main(["--config", str(path), "--show-config"]) == 0
    assert str(path) in capsys.readouterr().out


def test_no_network_removes_network_fields(capsys):
    nullfetch.os_release.cache_clear()
    assert nullfetch.main(["--all", "--no-network", "--json"]) == 0
    data = json.loads(capsys.readouterr().out)
    assert "public_ip" not in data
    assert "local_ip" not in data
    assert "network" not in data


def test_getters_with_mocked_files(monkeypatch):
    monkeypatch.setattr(nullfetch, "read", lambda path: "12345.67 0.0\n")
    assert nullfetch.get_uptime() == "3h 25m"
    monkeypatch.setenv("SHELL", "/bin/zsh")
    assert nullfetch.get_shell() == "zsh"
    monkeypatch.setenv("LANG", "ru_RU.UTF-8")
    assert nullfetch.get_locale() == "ru_RU.UTF-8"
    assert nullfetch.get_arch()
    assert nullfetch.get_processes().isdigit()


def test_swap_getter(monkeypatch):
    meminfo = "MemTotal: 2097152 kB\nSwapTotal: 1048576 kB\nSwapFree: 524288 kB\n"
    monkeypatch.setattr(nullfetch, "read", lambda path: meminfo)
    assert nullfetch.get_swap() == {
        "used": 0.5,
        "total": 1.0,
        "percent": 50,
    }


def test_os_release_is_memoized(monkeypatch):
    calls = []

    def fake_release():
        calls.append(1)
        return {"ID": "void"}

    monkeypatch.setattr(nullfetch.platform, "freedesktop_os_release", fake_release)
    nullfetch.os_release.cache_clear()
    assert nullfetch.os_release() == {"ID": "void"}
    assert nullfetch.os_release() == {"ID": "void"}
    assert len(calls) == 1


def test_collect_runs_getters_in_parallel(monkeypatch):
    ready = threading.Event()

    def first():
        assert ready.wait(2)
        return "first"

    def second():
        ready.set()
        return "second"

    monkeypatch.setitem(nullfetch.GETTERS, "arch", first)
    monkeypatch.setitem(nullfetch.GETTERS, "kernel", second)
    started = time.monotonic()
    result = nullfetch.collect(["arch", "kernel"], use_cache=False)
    assert result == {"arch": "first", "kernel": "second"}
    assert time.monotonic() - started < 1.0


def test_zero_cache_ttl_disables_cache():
    result = run("--cache-ttl", "0", "--only", "user")
    assert result.returncode == 0


def test_cli_only_overrides_saved_all(capsys):
    path = nullfetch.config_path()
    path.parent.mkdir(parents=True)
    path.write_text(json.dumps({"all": True, "logo": "none"}))
    assert nullfetch.main(["--only", "user", "--no-color"]) == 0
    assert capsys.readouterr().out.strip() == "USER: " + nullfetch.get_user()


def test_saved_format_is_reused(capsys):
    assert (
        nullfetch.main(["--format", "markdown", "--only", "os", "--save-config"]) == 0
    )
    capsys.readouterr()
    assert nullfetch.main([]) == 0
    assert capsys.readouterr().out.startswith("| Field | Value |")
