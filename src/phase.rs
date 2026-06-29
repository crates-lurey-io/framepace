//! Phase sub-timer types: [`PhaseData`] (internal storage) and
//! [`PhaseGuard`] (RAII guard returned to callers).

// `redundant_pub_crate` fires on `pub(crate)` items inside a private module.
// The items genuinely need `pub(crate)` because they are used from `timer.rs`.
#![allow(clippy::redundant_pub_crate)]

use core::fmt;
use core::marker::PhantomData;

use crate::time::TimeSource;
use crate::timer::FrameTimer;

// ── Internal storage ─────────────────────────────────────────────────────────

/// Internal ring-buffer storage for a single named phase.
///
/// One `PhaseData<N>` slot lives inside `FrameTimer`'s inline phase array.
/// Fields are `pub(crate)` so `FrameTimer` can manipulate them directly
/// without going through the public API.
#[derive(Clone, Copy)]
pub(crate) struct PhaseData<const N: usize> {
    pub(crate) name: &'static str,
    pub(crate) samples: [u64; N],
    pub(crate) head: usize,
    pub(crate) count: usize,
}

impl<const N: usize> PhaseData<N> {
    /// Returns an empty, unnamed slot ready to be claimed by a new phase.
    pub(crate) const fn empty() -> Self {
        Self {
            name: "",
            samples: [0_u64; N],
            head: 0,
            count: 0,
        }
    }
}

impl<const N: usize> fmt::Debug for PhaseData<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhaseData")
            .field("name", &self.name)
            .field("count", &self.count)
            .finish_non_exhaustive()
    }
}

// ── Public RAII guard ─────────────────────────────────────────────────────────

/// RAII guard that times a named sub-phase of a frame.
///
/// Created by [`FrameTimer::phase`][crate::FrameTimer::phase]. The elapsed
/// time is recorded into the parent timer's ring buffer when this guard is
/// dropped.
///
/// # Example
///
/// ```rust
/// use framepace::FrameTimer;
///
/// let mut timer: FrameTimer = FrameTimer::new();
/// timer.begin_frame();
///
/// {
///     let _render = timer.phase("render");
///     // ... render work ...
/// } // elapsed time recorded here
///
/// if let Some(stats) = timer.phase_stats("render") {
///     println!("render avg={:.2}ms", stats.avg_ms);
/// }
/// ```
#[must_use = "dropping PhaseGuard immediately records a near-zero duration; \
              keep it alive for the duration of the phase"]
pub struct PhaseGuard<'a, const N: usize, T: TimeSource> {
    timer: &'a mut FrameTimer<N, T>,
    name: &'static str,
    start_us: u64,
    _time: PhantomData<T>,
}

impl<'a, const N: usize, T: TimeSource> PhaseGuard<'a, N, T> {
    pub(crate) fn new(timer: &'a mut FrameTimer<N, T>, name: &'static str) -> Self {
        Self {
            timer,
            name,
            start_us: T::now_us(),
            _time: PhantomData,
        }
    }
}

impl<const N: usize, T: TimeSource> Drop for PhaseGuard<'_, N, T> {
    fn drop(&mut self) {
        let elapsed = T::now_us().saturating_sub(self.start_us);
        self.timer.record_phase(self.name, elapsed);
    }
}

impl<const N: usize, T: TimeSource> fmt::Debug for PhaseGuard<'_, N, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhaseGuard")
            .field("name", &self.name)
            .field("start_us", &self.start_us)
            .finish_non_exhaustive()
    }
}
