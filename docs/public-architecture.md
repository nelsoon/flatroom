# Public architecture

`flatroom` is a single Rust binary (std + `serde_json`).

- `src/summary.rs` — pure `summarize(bytes)`: parses JSON, returns the
  size, top-level shape, sorted key list (cap 64), per-key child counts,
  and a truncation flag. No I/O, no vendor knowledge.
- `src/main.rs` — CLI: `inspect [--json] <path>`. Refuses files over
  64 MiB, maps errors to exit codes (`2` usage, `1` missing/invalid),
  prints human text or one JSON summary object.
- `tests/cli.rs` — end-to-end coverage of the four spec scenarios using
  synthetic fixtures only.

Limits: read-only summaries. No EQ computation, no target curves, no
device control, no network.
