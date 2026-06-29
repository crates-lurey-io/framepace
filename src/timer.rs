//! [`FrameTimer`]: the main entry point for frame-time tracking.

use core::fmt;
use core::marker::PhantomData;

use crate::phase::{PhaseData, PhaseGuard};
use crate::stats::FrameStats;
use crate::time::{DefaultTime, TimeSource};

/// Maximum number of named phases a [`FrameTimer`] can track simultaneously.
///
/// Phases beyond this limit are silently ignored. Eight phases is enough for
/// typical breakdowns (input, update, physics, render, present, audio, …).
pub const MAX_PHASES: usize = 8;

/// Rolling frame-time tracker with optional named sub-phase timers.
///
/// `N` sets the rolling window size in frames (default 64). Larger windows
/// smooth out percentile estimates; smaller windows react faster to spikes.
///
/// `T` is the time source (default [`DefaultTime`]). Swap it for a
/// high-precision clock by passing a type that implements [`TimeSource`].
///
/// # Example
///
/// ```rust
/// use framepace::FrameTimer;
///
/// let mut timer: FrameTimer = FrameTimer::new();
///
/// for _ in 0..10 {
///     timer.begin_frame();
///     // ... game tick ...
/// }
///
/// let stats = timer.stats().expect("should have samples by now");
/// assert!(stats.fps > 0.0);
/// ```
pub struct FrameTimer<const N: usize = 64, T = DefaultTime> {
    samples: [u64; N],
    head: usize,
    count: usize,
    frame_start_us: u64,
    started: bool,
    phases: [PhaseData<N>; MAX_PHASES],
    phase_len: usize,
    _time: PhantomData<T>,
}

// ── Construction ─────────────────────────────────────────────────────────────

impl<const N: usize, T> FrameTimer<N, T> {
    /// Creates a new timer with an empty window.
    #[must_use]
    pub const fn new() -> Self {
        let empty = PhaseData::empty();
        Self {
            samples: [0_u64; N],
            head: 0,
            count: 0,
            frame_start_us: 0,
            started: false,
            phases: [empty; MAX_PHASES],
            phase_len: 0,
            _time: PhantomData,
        }
    }

    /// Resets the frame-time window and all phase windows to empty.
    ///
    /// Phase names are preserved so that subsequent calls to
    /// [`phase`][Self::phase] and [`phase_stats`][Self::phase_stats] continue
    /// to work without re-registration. Use this to discard warmup frames
    /// before a benchmark run.
    pub fn reset(&mut self) {
        self.samples = [0_u64; N];
        self.head = 0;
        self.count = 0;
        self.frame_start_us = 0;
        self.started = false;
        for phase in &mut self.phases[..self.phase_len] {
            phase.samples = [0_u64; N];
            phase.head = 0;
            phase.count = 0;
        }
    }
}

impl<const N: usize, T> Default for FrameTimer<N, T> {
    fn default() -> Self {
        Self::new()
    }
}

// ── Measurement (requires a time source) ─────────────────────────────────────

impl<const N: usize, T: TimeSource> FrameTimer<N, T> {
    /// Records the start of a new frame.
    ///
    /// Call this at the very top of your tick function — before any game
    /// logic — so the elapsed time includes vsync waits and OS scheduling
    /// that occur between ticks.
    ///
    /// The first call anchors the clock. The inter-frame duration is recorded
    /// on every subsequent call.
    pub fn begin_frame(&mut self) {
        let now = T::now_us();
        if self.started {
            let elapsed = now.saturating_sub(self.frame_start_us);
            self.samples[self.head] = elapsed;
            self.head = (self.head + 1) % N;
            if self.count < N {
                self.count += 1;
            }
        }
        self.frame_start_us = now;
        self.started = true;
    }

    /// Starts timing a named sub-phase of the current frame.
    ///
    /// The elapsed duration is recorded into the phase's ring buffer when the
    /// returned [`PhaseGuard`] is dropped. Up to [`MAX_PHASES`] distinct phase
    /// names can be tracked; additional names are silently ignored.
    ///
    /// # Example
    ///
    /// ```rust
    /// use framepace::FrameTimer;
    ///
    /// let mut timer: FrameTimer = FrameTimer::new();
    /// timer.begin_frame();
    ///
    /// let _render = timer.phase("render");
    /// // ... render work ...
    /// drop(_render); // or let it fall out of scope
    /// ```
    #[must_use = "dropping PhaseGuard immediately records a near-zero duration; \
                  keep it alive for the duration of the phase"]
    pub fn phase(&mut self, name: &'static str) -> PhaseGuard<'_, N, T> {
        PhaseGuard::new(self, name)
    }
}

// ── Queries (no time-source bound needed) ────────────────────────────────────

impl<const N: usize, T> FrameTimer<N, T> {
    /// Number of frame samples currently in the window.
    ///
    /// Returns 0 until [`begin_frame`][Self::begin_frame] has been called at
    /// least twice (one call anchors the clock; the second records the first
    /// interval).
    #[must_use]
    pub const fn count(&self) -> usize {
        self.count
    }

    /// Computes statistics over the current window.
    ///
    /// Returns `None` until at least one sample has been recorded.
    #[must_use]
    pub fn stats(&self) -> Option<FrameStats> {
        FrameStats::compute(&self.samples, self.head, self.count)
    }

    /// Computes statistics for a named phase over its own window.
    ///
    /// Returns `None` if the phase has not been used yet or has no samples.
    #[must_use]
    pub fn phase_stats(&self, name: &'static str) -> Option<FrameStats> {
        self.phases[..self.phase_len]
            .iter()
            .find(|p| p.name == name)
            .and_then(|p| FrameStats::compute(&p.samples, p.head, p.count))
    }

    /// Returns an iterator over the raw frame-time samples, oldest first.
    ///
    /// Values are in microseconds. Useful for writing CSV output or asserting
    /// on percentile budgets in benchmark harnesses.
    #[must_use]
    pub fn samples_us(&self) -> impl DoubleEndedIterator<Item = u64> + '_ {
        let start = if self.count < N { 0 } else { self.head };
        let count = self.count;
        (0..count).map(move |i| self.samples[(start + i) % N])
    }

    /// Returns the names of all phases that have been registered so far.
    pub fn phase_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.phases[..self.phase_len].iter().map(|p| p.name)
    }

    /// Serializes all samples as a comma-separated string of microsecond
    /// values (oldest first), with no trailing newline.
    ///
    /// Useful for piping into a file for offline analysis.
    #[must_use]
    pub fn to_csv(&self) -> String {
        let parts: Vec<String> = self.samples_us().map(|us| us.to_string()).collect();
        parts.join(",")
    }

    /// Returns `true` if the most recent frame exceeded `budget_ms`.
    ///
    /// Returns `false` if no samples have been recorded yet.
    #[must_use]
    pub fn is_over_budget(&self, budget_ms: f64) -> bool {
        self.samples_us().next_back().is_some_and(|us| {
            #[allow(clippy::cast_precision_loss)]
            let ms = us as f64 / 1_000.0;
            ms > budget_ms
        })
    }

    /// Records a completed phase interval. Called from [`PhaseGuard::drop`].
    pub(crate) fn record_phase(&mut self, name: &'static str, elapsed_us: u64) {
        // Find an existing slot for this phase name.
        for phase in &mut self.phases[..self.phase_len] {
            if phase.name == name {
                phase.samples[phase.head] = elapsed_us;
                phase.head = (phase.head + 1) % N;
                if phase.count < N {
                    phase.count += 1;
                }
                return;
            }
        }
        // Register a new phase if there is room.
        if self.phase_len < MAX_PHASES {
            let slot = &mut self.phases[self.phase_len];
            slot.name = name;
            slot.samples[0] = elapsed_us;
            slot.head = 1 % N;
            slot.count = 1;
            self.phase_len += 1;
        }
    }
}

// ── Trait impls ──────────────────────────────────────────────────────────────

// Manual impls avoid adding spurious T: Clone / T: Debug bounds.

impl<const N: usize, T> Clone for FrameTimer<N, T> {
    fn clone(&self) -> Self {
        Self {
            samples: self.samples,
            head: self.head,
            count: self.count,
            frame_start_us: self.frame_start_us,
            started: self.started,
            phases: self.phases,
            phase_len: self.phase_len,
            _time: PhantomData,
        }
    }
}

impl<const N: usize, T> fmt::Debug for FrameTimer<N, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrameTimer")
            .field("sample_count", &self.count)
            .field("phase_count", &self.phase_len)
            .finish_non_exhaustive()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::StdTime;

    #[test]
    fn new_has_no_samples() {
        let timer: FrameTimer<64, StdTime> = FrameTimer::new();
        assert_eq!(timer.count(), 0);
        assert!(timer.stats().is_none());
    }

    #[test]
    fn single_begin_frame_still_no_samples() {
        let mut timer: FrameTimer<64, StdTime> = FrameTimer::new();
        timer.begin_frame();
        // First call anchors the clock; no interval recorded yet.
        assert_eq!(timer.count(), 0);
    }

    #[test]
    fn two_begin_frames_yields_one_sample() {
        let mut timer: FrameTimer<64, StdTime> = FrameTimer::new();
        timer.begin_frame();
        timer.begin_frame();
        assert_eq!(timer.count(), 1);
        assert!(timer.stats().is_some());
    }

    #[test]
    fn window_saturates_at_n() {
        let mut timer: FrameTimer<4, StdTime> = FrameTimer::new();
        for _ in 0..10 {
            timer.begin_frame();
        }
        assert_eq!(timer.count(), 4);
    }

    #[test]
    fn reset_clears_samples_but_keeps_phase_names() {
        let mut timer: FrameTimer<8, StdTime> = FrameTimer::new();
        timer.begin_frame();
        {
            let _p = timer.phase("render");
        }
        timer.begin_frame();
        timer.reset();

        assert_eq!(timer.count(), 0);
        assert!(timer.stats().is_none());
        // Phase name still registered, but no samples.
        assert!(timer.phase_stats("render").is_none());
        assert_eq!(timer.phase_names().next(), Some("render"));
    }

    #[test]
    fn to_csv_empty_when_no_samples() {
        let timer: FrameTimer<64, StdTime> = FrameTimer::new();
        assert_eq!(timer.to_csv(), "");
    }

    #[test]
    fn is_over_budget_false_when_no_samples() {
        let timer: FrameTimer<64, StdTime> = FrameTimer::new();
        assert!(!timer.is_over_budget(16.0));
    }

    #[test]
    fn phase_stats_none_for_unknown_phase() {
        let timer: FrameTimer<64, StdTime> = FrameTimer::new();
        assert!(timer.phase_stats("nonexistent").is_none());
    }

    #[test]
    fn clone_is_independent() {
        let mut timer: FrameTimer<8, StdTime> = FrameTimer::new();
        timer.begin_frame();
        timer.begin_frame();
        let clone = timer.clone();
        // Mutating the original does not affect the clone.
        timer.reset();
        assert_eq!(timer.count(), 0);
        assert_eq!(clone.count(), 1);
    }

    #[test]
    fn phase_guard_records_duration() {
        let mut timer: FrameTimer<8, StdTime> = FrameTimer::new();
        timer.begin_frame();
        {
            let _p = timer.phase("work");
        }
        timer.begin_frame();
        // Phase must have at least one sample now.
        assert!(timer.phase_stats("work").is_some());
    }

    #[test]
    fn extra_phases_beyond_max_are_silently_ignored() {
        let mut timer: FrameTimer<8, StdTime> = FrameTimer::new();
        timer.begin_frame();
        for i in 0..=MAX_PHASES {
            let name = Box::leak(format!("phase_{i}").into_boxed_str());
            drop(timer.phase(name));
        }
        // Only MAX_PHASES names should be registered.
        assert_eq!(timer.phase_names().count(), MAX_PHASES);
    }
}
