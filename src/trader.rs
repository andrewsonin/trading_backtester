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

pub trait Trader<TraderID, BrokerID, B2T, T2B, T2T>:
TimeSync + Named<TraderID> + Agent<Action=TraderAction<T2B, T2T>>
    where TraderID: Id,
          BrokerID: Id,
          B2T: BrokerToTrader<TraderID=TraderID>,
          T2T: TraderToItself,
          T2B: TraderToBroker<BrokerID=BrokerID>
{
    fn wakeup<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(&Self, Self::Action, &mut RNG) -> KerMsg,
        scheduled_action: T2T,
        rng: &mut RNG,
    );

    fn process_broker_reply<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(&Self, Self::Action, &mut RNG) -> KerMsg,
        reply: B2T,
        broker_id: BrokerID,
        rng: &mut RNG,
    );

    fn broker_to_trader_latency(
        &self,
        broker_id: BrokerID,
        event_dt: DateTime,
        rng: &mut impl Rng) -> u64;

    fn trader_to_broker_latency(
        &self,
        broker_id: BrokerID,
        event_dt: DateTime,
        rng: &mut impl Rng) -> u64;

    fn upon_register_at_broker(&mut self, broker_id: BrokerID);
}