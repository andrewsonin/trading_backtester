use crate::{
    broker::BrokerToTrader,
    types::{Agent, DateTime, Id, Named, TimeSync},
    utils::{queue::MessageReceiver, rand::Rng},
};

pub mod request;
pub mod concrete;
pub mod subscriptions;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct TraderAction<T2B: TraderToBroker, T2T: TraderToItself> {
    pub delay: u64,
    pub content: TraderActionKind<T2B, T2T>,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum TraderActionKind<T2B: TraderToBroker, T2T: TraderToItself> {
    TraderToItself(T2T),
    TraderToBroker(T2B),
}

pub trait TraderToItself: Ord {}

pub trait TraderToBroker: Ord {
    type BrokerID: Id;
    fn get_broker_id(&self) -> Self::BrokerID;
}

pub trait Trader:
TimeSync + Named<Self::TraderID> + Agent<Action=TraderAction<Self::T2B, Self::T2T>>
{
    type TraderID: Id;
    type BrokerID: Id;
    type B2T: BrokerToTrader<TraderID=Self::TraderID>;
    type T2T: TraderToItself;
    type T2B: TraderToBroker<BrokerID=Self::BrokerID>;

    fn wakeup<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(&Self, Self::Action, &mut RNG) -> KerMsg,
        scheduled_action: Self::T2T,
        rng: &mut RNG,
    );

    fn process_broker_reply<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(&Self, Self::Action, &mut RNG) -> KerMsg,
        reply: Self::B2T,
        broker_id: Self::BrokerID,
        rng: &mut RNG,
    );

    fn broker_to_trader_latency(
        &self,
        broker_id: Self::BrokerID,
        event_dt: DateTime,
        rng: &mut impl Rng) -> u64;

    fn trader_to_broker_latency(
        &self,
        broker_id: Self::BrokerID,
        event_dt: DateTime,
        rng: &mut impl Rng) -> u64;

    fn upon_register_at_broker(&mut self, broker_id: Self::BrokerID);
}