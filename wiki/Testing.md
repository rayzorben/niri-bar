# Testing

Rules
- All tests in `tests/` (no inline in `src/*` unless critical).
- Validate the real `niri-bar.yaml` against `src/niri-bar-yaml.schema.json`.
- Simulate Niri IPC by feeding event JSON lines to `NiriBus`.
- No reliance on actual monitors; treat names as data.

Commands
```bash
cargo test -- --test-threads=1
cargo fmt -- --check
cargo clippy -- -D warnings
```

Coverage target: ≥80%.

