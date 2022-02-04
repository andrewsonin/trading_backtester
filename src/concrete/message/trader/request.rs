use crate::{
    concrete::{
        order::{LimitOrderCancelRequest, LimitOrderPlacingRequest, MarketOrderPlacingRequest},
        replay::settlement::GetSettlementLag,
    },
    interface::message::TraderToBroker,
    types::Id,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BasicTraderToBroker<
    BrokerID: Id,
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
> {
    pub broker_id: BrokerID,
    pub content: BasicTraderRequest<ExchangeID, Symbol, Settlement>,
}

impl<
    BrokerID: Id,
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
> TraderToBroker
for BasicTraderToBroker<BrokerID, ExchangeID, Symbol, Settlement>
{
    type BrokerID = BrokerID;

    fn get_broker_id(&self) -> Self::BrokerID {
        self.broker_id
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BasicTraderRequest<
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
> {
    CancelLimitOrder(LimitOrderCancelRequest<Symbol, Settlement>, ExchangeID),

    PlaceLimitOrder(LimitOrderPlacingRequest<Symbol, Settlement>, ExchangeID),

    PlaceMarketOrder(MarketOrderPlacingRequest<Symbol, Settlement>, ExchangeID),
}