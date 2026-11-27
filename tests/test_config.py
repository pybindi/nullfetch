import json
from pathlib import Path
from conftest import run, ENV
import os, subprocess

def run_cfg(*args, xdg):
    e = ENV.copy(); e["XDG_CONFIG_HOME"]=str(xdg)
    cmd = [str(Path(__file__).resolve().parents[1]/"target/release/nullfetch"), *args]
    return subprocess.run(cmd, capture_output=True, text=True, env=e)


def test_save_and_load(tmp_path):
    xdg = tmp_path/"config"
    r = run_cfg("--logo","linux","--save-config", xdg=xdg)
    assert r.returncode == 0
    p = xdg/"nullfetch"/"config.json"
    data = json.loads(p.read_text())
    assert data["logo"] == "linux"
    r2 = run_cfg("--only","user","--no-color","--logo","none", xdg=xdg)
    assert "USER:" in r2.stdout


def test_reset_config(tmp_path):
    xdg = tmp_path/"config"
    run_cfg("--save-config", xdg=xdg)
    p = xdg/"nullfetch"/"config.json"
    assert p.exists()
    r = run_cfg("--reset-config", xdg=xdg)
    assert r.returncode == 0
    assert not p.exists()


def test_show_config(tmp_path):
    xdg = tmp_path/"config"
    run_cfg("--logo","linux","--save-config", xdg=xdg)
    r = run_cfg("--show-config", xdg=xdg)
    assert r.returncode == 0
    assert "logo" in r.stdout


def test_init_and_set(tmp_path):
    xdg = tmp_path/"config"
    r = run_cfg("--init-config", xdg=xdg)
    assert r.returncode == 0, r.stderr
    r = run_cfg("--set","theme=dracula", xdg=xdg)
    assert r.returncode == 0
    data = json.loads((xdg/"nullfetch"/"config.json").read_text())
    assert data["theme"] == "dracula"
