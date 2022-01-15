use crate::{
    order::{LimitOrderCancelRequest, LimitOrderPlacingRequest, MarketOrderPlacingRequest},
    settlement::GetSettlementLag,
    types::Identifier,
};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TraderToBroker<
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    pub broker_id: BrokerID,
    pub content: TraderRequest<ExchangeID, Symbol, Settlement>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TraderRequest<
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    CancelLimitOrder(LimitOrderCancelRequest<Symbol, Settlement>, ExchangeID),

    PlaceLimitOrder(LimitOrderPlacingRequest<Symbol, Settlement>, ExchangeID),

    PlaceMarketOrder(MarketOrderPlacingRequest<Symbol, Settlement>, ExchangeID),
}