use crate::{
    exchange::ExchangeToBroker,
    trader::TraderToBroker,
    types::{Agent, DateTime, Id, Named, TimeSync},
    utils::{queue::MessageReceiver, rand::Rng},
};

pub mod reply;
pub mod request;
pub mod concrete;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct BrokerAction<
    B2E: BrokerToExchange,
    B2T: BrokerToTrader,
    B2B: BrokerToItself
> {
    pub delay: u64,
    pub content: BrokerActionKind<B2E, B2T, B2B>,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum BrokerActionKind<
    B2E: BrokerToExchange,
    B2T: BrokerToTrader,
    B2B: BrokerToItself
> {
    BrokerToItself(B2B),
    BrokerToExchange(B2E),
    BrokerToTrader(B2T),
}

pub trait BrokerToItself: Ord {}

pub trait BrokerToExchange: Ord {
    type ExchangeID: Id;
    fn get_exchange_id(&self) -> Self::ExchangeID;
}

pub trait BrokerToTrader: Ord {
    type TraderID: Id;
    fn get_trader_id(&self) -> Self::TraderID;
}

pub trait Broker<BrokerID, TraderID, ExchangeID, E2B, T2B, B2E, B2T, B2B, SubCfg>:
TimeSync + Named<BrokerID> + Agent<Action=BrokerAction<B2E, B2T, B2B>>
    where BrokerID: Id,
          TraderID: Id,
          ExchangeID: Id,
          E2B: ExchangeToBroker<BrokerID=BrokerID>,
          T2B: TraderToBroker<BrokerID=BrokerID>,
          B2E: BrokerToExchange<ExchangeID=ExchangeID>,
          B2T: BrokerToTrader<TraderID=TraderID>,
          B2B: BrokerToItself
{
    fn wakeup<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(&Self, Self::Action, &mut RNG) -> KerMsg,
        scheduled_action: B2B,
        rng: &mut RNG,
    );

    fn process_trader_request<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(&Self, Self::Action, &mut RNG) -> KerMsg,
        request: T2B,
        trader_id: TraderID,
        rng: &mut RNG,
    );

    fn process_exchange_reply<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(&Self, Self::Action, &mut RNG) -> KerMsg,
        reply: E2B,
        exchange_id: ExchangeID,
        rng: &mut RNG,
    );

    fn broker_to_exchange_latency(
        &self,
        exchange_id: ExchangeID,
        event_dt: DateTime,
        rng: &mut impl Rng) -> u64;

    fn exchange_to_broker_latency(
        &self,
        exchange_id: ExchangeID,
        event_dt: DateTime,
        rng: &mut impl Rng) -> u64;

    fn upon_connection_to_exchange(&mut self, exchange_id: ExchangeID);

    fn register_trader(&mut self, trader_id: TraderID, sub_cfgs: impl IntoIterator<Item=SubCfg>);
}