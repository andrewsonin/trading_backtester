use crate::{
    broker::{reply::BrokerToTrader, request::BrokerToExchange},
    exchange::reply::ExchangeToBrokerReply,
    traded_pair::TradedPair,
    trader::{request::TraderRequest, subscriptions::SubscriptionList},
    types::{DateTime, Identifier, Named, TimeSync},
    utils::{enum_dispatch, rand::Rng},
};

pub mod reply;
pub mod request;
pub mod concrete;

pub struct BrokerAction<
    TraderID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier
> {
    pub delay: u64,
    pub content: BrokerActionKind<TraderID, ExchangeID, Symbol>,
}

pub enum BrokerActionKind<
    TraderID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier
> {
    BrokerToTrader(BrokerToTrader<TraderID, ExchangeID, Symbol>),
    BrokerToExchange(BrokerToExchange<ExchangeID, Symbol>),
    WakeUp,
}

#[enum_dispatch]
pub trait Broker<BrokerID, TraderID, ExchangeID, Symbol>: TimeSync + Named<BrokerID>
    where BrokerID: Identifier,
          TraderID: Identifier,
          ExchangeID: Identifier,
          Symbol: Identifier
{
    fn process_trader_request(
        &mut self,
        request: TraderRequest<ExchangeID, Symbol>,
        trader_id: TraderID) -> Vec<BrokerAction<TraderID, ExchangeID, Symbol>>;

    fn process_exchange_reply(
        &mut self,
        reply: ExchangeToBrokerReply<Symbol>,
        exchange_id: ExchangeID,
        exchange_dt: DateTime) -> Vec<BrokerAction<TraderID, ExchangeID, Symbol>>;

    fn wakeup(&mut self) -> Vec<BrokerAction<TraderID, ExchangeID, Symbol>>;

    fn broker_to_exchange_latency(
        &self,
        exchange_id: ExchangeID,
        rng: &mut impl Rng,
        event_dt: DateTime) -> u64;

    fn exchange_to_broker_latency(
        &self,
        exchange_id: ExchangeID,
        rng: &mut impl Rng,
        event_dt: DateTime) -> u64;

    fn upon_connection_to_exchange(&mut self, exchange_id: ExchangeID);

    fn register_trader(
        &mut self,
        trader_id: TraderID,
        sub_cfgs: impl IntoIterator<Item=(ExchangeID, TradedPair<Symbol>, SubscriptionList)>,
    );
}