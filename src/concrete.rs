/// Concrete implementors of the [`Broker`](crate::interface::broker::Broker).
pub mod broker;
/// Concrete implementors of the [`Exchange`](crate::interface::exchange::Exchange).
pub mod exchange;
/// Input parsers and initializer utilities.
pub mod input;
/// Concrete implementors related to the [`latency`](crate::interface::latency).
pub mod latency;
/// Concrete implementors related to the [`message_protocol`](crate::interface::message).
pub mod message_protocol;
/// Order types for the [`message_protocol`].
pub mod order;
/// Simple order book struct.
pub mod order_book;
/// Concrete implementors of the [`Replay`](crate::interface::replay::Replay).
pub mod replay;
/// Traded pair and financial instruments.
pub mod traded_pair;
/// Concrete implementors of the [`Trader`](crate::interface::trader::Trader).
pub mod trader;
/// Auxiliary types and traits.
pub mod types;