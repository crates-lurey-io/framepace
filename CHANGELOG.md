# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-06-28

### Added

- `FrameTimer<N, T>`: rolling frame-time tracker with configurable window size
  and pluggable time source
- `FrameStats`: mean, min, max, p50, p95, p99 statistics over the window
- `PhaseGuard`: RAII guard for timing named sub-phases within a frame
- `TimeSource` trait for custom clocks (TSC, WASM `performance.now()`, etc.)
- `StdTime`: default time source backed by `std::time::Instant`
- `FrameTimer::samples_us()`: raw sample iterator for CSV export and benchmark assertions
- `FrameTimer::to_csv()`: serializes the sample window as a comma-separated string
- `FrameTimer::is_over_budget(budget_ms)`: checks whether the last frame exceeded a time budget
- `FrameTimer::reset()`: clears samples while preserving phase registrations
