# flatroom (public)

Independent open tool to summarize your own calibration export files
(JSON). Clean-room implementation: no vendor code, curves, schemas, or
protocols. Not affiliated with A1 Evo AcoustiX, Audyssey, Denon, Marantz,
or Room EQ Wizard — see `THIRD_PARTY_NOTICES.md`.

## Install / Run

Requires Rust (stable).

```sh
cargo build --locked
./target/debug/flatroom inspect <file>
./target/debug/flatroom inspect --json <file>
```

## What it does

Reads one local file you own, parses it as JSON, and prints a generic
inventory: byte size, top-level shape, sorted key list (first 64), and
child counts for nested objects/arrays. Non-JSON input fails with
`not JSON: <reason>`; missing files fail with `no such file`.

## Docs

See `docs/`.
