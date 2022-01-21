use crate::{
    broker::reply::BrokerReply,
    settlement::GetSettlementLag,
    trader::request::TraderToBroker,
    types::{DateTime, Identifier, Named, TimeSync},
    utils::{queue::MessageReceiver, rand::Rng},
};

pub mod request;
pub mod concrete;
pub mod subscriptions;

pub struct TraderAction<
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    pub delay: u64,
    pub content: TraderActionKind<BrokerID, ExchangeID, Symbol, Settlement>,
}

pub enum TraderActionKind<
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    TraderToBroker(TraderToBroker<BrokerID, ExchangeID, Symbol, Settlement>),
    WakeUp,
}

pub trait Trader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>: TimeSync + Named<TraderID>
    where TraderID: Identifier,
          BrokerID: Identifier,
          ExchangeID: Identifier,
          Symbol: Identifier,
          Settlement: GetSettlementLag
{
    fn process_broker_reply<KernelMessage: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KernelMessage>,
        process_action: impl FnMut(TraderAction<BrokerID, ExchangeID, Symbol, Settlement>, &Self) -> KernelMessage,
        reply: BrokerReply<Symbol, Settlement>,
        broker_id: BrokerID,
        exchange_id: ExchangeID,
        event_dt: DateTime,
    );

    fn wakeup<KernelMessage: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KernelMessage>,
        process_action: impl FnMut(TraderAction<BrokerID, ExchangeID, Symbol, Settlement>, &Self) -> KernelMessage,
    );

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