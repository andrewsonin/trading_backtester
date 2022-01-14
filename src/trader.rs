use crate::{
    broker::reply::BrokerReply,
    trader::request::TraderToBroker,
    types::{DateTime, Identifier, Named, TimeSync},
    utils::{enum_dispatch, rand::Rng},
};

pub mod request;
pub mod concrete;
pub mod subscriptions;

pub struct TraderAction<
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier
> {
    pub delay: u64,
    pub content: TraderActionKind<BrokerID, ExchangeID, Symbol>,
}

pub enum TraderActionKind<
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier
> {
    TraderToBroker(TraderToBroker<BrokerID, ExchangeID, Symbol>),
    WakeUp,
}

#[enum_dispatch]
pub trait Trader<TraderID, BrokerID, ExchangeID, Symbol>: TimeSync + Named<TraderID>
    where TraderID: Identifier,
          BrokerID: Identifier,
          ExchangeID: Identifier,
          Symbol: Identifier
{
    fn process_broker_reply(
        &mut self,
        reply: BrokerReply<Symbol>,
        broker_id: BrokerID,
        exchange_id: ExchangeID,
        event_dt: DateTime) -> Vec<TraderAction<BrokerID, ExchangeID, Symbol>>;

    fn wakeup(&mut self) -> Vec<TraderAction<BrokerID, ExchangeID, Symbol>>;

    fn broker_to_trader_latency(
        &self,
        broker_id: BrokerID,
        rng: &mut impl Rng,
        event_dt: DateTime) -> u64;

    fn trader_to_broker_latency(
        &self,
        broker_id: BrokerID,
        rng: &mut impl Rng,
        event_dt: DateTime) -> u64;

    fn upon_register_at_broker(&mut self, broker_id: BrokerID);
}