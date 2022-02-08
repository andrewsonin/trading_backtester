/// Everything related to [`Broker`](broker::Broker).
pub mod broker;
/// Everything related to [`Exchange`](exchange::Exchange).
pub mod exchange;
/// Latency-related traits
/// that are used by [`Trader`](trader::Trader) and [`Broker`](broker::Broker).
pub mod latency;
/// Everything related to [`Replay`](replay::Replay).
pub mod replay;
/// Everything related to [`Trader`](trader::Trader).
pub mod trader;
/// Inter-agent message protocol traits.
pub mod message;