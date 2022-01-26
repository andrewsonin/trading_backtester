use crate::{
    exchange::ExchangeToBroker,
    latency::Latent,
    trader::TraderToBroker,
    types::{Agent, Id, Named, TimeSync},
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

pub trait Broker:
TimeSync
+ Named<Self::BrokerID>
+ Agent<Action=BrokerAction<Self::B2E, Self::B2T, Self::B2B>>
+ Latent<OuterID=Self::ExchangeID>
{
    type BrokerID: Id;
    type TraderID: Id;
    type ExchangeID: Id;

    type E2B: ExchangeToBroker<BrokerID=Self::BrokerID>;
    type T2B: TraderToBroker<BrokerID=Self::BrokerID>;
    type B2E: BrokerToExchange<ExchangeID=Self::ExchangeID>;
    type B2T: BrokerToTrader<TraderID=Self::TraderID>;
    type B2B: BrokerToItself;
    type SubCfg;

    fn wakeup<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(Self::LatencyGenerator, Self::Action, &mut RNG) -> KerMsg,
        scheduled_action: Self::B2B,
        rng: &mut RNG,
    );

    fn process_trader_request<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(Self::LatencyGenerator, Self::Action, &mut RNG) -> KerMsg,
        request: Self::T2B,
        trader_id: Self::TraderID,
        rng: &mut RNG,
    );

    fn process_exchange_reply<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(Self::LatencyGenerator, Self::Action, &mut RNG) -> KerMsg,
        reply: Self::E2B,
        exchange_id: Self::ExchangeID,
        rng: &mut RNG,
    );

    fn upon_connection_to_exchange(&mut self, exchange_id: Self::ExchangeID);

    fn register_trader(
        &mut self,
        trader_id: Self::TraderID,
        sub_cfgs: impl IntoIterator<Item=Self::SubCfg>);
}