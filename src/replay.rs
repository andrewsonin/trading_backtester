use {
    crate::{
        exchange::ExchangeToReplay,
        types::{DateTime, Id, TimeSync},
        utils::{queue::MessageReceiver, rand::Rng},
    },
    std::fmt::Debug,
};

pub mod request;
pub mod concrete;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ReplayAction<R2R: ReplayToItself, R2E: ReplayToExchange> {
    pub datetime: DateTime,
    pub content: ReplayActionKind<R2R, R2E>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ReplayActionKind<R2R: ReplayToItself, R2E: ReplayToExchange> {
    ReplayToItself(R2R),
    ReplayToExchange(R2E),
}

pub trait ReplayToItself: Debug + Ord {}

pub trait ReplayToExchange: Debug + Ord {
    type ExchangeID: Id;
    fn get_exchange_id(&self) -> Self::ExchangeID;
}

pub trait Replay: TimeSync + Iterator<Item=ReplayAction<Self::R2R, Self::R2E>>
{
    type ExchangeID: Id;
    type E2R: ExchangeToReplay;
    type R2R: ReplayToItself;
    type R2E: ReplayToExchange<ExchangeID=Self::ExchangeID>;

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
}