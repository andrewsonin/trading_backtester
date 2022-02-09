/// Message delay in nanoseconds that indicates that there is no delay.
pub const NOW: u64 = 0;
/// Message delay in nanoseconds that indicates that there is 1ns delay.
pub const ONE_NANOSECOND: u64 = 1;
/// Message delay in nanoseconds that indicates that there is 1μs delay.
pub const ONE_MICROSECOND: u64 = 1_000 * ONE_NANOSECOND;
/// Message delay in nanoseconds that indicates that there is 1ms delay.
pub const ONE_MILLISECOND: u64 = 1_000 * ONE_MICROSECOND;
/// Message delay in nanoseconds that indicates that there is 1s delay.
pub const ONE_SECOND: u64 = 1_000 * ONE_MILLISECOND;
/// Message delay in nanoseconds that indicates that there is 1m delay.
pub const ONE_MINUTE: u64 = 60 * ONE_SECOND;
/// Message delay in nanoseconds that indicates that there is 1h delay.
pub const ONE_HOUR: u64 = 60 * ONE_MINUTE;
/// Message delay in nanoseconds that indicates that there is 24h delay.
pub const ONE_DAY: u64 = 24 * ONE_HOUR;