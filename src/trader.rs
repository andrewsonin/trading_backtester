use {
    crate::{
        broker::BrokerToTrader,
        kernel::ActionProcessor,
        latency::Latent,
        types::{Agent, Id, Named, TimeSync},
        utils::queue::MessageReceiver,
    },
    rand::Rng,
};

pub mod concrete;
pub mod request;
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
TimeSync + Latent<OuterID=Self::BrokerID> + Named<Self::TraderID> + Agent<
    Action=TraderAction<Self::T2B, Self::T2T>
> {
    type TraderID: Id;
    type BrokerID: Id;
    type B2T: BrokerToTrader<TraderID=Self::TraderID>;
    type T2T: TraderToItself;
    type T2B: TraderToBroker<BrokerID=Self::BrokerID>;

    fn wakeup<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        action_processor: impl ActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
        scheduled_action: Self::T2T,
        rng: &mut RNG,
    );

    fn process_broker_reply<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        action_processor: impl ActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
        reply: Self::B2T,
        broker_id: Self::BrokerID,
        rng: &mut RNG,
    );

    fn upon_register_at_broker(&mut self, broker_id: Self::BrokerID);
}