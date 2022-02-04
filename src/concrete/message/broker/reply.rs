use crate::{
    concrete::{
        message::exchange::reply::{
            ExchangeEventNotification,
            MarketOrderNotFullyExecuted,
            OrderAccepted,
            OrderExecuted,
            OrderPartiallyExecuted,
        },
        replay::settlement::GetSettlementLag,
        traded_pair::TradedPair,
        types::OrderID,
    },
    interface::message::BrokerToTrader,
    types::{DateTime, Id},
};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BasicBrokerToTrader<
    TraderID: Id,
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
> {
    pub trader_id: TraderID,
    pub exchange_id: ExchangeID,
    pub event_dt: DateTime,
    pub content: BasicBrokerReply<Symbol, Settlement>,
}

impl<TraderID: Id, ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
BrokerToTrader
for BasicBrokerToTrader<TraderID, ExchangeID, Symbol, Settlement>
{
    type TraderID = TraderID;
    fn get_trader_id(&self) -> Self::TraderID {
        self.trader_id
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BasicBrokerReply<Symbol: Id, Settlement: GetSettlementLag>
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderPlacementDiscarded<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub reason: PlacementDiscardingReason,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum PlacementDiscardingReason
{
    OrderWithSuchIDAlreadySubmitted,

    ZeroSize,

    ExchangeClosed,

    NoSuchTradedPair,

    BrokerNotConnectedToExchange,

    TraderNotRegistered,
}

type ExchangePlacementDiscardingReason = crate::concrete::message::exchange::reply::PlacementDiscardingReason;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderCancelled<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub reason: CancellationReason,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CancellationReason {
    TraderRequested,
    BrokerRequested,
    TradesStopped,
    ExchangeClosed,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotCancelOrder<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub reason: InabilityToCancelReason,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToCancelReason
{
    OrderHasNotBeenSubmitted,

    OrderAlreadyExecuted,

    ExchangeClosed,

    NoSuchTradedPair,

    BrokerNotConnectedToExchange,

    TraderNotRegistered,
}

type ExchangeInabilityToCancelReason = crate::concrete::message::exchange::reply::InabilityToCancelReason;

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