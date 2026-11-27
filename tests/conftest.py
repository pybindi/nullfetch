import os
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
BIN = ROOT / "target" / "release" / "nullfetch"
SCRIPT = ROOT / "nullfetch.py"

ENV = {k: v for k, v in os.environ.items() if k != "NO_COLOR"}
ENV["TERM"] = "xterm-256color"
ENV["COLORTERM"] = "truecolor"
ENV["PATH"] = str(Path.home() / ".cargo" / "bin") + ":" + ENV.get("PATH", "")


def run(*args, env=None):
    e = ENV.copy()
    if env:
        e.update(env)
    cmd = [str(BIN), *args] if BIN.exists() else [sys.executable, str(SCRIPT), *args]
    return subprocess.run(cmd, capture_output=True, text=True, env=e)


@pytest.fixture(autouse=True)
def isolated_config(monkeypatch, tmp_path):
    monkeypatch.setenv("XDG_CONFIG_HOME", str(tmp_path / "config"))
    monkeypatch.setenv("XDG_CACHE_HOME", str(tmp_path / "cache"))
