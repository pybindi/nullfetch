import json
from conftest import run


def test_all_modules():
    r = run("--all", "--no-network", "--no-color", "--logo", "none")
    assert r.returncode == 0
    assert "uptime:" in r.stdout
    assert "shell:" in r.stdout


def test_presets():
    r = run("--preset", "minimal", "--no-color", "--logo", "none")
    assert r.returncode == 0
    assert "OS:" in r.stdout
    assert "CPU:" in r.stdout


def test_formats():
    r = run("--format", "csv", "--only", "user,os")
    assert r.returncode == 0
    assert "field,value" in r.stdout
    r = run("--format", "markdown", "--only", "os")
    assert r.returncode == 0
    assert "| Field | Value |" in r.stdout


def test_command_module():
    r = run("--only", "command:hi:echo hello", "--no-color", "--logo", "none")
    assert r.returncode == 0
    assert "hi:" in r.stdout
    assert "hello" in r.stdout


def test_doctor_and_timing():
    r = run("--doctor")
    assert r.returncode == 0
    assert "rust_core: yes" in r.stdout
    r = run("--only", "user", "--timing")
    assert r.returncode == 0
    assert "fields in" in r.stderr
