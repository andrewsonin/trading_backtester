use crate::{
    order::{LimitOrderCancelRequest, LimitOrderPlacingRequest, MarketOrderPlacingRequest},
    settlement::GetSettlementLag,
    types::Identifier,
};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BrokerToExchange<
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    pub exchange_id: ExchangeID,
    pub content: BrokerRequest<Symbol, Settlement>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BrokerRequest<Symbol: Identifier, Settlement: GetSettlementLag>
{
    CancelLimitOrder(LimitOrderCancelRequest<Symbol, Settlement>),

    PlaceLimitOrder(LimitOrderPlacingRequest<Symbol, Settlement>),

    PlaceMarketOrder(MarketOrderPlacingRequest<Symbol, Settlement>),
}