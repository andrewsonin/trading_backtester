/// Everything related to the [`Broker`](broker::Broker).
pub mod broker;
/// Everything related to the [`Exchange`](exchange::Exchange).
pub mod exchange;
/// Latency-related traits used by [`Trader`](trader::Trader) and [`Broker`](broker::Broker).
pub mod latency;
/// Everything related to the [`Replay`](replay::Replay).
pub mod replay;
/// Everything related to the [`Trader`](trader::Trader).
pub mod trader;
/// Inter-agent message protocol traits.
pub mod message;