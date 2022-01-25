use crate::{
    broker::BrokerToExchange,
    replay::ReplayToExchange,
    types::{Agent, Id, Named, TimeSync},
    utils::{queue::MessageReceiver, rand::Rng},
};

pub mod reply;
pub mod concrete;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct ExchangeAction<
    E2R: ExchangeToReplay,
    E2B: ExchangeToBroker,
    E2E: ExchangeToItself
> {
    pub delay: u64,
    pub content: ExchangeActionKind<E2R, E2B, E2E>,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum ExchangeActionKind<
    E2R: ExchangeToReplay,
    E2B: ExchangeToBroker,
    E2E: ExchangeToItself
> {
    ExchangeToItself(E2E),
    ExchangeToReplay(E2R),
    ExchangeToBroker(E2B),
}

pub trait ExchangeToItself: Ord {}

pub trait ExchangeToReplay: Ord {}

pub trait ExchangeToBroker: Ord {
    type BrokerID: Id;
    fn get_broker_id(&self) -> Self::BrokerID;
}

pub trait Exchange<ExchangeID, BrokerID, R2E, B2E, E2R, E2B, E2E>:
TimeSync + Named<ExchangeID> + Agent<Action=ExchangeAction<E2R, E2B, E2E>>
    where ExchangeID: Id,
          BrokerID: Id,
          R2E: ReplayToExchange<ExchangeID=ExchangeID>,
          B2E: BrokerToExchange<ExchangeID=ExchangeID>,
          E2R: ExchangeToReplay,
          E2B: ExchangeToBroker<BrokerID=BrokerID>,
          E2E: ExchangeToItself,
{
    fn wakeup<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
        scheduled_action: E2E,
        rng: &mut RNG,
    );

    fn process_broker_request<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
        request: B2E,
        broker_id: BrokerID,
        rng: &mut RNG,
    );

    fn process_replay_request<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
        request: R2E,
        rng: &mut RNG,
    );

    fn connect_broker(&mut self, broker: BrokerID);
}