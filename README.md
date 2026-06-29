# framepace

Rolling frame-time statistics for game loops and real-time applications.

[![Test](https://github.com/crates-lurey-io/framepace/actions/workflows/test.yml/badge.svg)](https://github.com/crates-lurey-io/framepace/actions/workflows/test.yml)
[![Docs](https://github.com/crates-lurey-io/framepace/actions/workflows/docs.yml/badge.svg)](https://github.com/crates-lurey-io/framepace/actions/workflows/docs.yml)
[![Crates.io Version](https://img.shields.io/crates/v/framepace)](https://crates.io/crates/framepace)

`framepace` tracks per-frame wall-time durations in a fixed-size ring buffer
and exposes mean, min, max, and percentile (p50, p95, p99) statistics over the
window. No rendering. No rate-limiting. No loop control. Just measurement.

## Usage

```rust
use framepace::FrameTimer;

let mut timer = FrameTimer::new();

// At the top of every tick:
timer.begin_frame();

// Query at any point:
if let Some(stats) = timer.stats() {
    println!("fps={:.1}  avg={:.2}ms  p95={:.2}ms", stats.fps, stats.avg_ms, stats.p95_ms);
}
```

Named phases let you break down where time is spent:

```rust
use framepace::FrameTimer;

let mut timer = FrameTimer::new();
timer.begin_frame();

{
    let _update = timer.phase("update");
    // ... update logic ...
}
{
    let _render = timer.phase("render");
    // ... render logic ...
}

if let Some(stats) = timer.phase_stats("render") {
    println!("render p95={:.2}ms", stats.p95_ms);
}
```

Plug in a custom time source for higher precision or WASM compatibility:

```rust
use framepace::{FrameTimer, TimeSource};

struct QuantaTime;

impl TimeSource for QuantaTime {
    fn now_us() -> u64 {
        // quanta::Instant::recent().as_u64() / 1_000
        0
    }
}

let mut timer: FrameTimer<64, QuantaTime> = FrameTimer::new();
```

## Contributing

This project uses [`just`][] to run commands the same way as CI:

- `just check` — check formatting and lints
- `just test` — run tests
- `just coverage` — generate and preview code coverage
- `just doc` — generate and preview docs

[`just`]: https://crates.io/crates/just

For a full list of commands, see the [`Justfile`](./Justfile).

## Releasing

1. Update the version in `Cargo.toml` and `CHANGELOG.md`, then commit.
2. Push a tag matching the version (e.g. `v0.1.1`).
3. The [`publish`](.github/workflows/publish.yml) workflow validates the tag,
   runs checks and tests, publishes to [crates.io](https://crates.io/crates/framepace),
   and creates a GitHub Release with the changelog excerpt.
