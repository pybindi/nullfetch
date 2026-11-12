# Contributing

Thanks for taking a look at nullfetch.

## Requirements

- Python 3.10+
- Linux
- `pytest` for running tests

## Running from source

```bash
git clone https://github.com/pybindi/nullfetch.git
cd nullfetch
python nullfetch.py
```

## Tests

```bash
python -m pytest -q
```

The tests run the script as a subprocess, so they check the real command line
behaviour (`--version`, `--json`, `--only`, ...).

## Code style

- standard library only — no dependencies;
- keep functions small and named after what they return (`get_cpu`, `get_ram`);
- every new field goes into `FIELDS`, `LABELS` and `GETTERS`;
- add a test for every new option.

## Adding a field

1. Write a `get_something()` function that returns a string.
2. Register it in `GETTERS`.
3. Add the name to `FIELDS` and a label to `LABELS`.
4. Update `nullfetch.1` and the README options table.
5. Run the tests.

## Pull requests

Small, focused PRs are easiest to review. Describe what changed and why.
