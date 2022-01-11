use crate::{
    order::{LimitOrderCancelRequest, LimitOrderPlacingRequest, MarketOrderPlacingRequest},
    types::Identifier,
};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BrokerToExchange<
    ExchangeID: Identifier,
    Symbol: Identifier
> {
    pub exchange_id: ExchangeID,
    pub content: BrokerRequest<Symbol>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BrokerRequest<Symbol: Identifier>
{
    CancelLimitOrder(LimitOrderCancelRequest<Symbol>),

    PlaceLimitOrder(LimitOrderPlacingRequest<Symbol>),

    PlaceMarketOrder(MarketOrderPlacingRequest<Symbol>),
}