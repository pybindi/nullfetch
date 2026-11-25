import json
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import nullfetch  # noqa: E402


@pytest.fixture(autouse=True)
def config_home(monkeypatch, tmp_path):
    monkeypatch.setenv("XDG_CONFIG_HOME", str(tmp_path))


def test_save_and_load(capsys):
    assert nullfetch.main(
        ["--logo", "cat", "--accent", "#ff8800", "--save-config"]
    ) == 0
    data = json.loads(nullfetch.config_path().read_text())
    assert data["logo"] == "cat"
    assert data["accent"] == "#ff8800"
    capsys.readouterr()
    assert nullfetch.main(["--only", "user", "--color", "always"]) == 0
    out = capsys.readouterr().out
    assert "o.o" in out
    assert nullfetch.fg((255, 136, 0)) in out


def test_cli_overrides_config(capsys):
    nullfetch.main(["--logo=cat", "--save-config"])
    capsys.readouterr()
    nullfetch.main(["--logo=linux", "--only=user", "--no-color"])
    assert "o_o" in capsys.readouterr().out
    assert json.loads(nullfetch.config_path().read_text())["logo"] == "cat"


def test_reset_bad_config(capsys):
    path = nullfetch.config_path()
    path.parent.mkdir(parents=True)
    path.write_text("broken json")
    with pytest.raises(SystemExit):
        nullfetch.main([])
    assert nullfetch.main(["--reset-config"]) == 0
    assert not path.exists()
    assert nullfetch.main(["--reset-config"]) == 0


@pytest.mark.parametrize("data", [[], {"json": "true"}, {"logo": 42}])
def test_invalid_config(data):
    path = nullfetch.config_path()
    path.parent.mkdir(parents=True)
    path.write_text(json.dumps(data))
    with pytest.raises(SystemExit):
        nullfetch.main([])
    assert nullfetch.main(["--help"]) == 0


def test_invalid_choice_not_saved():
    with pytest.raises(SystemExit):
        nullfetch.main(["--logo=nope", "--save-config"])
    assert not nullfetch.config_path().exists()
    with pytest.raises(SystemExit):
        nullfetch.main(["--accent=notacolor", "--save-config"])
    assert not nullfetch.config_path().exists()


def test_logo_file_saved_absolute(tmp_path, capsys, monkeypatch):
    monkeypatch.chdir(tmp_path)
    Path("art.txt").write_text("my logo\n")
    nullfetch.main(["--logo-file=art.txt", "--save-config"])
    assert json.loads(nullfetch.config_path().read_text())["logo-file"] == str(
        tmp_path / "art.txt"
    )
    capsys.readouterr()
    nullfetch.main(["--logo=linux", "--only=user"])
    assert "o_o" in capsys.readouterr().out


def test_no_logo_saved(capsys):
    nullfetch.main(["--no-logo", "--save-config"])
    capsys.readouterr()
    nullfetch.main(["--only=user"])
    assert len(capsys.readouterr().out.splitlines()) == 1


def test_default_config_path(monkeypatch, tmp_path):
    monkeypatch.delenv("XDG_CONFIG_HOME")
    monkeypatch.setenv("HOME", str(tmp_path))
    assert nullfetch.config_path() == tmp_path / ".config/nullfetch/config.json"


def test_show_config_empty(capsys):
    assert nullfetch.main(["--show-config"]) == 0
    assert "no saved defaults" in capsys.readouterr().out


def test_show_config_saved(capsys):
    nullfetch.main(
        ["--logo=cat", "--accent=blue", "--theme=light", "--save-config"]
    )
    capsys.readouterr()
    assert nullfetch.main(["--show-config"]) == 0
    out = capsys.readouterr().out
    assert f"config: {nullfetch.config_path()}" in out
    assert "logo: cat" in out
    assert "accent: blue" in out
    assert "theme: light" in out


def test_show_config_broken(capsys):
    path = nullfetch.config_path()
    path.parent.mkdir(parents=True)
    path.write_text("broken")
    with pytest.raises(SystemExit):
        nullfetch.main(["--show-config"])
    assert "cannot load config" in capsys.readouterr().err
