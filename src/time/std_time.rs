//! [`StdTime`]: a [`TimeSource`][super::TimeSource] backed by
//! [`std::time::Instant`].

use std::sync::OnceLock;
use std::time::Instant;

use super::TimeSource;

/// A time source backed by [`std::time::Instant`].
///
/// This is the [`DefaultTime`][crate::DefaultTime] on all native targets.
///
/// On WebAssembly, `std::time::Instant` may not be available. Implement
/// [`TimeSource`][crate::TimeSource] using `performance.now()` and pass it as
/// the `T` parameter of [`FrameTimer`][crate::FrameTimer].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StdTime;

static EPOCH: OnceLock<Instant> = OnceLock::new();

impl TimeSource for StdTime {
    /// Returns microseconds elapsed since the first call to this function.
    fn now_us() -> u64 {
        let epoch = EPOCH.get_or_init(Instant::now);
        u64::try_from(epoch.elapsed().as_micros()).unwrap_or(u64::MAX)
    }
}
