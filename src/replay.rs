use crate::{
    exchange::reply::ExchangeToReplay,
    replay::request::ReplayToExchange,
    types::{DateTime, Identifier, TimeSync},
    utils::{enum_dispatch, rand::Rng},
};

pub mod request;
pub mod concrete;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct ReplayAction<
    ExchangeID: Identifier,
    Symbol: Identifier
> {
    pub datetime: DateTime,
    pub content: ReplayToExchange<ExchangeID, Symbol>,
}

#[enum_dispatch]
pub trait Replay<ExchangeID, Symbol>: TimeSync + Iterator<Item=ReplayAction<ExchangeID, Symbol>>
    where ExchangeID: Identifier,
          Symbol: Identifier
{
    fn handle_exchange_reply(
        &mut self,
        reply: ExchangeToReplay<Symbol>,
        exchange_id: ExchangeID,
        rng: &mut impl Rng) -> Vec<ReplayAction<ExchangeID, Symbol>>;
}