use crate::types::{Id, NeverType, Nothing};

pub trait TraderToItself: Ord {}

pub trait TraderToBroker: Ord {
    type BrokerID: Id;
    fn get_broker_id(&self) -> Self::BrokerID;
}

/// Trait indicating that the type is the [`Broker`]-to-itself message.
pub trait BrokerToItself: Ord {}

/// Trait indicating that the type is the [`Broker`]-to-[`Replay`] message.
pub trait BrokerToReplay: Ord {}

/// Trait indicating that the type
/// is the [`Broker`]-to-[`Exchange`](crate::exchange::Exchange) message.
pub trait BrokerToExchange: Ord {
    type ExchangeID: Id;
    fn get_exchange_id(&self) -> Self::ExchangeID;
}

/// Trait indicating that the type is the [`Broker`]-to-[`Trader`](crate::trader::Trader) message.
pub trait BrokerToTrader: Ord {
    type TraderID: Id;
    fn get_trader_id(&self) -> Self::TraderID;
}

pub trait ExchangeToItself: Ord {}

pub trait ExchangeToReplay: Ord {}

pub trait ExchangeToBroker: Ord {
    type BrokerID: Id;
    fn get_broker_id(&self) -> Self::BrokerID;
}

pub trait ReplayToItself: Ord {}

pub trait ReplayToExchange: Ord {
    type ExchangeID: Id;
    fn get_exchange_id(&self) -> Self::ExchangeID;
}

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