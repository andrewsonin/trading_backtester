use {
    crate::{
        interface::message::{
            BrokerToReplay,
            ExchangeToReplay,
            ReplayToBroker,
            ReplayToExchange,
            ReplayToItself,
        },
        types::{DateTime, Id, TimeSync},
    },
    rand::Rng,
    std::fmt::Debug,
};

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

    fn wakeup(&mut self, scheduled_action: Self::R2R, rng: &mut impl Rng);

    fn handle_exchange_reply(
        &mut self,
        reply: Self::E2R,
        exchange_id: Self::ExchangeID,
        rng: &mut impl Rng,
    );

    fn handle_broker_reply(
        &mut self,
        reply: Self::B2R,
        broker_id: Self::BrokerID,
        rng: &mut impl Rng,
    );
}