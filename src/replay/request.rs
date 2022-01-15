use crate::{
    order::{LimitOrderCancelRequest, LimitOrderPlacingRequest, MarketOrderPlacingRequest},
    settlement::GetSettlementLag,
    traded_pair::TradedPair,
    types::{Identifier, PriceStep},
};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ReplayToExchange<
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    pub exchange_id: ExchangeID,
    pub content: ReplayRequest<Symbol, Settlement>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ReplayRequest<Symbol: Identifier, Settlement: GetSettlementLag>
{
    ExchangeOpen,

    StartTrades(TradedPair<Symbol, Settlement>, PriceStep),

    CancelLimitOrder(LimitOrderCancelRequest<Symbol, Settlement>),

    PlaceMarketOrder(MarketOrderPlacingRequest<Symbol, Settlement>),

    PlaceLimitOrder(LimitOrderPlacingRequest<Symbol, Settlement>),

    BroadcastObStateToBrokers(TradedPair<Symbol, Settlement>),

    StopTrades(TradedPair<Symbol, Settlement>),

    ExchangeClosed,
}