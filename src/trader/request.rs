use crate::{
    order::{LimitOrderCancelRequest, LimitOrderPlacingRequest, MarketOrderPlacingRequest},
    types::Identifier,
};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TraderToBroker<
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier
> {
    pub broker_id: BrokerID,
    pub content: TraderRequest<ExchangeID, Symbol>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TraderRequest<ExchangeID: Identifier, Symbol: Identifier>
{
    CancelLimitOrder(LimitOrderCancelRequest<Symbol>, ExchangeID),

    PlaceLimitOrder(LimitOrderPlacingRequest<Symbol>, ExchangeID),

    PlaceMarketOrder(MarketOrderPlacingRequest<Symbol>, ExchangeID),
}