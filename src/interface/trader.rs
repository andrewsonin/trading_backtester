use {
    crate::{
        interface::{latency::Latent, message::{BrokerToTrader, TraderToBroker, TraderToItself}},
        kernel::LatentActionProcessor,
        types::{Agent, Id, Named, TimeSync},
        utils::queue::MessageReceiver,
    },
    rand::Rng,
};

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

pub trait Trader
    where Self: TimeSync,
          Self: Latent<OuterID=Self::BrokerID>,
          Self: Named<Self::TraderID>,
          Self: Agent<Action=TraderAction<Self::T2B, Self::T2T>>
{
    type TraderID: Id;
    type BrokerID: Id;

    type B2T: BrokerToTrader<TraderID=Self::TraderID>;
    type T2T: TraderToItself;
    type T2B: TraderToBroker<BrokerID=Self::BrokerID>;

    fn wakeup<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        action_processor: impl LatentActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
        scheduled_action: Self::T2T,
        rng: &mut RNG,
    );

    fn process_broker_reply<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        action_processor: impl LatentActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
        reply: Self::B2T,
        broker_id: Self::BrokerID,
        rng: &mut RNG,
    );

    fn upon_register_at_broker(&mut self, broker_id: Self::BrokerID);
}