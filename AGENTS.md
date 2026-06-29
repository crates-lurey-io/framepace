# Agent Guidelines for framepace

## Rust API Guidelines

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html).

Key rules enforced by CI:

- No trait bounds on struct definitions (`C-STRUCT-BOUNDS`): bounds belong on
  `impl` blocks only. `FrameTimer<N, T>` stores `PhantomData<T>`; the `T:
  TimeSource` bound appears only on impl blocks that actually call `T::now_us()`.
- `#[non_exhaustive]` on public data structs (`FrameStats`, `StdTime`) so
  adding fields is non-breaking.
- `#[must_use]` on all methods returning a meaningful value. The `PhaseGuard`
  struct itself is `#[must_use]` because dropping it immediately is almost
  always a bug.
- Dual-licensed `MIT OR Apache-2.0`.
- Keep `CHANGELOG.md` up to date with every public API change.

## Code style

- `cargo just check` before committing (format + clippy with all pedantic and
  nursery lints as errors).
- `cargo just test-all` to run all tests including doctests.
- `cargo just semver-checks` to verify no accidental breaking changes.
- All public items must have doc comments with at least one `# Example`.
- No `eprintln!` or `println!` in library code.
- `unsafe_code` is `forbid`; no unsafe blocks anywhere.

## Design invariants

- `FrameTimer` stores `N` frame samples in a ring buffer indexed by `head`.
  The invariant `head < N` is maintained by `% N` in every write path.
- Phase slots are registered by name on first use and reused thereafter.
  The inline array has `MAX_PHASES = 8` slots; extra phase names beyond that
  limit are silently dropped (safe, never panics).
- `FrameStats::compute` sorts a stack-allocated copy of the window to compute
  percentiles. This is O(N log N) on the stack and is only called when the
  caller explicitly requests statistics.

## No interactive jj/git

Never use `-i`/`--interactive` flags. Always pass `-m` to `jj describe`/`jj new`.
