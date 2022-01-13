use crate::{
    order::{LimitOrderCancelRequest, LimitOrderPlacingRequest, MarketOrderPlacingRequest},
    traded_pair::TradedPair,
    types::{Identifier, PriceStep},
};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ReplayToExchange<
    ExchangeID: Identifier,
    Symbol: Identifier
> {
    pub exchange_id: ExchangeID,
    pub content: ReplayRequest<Symbol>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ReplayRequest<Symbol: Identifier>
{
    ExchangeOpen,

    StartTrades(TradedPair<Symbol>, PriceStep),

    CancelLimitOrder(LimitOrderCancelRequest<Symbol>),

    PlaceMarketOrder(MarketOrderPlacingRequest<Symbol>),

    PlaceLimitOrder(LimitOrderPlacingRequest<Symbol>),

    BroadcastObStateToBrokers(TradedPair<Symbol>),

    StopTrades(TradedPair<Symbol>),

    ExchangeClosed,
}