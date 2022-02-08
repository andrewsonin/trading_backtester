use crate::{
    concrete::{
        order::{LimitOrderCancelRequest, LimitOrderPlacingRequest, MarketOrderPlacingRequest},
        traded_pair::settlement::GetSettlementLag,
    },
    interface::message::BrokerToExchange,
    types::Id,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BasicBrokerToExchange<
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
> {
    pub exchange_id: ExchangeID,
    pub content: BasicBrokerRequest<Symbol, Settlement>,
}

impl<ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
BrokerToExchange
for BasicBrokerToExchange<ExchangeID, Symbol, Settlement>
{
    type ExchangeID = ExchangeID;
    fn get_exchange_id(&self) -> Self::ExchangeID {
        self.exchange_id
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BasicBrokerRequest<Symbol: Id, Settlement: GetSettlementLag>
{
    CancelLimitOrder(LimitOrderCancelRequest<Symbol, Settlement>),

    PlaceLimitOrder(LimitOrderPlacingRequest<Symbol, Settlement>),

    PlaceMarketOrder(MarketOrderPlacingRequest<Symbol, Settlement>),
}