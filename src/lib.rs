//! Rolling frame-time statistics for game loops and real-time applications.
//!
//! `framepace` tracks per-frame wall-time durations in a fixed-size ring
//! buffer and exposes mean, min, max, and percentile (p50, p95, p99)
//! statistics over the window. There is no rendering, no rate-limiting, and
//! no loop control — just measurement.
//!
//! # Quick start
//!
//! ```rust
//! use framepace::FrameTimer;
//!
//! let mut timer: FrameTimer = FrameTimer::new();
//!
//! // At the top of every tick:
//! timer.begin_frame();
//!
//! // Query at any point:
//! if let Some(stats) = timer.stats() {
//!     println!("fps={:.1}  avg={:.2}ms  p95={:.2}ms", stats.fps, stats.avg_ms, stats.p95_ms);
//! }
//! ```
//!
//! # Named phases
//!
//! Break down where time goes within a frame using RAII phase guards:
//!
//! ```rust
//! use framepace::FrameTimer;
//!
//! let mut timer: FrameTimer = FrameTimer::new();
//! timer.begin_frame();
//!
//! {
//!     let _update = timer.phase("update");
//!     // ... update logic ...
//! } // guard drops here, recording the elapsed time
//!
//! {
//!     let _render = timer.phase("render");
//!     // ... render logic ...
//! }
//!
//! if let Some(stats) = timer.phase_stats("render") {
//!     println!("render p95={:.2}ms", stats.p95_ms);
//! }
//! ```
//!
//! # Custom time sources
//!
//! The default [`DefaultTime`] uses [`std::time::Instant`]. Implement
//! [`TimeSource`] to use a higher-precision clock (e.g. TSC via the `quanta`
//! crate) or to adapt to platforms where `Instant` is unavailable:
//!
//! ```rust
//! use framepace::{FrameTimer, TimeSource};
//!
//! struct QuantaTime;
//!
//! impl TimeSource for QuantaTime {
//!     fn now_us() -> u64 {
//!         // e.g. quanta::Instant::now().as_u64() / 1_000
//!         0 // placeholder
//!     }
//! }
//!
//! let mut timer: FrameTimer<64, QuantaTime> = FrameTimer::new();
//! timer.begin_frame();
//! ```

pub mod time;

mod phase;
mod stats;
mod timer;

pub use phase::PhaseGuard;
pub use stats::FrameStats;
pub use time::{DefaultTime, TimeSource};
pub use timer::{FrameTimer, MAX_PHASES};

#[cfg(test)]
mod tests {
    use super::*;

    // Verify the default FrameTimer is Send + Sync so it can be stashed in
    // game state that crosses thread boundaries.
    #[test]
    fn assert_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<FrameTimer>();
        assert_sync::<FrameTimer>();
    }
}
