use crate::{
    broker::{reply::BrokerToTrader, request::BrokerToExchange},
    exchange::reply::ExchangeToBrokerReply,
    settlement::GetSettlementLag,
    trader::{request::TraderRequest, subscriptions::SubscriptionConfig},
    types::{DateTime, Identifier, Named, TimeSync},
    utils::{queue::MessageReceiver, rand::Rng},
};

pub mod reply;
pub mod request;
pub mod concrete;

pub struct BrokerAction<
    TraderID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    pub delay: u64,
    pub content: BrokerActionKind<TraderID, ExchangeID, Symbol, Settlement>,
}

pub enum BrokerActionKind<
    TraderID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    BrokerToTrader(BrokerToTrader<TraderID, ExchangeID, Symbol, Settlement>),
    BrokerToExchange(BrokerToExchange<ExchangeID, Symbol, Settlement>),
    WakeUp,
}

pub trait Broker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>: TimeSync + Named<BrokerID>
    where BrokerID: Identifier,
          TraderID: Identifier,
          ExchangeID: Identifier,
          Symbol: Identifier,
          Settlement: GetSettlementLag
{
    fn process_trader_request<KernelMessage: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KernelMessage>,
        process_action: impl FnMut(BrokerAction<TraderID, ExchangeID, Symbol, Settlement>, &Self) -> KernelMessage,
        request: TraderRequest<ExchangeID, Symbol, Settlement>,
        trader_id: TraderID,
    );

    fn process_exchange_reply<KernelMessage: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KernelMessage>,
        process_action: impl FnMut(BrokerAction<TraderID, ExchangeID, Symbol, Settlement>, &Self) -> KernelMessage,
        reply: ExchangeToBrokerReply<Symbol, Settlement>,
        exchange_id: ExchangeID,
        exchange_dt: DateTime,
    );

    fn wakeup<KernelMessage: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KernelMessage>,
        process_action: impl FnMut(BrokerAction<TraderID, ExchangeID, Symbol, Settlement>, &Self) -> KernelMessage,
    );

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
        sub_cfgs: impl IntoIterator<Item=SubscriptionConfig<ExchangeID, Symbol, Settlement>>,
    );
}