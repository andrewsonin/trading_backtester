use crate::types::{Id, NeverType, Nothing};

/// Indicate that the type is the [`Trader`](crate::interface::trader::Trader)-to-itself message.
pub trait TraderToItself: Ord {}

/// Indicate that the type is the
/// [`Trader`](crate::interface::trader::Trader)-to-[`Broker`](crate::interface::broker::Broker)
/// message.
pub trait TraderToBroker: Ord {
    type BrokerID: Id;
    fn get_broker_id(&self) -> Self::BrokerID;
}

/// Indicate that the type is the [`Broker`](crate::interface::broker::Broker)-to-itself message.
pub trait BrokerToItself: Ord {}

/// Indicate that the type is the
/// [`Broker`](crate::interface::broker::Broker)-to-[`Replay`](crate::interface::replay::Replay)
/// message.
pub trait BrokerToReplay: Ord {}

/// Indicate that the type is the
/// [`Broker`](crate::interface::broker::Broker)-to-[`Exchange`](
/// crate::interface::exchange::Exchange) message.
pub trait BrokerToExchange: Ord {
    type ExchangeID: Id;
    fn get_exchange_id(&self) -> Self::ExchangeID;
}

/// Indicate that the type is the
/// [`Broker`](crate::interface::broker::Broker)-to-[`Trader`](crate::interface::trader::Trader)
/// message.
pub trait BrokerToTrader: Ord {
    type TraderID: Id;
    fn get_trader_id(&self) -> Self::TraderID;
}

/// Indicate that the type is the
/// [`Exchange`](crate::interface::exchange::Exchange)-to-itself message.
pub trait ExchangeToItself: Ord {}

/// Indicate that the type is the
/// [`Exchange`](crate::interface::exchange::Exchange)-to-[`Replay`](
/// crate::interface::replay::Replay) message.
pub trait ExchangeToReplay: Ord {}

/// Indicate that the type is the
/// [`Exchange`](crate::interface::exchange::Exchange)-to-[`Broker`](
/// crate::interface::broker::Broker) message.
pub trait ExchangeToBroker: Ord {
    type BrokerID: Id;
    fn get_broker_id(&self) -> Self::BrokerID;
}

/// Indicate that the type is the
/// [`Replay`](crate::interface::replay::Replay)-to-itself message.
pub trait ReplayToItself: Ord {}

/// Indicate that the type is the
/// [`Replay`](crate::interface::replay::Replay)-to-[`Exchange`](
/// crate::interface::exchange::Exchange) message.
pub trait ReplayToExchange: Ord {
    type ExchangeID: Id;
    fn get_exchange_id(&self) -> Self::ExchangeID;
}

/// Indicate that the type is the
/// [`Replay`](crate::interface::replay::Replay)-to-[`Broker`]( crate::interface::broker::Broker)
/// message.
pub trait ReplayToBroker: Ord {
    type BrokerID: Id;
    fn get_broker_id(&self) -> Self::BrokerID;
}


impl TraderToItself for Nothing {}

impl BrokerToReplay for Nothing {}

impl BrokerToItself for Nothing {}

impl ReplayToItself for Nothing {}

impl ExchangeToItself for Nothing {}

impl ExchangeToReplay for Nothing {}


impl<BrokerID: Id> TraderToBroker for NeverType<BrokerID> {
    type BrokerID = BrokerID;
    fn get_broker_id(&self) -> BrokerID {
        unreachable!("Does not contain BrokerID")
    }
}

impl<ExchangeID: Id> BrokerToExchange for NeverType<ExchangeID> {
    type ExchangeID = ExchangeID;
    fn get_exchange_id(&self) -> Self::ExchangeID {
        unreachable!("Does not contain ExchangeID")
    }
}

impl<TraderID: Id> BrokerToTrader for NeverType<TraderID> {
    type TraderID = TraderID;
    fn get_trader_id(&self) -> Self::TraderID {
        unreachable!("Does not contain TraderID")
    }
}

impl<BrokerID: Id> ExchangeToBroker for NeverType<BrokerID> {
    type BrokerID = BrokerID;
    fn get_broker_id(&self) -> Self::BrokerID {
        unreachable!("Does not contain BrokerID")
    }
}

impl<ExchangeID: Id> ReplayToExchange for NeverType<ExchangeID> {
    type ExchangeID = ExchangeID;
    fn get_exchange_id(&self) -> ExchangeID {
        unreachable!("Does not contain ExchangeID")
    }
}

impl<BrokerID: Id> ReplayToBroker for NeverType<BrokerID> {
    type BrokerID = BrokerID;
    fn get_broker_id(&self) -> BrokerID {
        unreachable!("Does not contain BrokerID")
    }
}