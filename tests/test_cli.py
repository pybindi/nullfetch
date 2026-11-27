import json
import os
import subprocess
import sys
from pathlib import Path

from conftest import run, BIN

FIELDS = ["user","host","os","kernel","cpu","gpu","ram","disk"]


def test_version():
    r = run("--version")
    assert r.returncode == 0
    assert r.stdout.strip().startswith("nullfetch ")


def test_help():
    r = run("--help")
    assert r.returncode == 0
    assert "--no-logo" in r.stdout
    assert "--json" in r.stdout
    assert "--list-logos" in r.stdout


def test_unknown_option():
    r = run("--nope")
    assert r.returncode == 2
    assert "unknown option" in r.stderr


def test_default_output_has_logo():
    out = run("--logo", "linux", "--no-color").stdout
    assert "USER:" in out


def test_logo_none():
    out = run("--logo", "none", "--no-color").stdout
    assert "o_o" not in out
    assert "USER:" in out


def test_no_logo():
    out = run("--no-logo", "--no-color").stdout
    assert "USER:" in out


def test_no_color():
    out = run("--no-color").stdout
    assert "\033" not in out


def test_color_modes():
    assert "\033[" in run("--color", "always").stdout
    assert "\033[" not in run("--color", "never").stdout


def test_theme_invalid():
    r = run("--theme", "sunset")
    assert r.returncode == 2


def test_json():
    r = run("--json")
    assert r.returncode == 0
    data = json.loads(r.stdout)
    assert set(data) == set(FIELDS)


def test_only():
    out = run("--only", "cpu,ram", "--no-color", "--logo", "none").stdout
    assert "CPU:" in out
    assert "RAM:" in out
    assert "HOST:" not in out


def test_only_unknown_field():
    r = run("--only", "nope")
    assert r.returncode == 2


def test_width_option():
    out = run("--width", "50", "--no-color").stdout
    assert max((len(l) for l in out.splitlines()), default=0) <= 50


def test_oneline():
    out = run("--oneline", "--no-color").stdout.strip()
    assert len(out.splitlines()) == 1
    assert "·" in out or "|" in out


def test_list_logos():
    r = run("--list-logos")
    assert r.returncode == 0
    assert "arch" in r.stdout or "linux" in r.stdout
