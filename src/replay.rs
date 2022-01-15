use crate::{
    exchange::reply::ExchangeToReplay,
    replay::request::ReplayToExchange,
    settlement::GetSettlementLag,
    types::{DateTime, Identifier, TimeSync},
    utils::{enum_dispatch, rand::Rng},
};

pub mod request;
pub mod concrete;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct ReplayAction<
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    pub datetime: DateTime,
    pub content: ReplayToExchange<ExchangeID, Symbol, Settlement>,
}

#[enum_dispatch]
pub trait Replay<ExchangeID, Symbol, Settlement>: TimeSync + Iterator<
    Item=ReplayAction<ExchangeID, Symbol, Settlement>
>
    where ExchangeID: Identifier,
          Symbol: Identifier,
          Settlement: GetSettlementLag
{
    fn handle_exchange_reply(
        &mut self,
        reply: ExchangeToReplay<Symbol, Settlement>,
        exchange_id: ExchangeID,
        rng: &mut impl Rng) -> Vec<ReplayAction<ExchangeID, Symbol, Settlement>>;
}