# Contributing

## Dev setup

```bash
git clone https://github.com/pybindi/nullfetch.git
cd nullfetch
cargo build --release
```

## Run

```bash
./target/release/nullfetch
```

## Tests

```bash
cargo test
python3 -m pytest -q --ignore=nullfetch --ignore=target
```

## Style

- Keep it fast and simple.
- No required runtime dependencies.
- Preserve CLI compatibility where practical.
