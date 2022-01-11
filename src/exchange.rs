use crate::{
    broker::request::BrokerRequest,
    exchange::reply::{ExchangeToBroker, ExchangeToReplay},
    replay::request::ReplayRequest,
    types::{Identifier, Named, TimeSync},
    utils::enum_dispatch,
};

pub mod reply;
pub mod concrete;

pub struct ExchangeAction<
    BrokerID: Identifier,
    Symbol: Identifier
> {
    pub delay: u64,
    pub content: ExchangeActionKind<BrokerID, Symbol>,
}

pub enum ExchangeActionKind<
    BrokerID: Identifier,
    Symbol: Identifier
> {
    ExchangeToBroker(ExchangeToBroker<BrokerID, Symbol>),
    ExchangeToReplay(ExchangeToReplay<Symbol>),
}

#[enum_dispatch]
pub trait Exchange<ExchangeID, BrokerID, Symbol>: TimeSync + Named<ExchangeID>
    where ExchangeID: Identifier,
          BrokerID: Identifier,
          Symbol: Identifier
{
    fn process_broker_request(
        &mut self,
        request: BrokerRequest<Symbol>,
        broker_id: BrokerID) -> Vec<ExchangeAction<BrokerID, Symbol>>;

    fn process_replay_request(
        &mut self,
        request: ReplayRequest<Symbol>) -> Vec<ExchangeAction<BrokerID, Symbol>>;

    fn connect_broker(&mut self, broker: BrokerID);
}