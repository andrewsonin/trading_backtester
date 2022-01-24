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

pub trait
Replay<ExchangeID, E2R, R2R, R2E>: TimeSync + Iterator<Item=ReplayAction<R2R, R2E>>
    where ExchangeID: Id,
          E2R: ExchangeToReplay,
          R2R: ReplayToItself,
          R2E: ReplayToExchange<ExchangeID=ExchangeID>
{
    fn wakeup<KernelMessage: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KernelMessage>,
        process_action: impl Fn(Self::Item) -> KernelMessage,
        scheduled_action: R2R,
        rng: &mut impl Rng,
    );

    fn handle_exchange_reply<KernelMessage: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KernelMessage>,
        process_action: impl Fn(Self::Item) -> KernelMessage,
        reply: E2R,
        exchange_id: ExchangeID,
        rng: &mut impl Rng,
    );
}