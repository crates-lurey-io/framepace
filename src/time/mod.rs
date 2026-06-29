//! Time source abstraction and the built-in [`StdTime`] implementation.

mod std_time;
pub use std_time::StdTime;

/// Returns the current time as microseconds since an arbitrary epoch.
///
/// The absolute value is unspecified; only differences between successive
/// calls are meaningful. Implementations must be monotonically non-decreasing.
///
/// # Implementing for a custom clock
///
/// ```rust
/// use framepace::TimeSource;
///
/// /// A time source backed by the `quanta` crate (illustrative).
/// struct QuantaTime;
///
/// impl TimeSource for QuantaTime {
///     fn now_us() -> u64 {
///         // quanta::Instant::recent().as_u64() / 1_000
///         0 // placeholder
///     }
/// }
/// ```
pub trait TimeSource {
    /// Returns the current time in microseconds.
    fn now_us() -> u64;
}

/// The default time source on all supported targets.
///
/// Currently an alias for [`StdTime`], which is backed by
/// [`std::time::Instant`]. On WebAssembly targets where `Instant` is
/// unavailable, implement [`TimeSource`] with `performance.now()` and pass
/// your type as the `T` parameter of [`FrameTimer`][crate::FrameTimer].
pub type DefaultTime = StdTime;
