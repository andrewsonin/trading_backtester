use crate::{
    exchange::reply::{
        ExchangeEventNotification,
        MarketOrderNotFullyExecuted,
        OrderAccepted,
        OrderExecuted,
        OrderPartiallyExecuted,
    },
    settlement::GetSettlementLag,
    traded_pair::TradedPair,
    types::{DateTime, Identifier, OrderID},
};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BrokerToTrader<
    TraderID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    pub trader_id: TraderID,
    pub exchange_id: ExchangeID,
    pub event_dt: DateTime,
    pub content: BrokerReply<Symbol, Settlement>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BrokerReply<Symbol: Identifier, Settlement: GetSettlementLag>
{
    OrderAccepted(OrderAccepted<Symbol, Settlement>),

    OrderPlacementDiscarded(OrderPlacementDiscarded<Symbol, Settlement>),

    OrderPartiallyExecuted(OrderPartiallyExecuted<Symbol, Settlement>),

    OrderExecuted(OrderExecuted<Symbol, Settlement>),

    MarketOrderNotFullyExecuted(MarketOrderNotFullyExecuted<Symbol, Settlement>),

    OrderCancelled(OrderCancelled<Symbol, Settlement>),

    CannotCancelOrder(CannotCancelOrder<Symbol, Settlement>),

    ExchangeEventNotification(ExchangeEventNotification<Symbol, Settlement>),
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderPlacementDiscarded<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub reason: PlacementDiscardingReason,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PlacementDiscardingReason
{
    OrderWithSuchIDAlreadySubmitted,

    ZeroSize,

    ExchangeClosed,

    NoSuchTradedPair,

    BrokerNotConnectedToExchange,

    TraderNotRegistered,
}

type ExchangePlacementDiscardingReason = crate::exchange::reply::PlacementDiscardingReason;

impl From<ExchangePlacementDiscardingReason> for PlacementDiscardingReason {
    fn from(reason: ExchangePlacementDiscardingReason) -> Self {
        match reason {
            ExchangePlacementDiscardingReason::OrderWithSuchIDAlreadySubmitted => {
                Self::OrderWithSuchIDAlreadySubmitted
            }
            ExchangePlacementDiscardingReason::ZeroSize => {
                Self::ZeroSize
            }
            ExchangePlacementDiscardingReason::ExchangeClosed => {
                Self::ExchangeClosed
            }
            ExchangePlacementDiscardingReason::BrokerNotConnectedToExchange => {
                Self::BrokerNotConnectedToExchange
            }
            ExchangePlacementDiscardingReason::NoSuchTradedPair => {
                Self::NoSuchTradedPair
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderCancelled<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub reason: CancellationReason,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CancellationReason {
    TraderRequested,
    BrokerRequested,
    TradesStopped,
    ExchangeClosed,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotCancelOrder<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub reason: InabilityToCancelReason,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToCancelReason
{
    OrderHasNotBeenSubmitted,

    OrderAlreadyExecuted,

    ExchangeClosed,

    NoSuchTradedPair,

    BrokerNotConnectedToExchange,

    TraderNotRegistered,
}

type ExchangeInabilityToCancelReason = crate::exchange::reply::InabilityToCancelReason;

impl From<ExchangeInabilityToCancelReason> for InabilityToCancelReason {
    fn from(reason: ExchangeInabilityToCancelReason) -> Self {
        match reason {
            ExchangeInabilityToCancelReason::OrderHasNotBeenSubmitted => {
                Self::OrderHasNotBeenSubmitted
            }
            ExchangeInabilityToCancelReason::OrderAlreadyExecuted => {
                Self::OrderAlreadyExecuted
            }
            ExchangeInabilityToCancelReason::ExchangeClosed => {
                Self::ExchangeClosed
            }
            ExchangeInabilityToCancelReason::BrokerNotConnectedToExchange => {
                Self::BrokerNotConnectedToExchange
            }
            ExchangeInabilityToCancelReason::NoSuchTradedPair => {
                Self::NoSuchTradedPair
            }
        }
    }
}