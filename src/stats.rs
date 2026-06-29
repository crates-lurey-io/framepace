//! [`FrameStats`] and the internal computation that produces it.

/// Frame-timing statistics computed over the current sample window.
///
/// Produced by [`FrameTimer::stats`][crate::FrameTimer::stats] and
/// [`FrameTimer::phase_stats`][crate::FrameTimer::phase_stats].
/// All time fields are in milliseconds.
///
/// # Example
///
/// ```rust
/// use framepace::FrameTimer;
///
/// let mut timer: FrameTimer = FrameTimer::new();
/// timer.begin_frame();
///
/// if let Some(s) = timer.stats() {
///     assert!(s.fps >= 0.0);
///     assert!(s.p95_ms >= s.p50_ms);
///     assert!(s.max_ms >= s.min_ms);
/// }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameStats {
    /// Number of samples in the window (at most the window size `N`).
    pub count: usize,
    /// Frames per second derived from the mean frame time.
    pub fps: f64,
    /// Mean frame time in milliseconds.
    pub avg_ms: f64,
    /// Shortest frame in the window, in milliseconds.
    pub min_ms: f64,
    /// Longest frame in the window, in milliseconds.
    pub max_ms: f64,
    /// 50th-percentile frame time (median), in milliseconds.
    pub p50_ms: f64,
    /// 95th-percentile frame time, in milliseconds.
    pub p95_ms: f64,
    /// 99th-percentile frame time, in milliseconds.
    pub p99_ms: f64,
}

impl FrameStats {
    /// Computes statistics from a ring-buffer snapshot.
    ///
    /// `head` is the write-head index into `samples`; `count` is the number of
    /// valid samples (≤ N). Returns `None` if `count` is zero.
    pub(crate) fn compute<const N: usize>(
        samples: &[u64; N],
        head: usize,
        count: usize,
    ) -> Option<Self> {
        if count == 0 {
            return None;
        }

        // When the buffer is not yet full, samples start at index 0.
        // When full, the oldest sample is at the write-head.
        let start = if count < N { 0 } else { head };

        let mut sum = 0_u64;
        let mut min_us = u64::MAX;
        let mut max_us = 0_u64;

        // Stack-allocated sorted copy for percentile computation.
        // Sorting N u64s (≤ 8 KB for N=1024) is negligible on any call path
        // that would invoke this function.
        let mut sorted = [0_u64; N];

        for i in 0..count {
            let v = samples[(start + i) % N];
            sorted[i] = v;
            sum = sum.saturating_add(v);
            min_us = min_us.min(v);
            max_us = max_us.max(v);
        }

        let valid = &mut sorted[..count];
        valid.sort_unstable();

        #[allow(clippy::cast_precision_loss)]
        let mean_us = sum as f64 / count as f64;

        Some(Self {
            count,
            fps: if mean_us > 0.0 {
                1_000_000.0 / mean_us
            } else {
                f64::INFINITY
            },
            avg_ms: mean_us / 1_000.0,
            #[allow(clippy::cast_precision_loss)]
            min_ms: min_us as f64 / 1_000.0,
            #[allow(clippy::cast_precision_loss)]
            max_ms: max_us as f64 / 1_000.0,
            p50_ms: percentile_ms(valid, 50.0),
            p95_ms: percentile_ms(valid, 95.0),
            p99_ms: percentile_ms(valid, 99.0),
        })
    }
}

/// Linear-interpolation percentile over a pre-sorted slice of microsecond
/// durations. Returns the result in milliseconds.
fn percentile_ms(sorted_us: &[u64], pct: f64) -> f64 {
    let n = sorted_us.len();
    debug_assert!(n > 0, "called with empty slice");

    if n == 1 {
        #[allow(clippy::cast_precision_loss)]
        return sorted_us[0] as f64 / 1_000.0;
    }

    #[allow(clippy::cast_precision_loss)]
    let rank = pct / 100.0 * (n - 1) as f64;
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let lo = rank as usize;
    let hi = (lo + 1).min(n - 1);
    #[allow(clippy::cast_precision_loss)]
    let frac = rank - lo as f64;

    #[allow(clippy::cast_precision_loss)]
    let result = frac.mul_add(
        sorted_us[hi] as f64 - sorted_us[lo] as f64,
        sorted_us[lo] as f64,
    ) / 1_000.0;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_sample() {
        let samples = [16_000_u64; 1];
        let stats = FrameStats::compute(&samples, 0, 1).unwrap();
        assert!((stats.avg_ms - 16.0).abs() < f64::EPSILON);
        assert!((stats.p50_ms - 16.0).abs() < f64::EPSILON);
        assert!((stats.p99_ms - 16.0).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_returns_none() {
        let samples = [0_u64; 4];
        assert!(FrameStats::compute(&samples, 0, 0).is_none());
    }

    #[test]
    fn min_max_correct() {
        let samples = [1_000_u64, 2_000, 3_000, 4_000];
        let stats = FrameStats::compute(&samples, 0, 4).unwrap();
        assert!((stats.min_ms - 1.0).abs() < f64::EPSILON);
        assert!((stats.max_ms - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ring_wrap_uses_correct_window() {
        // Fill a 4-slot buffer with [10, 20, 30, 40] then overwrite the first
        // two slots so the live window is [30, 40, 50, 60].
        let mut samples = [10_000_u64, 20_000, 30_000, 40_000];
        samples[0] = 50_000;
        samples[1] = 60_000;
        // head=2 (next write position), count=4 (full)
        let stats = FrameStats::compute(&samples, 2, 4).unwrap();
        // Oldest-first order: 30, 40, 50, 60 → avg = 45 ms
        assert!((stats.avg_ms - 45.0).abs() < 0.001);
    }

    #[test]
    fn percentile_monotone() {
        let samples = [
            5_000_u64, 10_000, 15_000, 20_000, 25_000, 30_000, 35_000, 40_000,
        ];
        let stats = FrameStats::compute(&samples, 0, 8).unwrap();
        assert!(stats.p50_ms <= stats.p95_ms);
        assert!(stats.p95_ms <= stats.p99_ms);
    }
}
