use crate::{
    concrete::{
        order::{LimitOrderCancelRequest, LimitOrderPlacingRequest, MarketOrderPlacingRequest},
        replay::settlement::GetSettlementLag,
        traded_pair::TradedPair,
        types::PriceStep,
    },
    interface::message::ReplayToExchange,
    types::Id,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BasicReplayToExchange<
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
> {
    pub exchange_id: ExchangeID,
    pub content: BasicReplayRequest<Symbol, Settlement>,
}

impl<
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
>
ReplayToExchange
for BasicReplayToExchange<ExchangeID, Symbol, Settlement>
{
    type ExchangeID = ExchangeID;
    fn get_exchange_id(&self) -> Self::ExchangeID {
        self.exchange_id
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BasicReplayRequest<Symbol: Id, Settlement: GetSettlementLag>
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