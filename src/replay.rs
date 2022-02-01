use {
    crate::{
        broker::BrokerToReplay,
        exchange::ExchangeToReplay,
        types::{DateTime, Id, TimeSync},
        utils::queue::MessageReceiver,
    },
    rand::Rng,
    std::fmt::Debug,
};

pub mod concrete;
pub mod request;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ReplayAction<R2R: ReplayToItself, R2E: ReplayToExchange, R2B: ReplayToBroker> {
    pub datetime: DateTime,
    pub content: ReplayActionKind<R2R, R2E, R2B>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ReplayActionKind<R2R: ReplayToItself, R2E: ReplayToExchange, R2B: ReplayToBroker> {
    ReplayToItself(R2R),
    ReplayToExchange(R2E),
    ReplayToBroker(R2B),
}

pub trait ReplayToItself: Debug + Ord {}

pub trait ReplayToExchange: Debug + Ord {
    type ExchangeID: Id;
    fn get_exchange_id(&self) -> Self::ExchangeID;
}

pub trait ReplayToBroker: Debug + Ord {
    type BrokerID: Id;
    fn get_broker_id(&self) -> Self::BrokerID;
}

pub trait Replay
    where Self: TimeSync,
          Self: Iterator<Item=ReplayAction<Self::R2R, Self::R2E, Self::R2B>>
{
    type ExchangeID: Id;
    type BrokerID: Id;

    type E2R: ExchangeToReplay;
    type B2R: BrokerToReplay;
    type R2R: ReplayToItself;
    type R2E: ReplayToExchange<ExchangeID=Self::ExchangeID>;
    type R2B: ReplayToBroker<BrokerID=Self::BrokerID>;

    fn wakeup<KerMsg: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl Fn(Self::Item) -> KerMsg,
        scheduled_action: Self::R2R,
        rng: &mut impl Rng,
    );

    fn handle_exchange_reply<KerMsg: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl Fn(Self::Item) -> KerMsg,
        reply: Self::E2R,
        exchange_id: Self::ExchangeID,
        rng: &mut impl Rng,
    );

    fn handle_broker_reply<KerMsg: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl Fn(Self::Item) -> KerMsg,
        reply: Self::B2R,
        broker_id: Self::BrokerID,
        rng: &mut impl Rng,
    );
}