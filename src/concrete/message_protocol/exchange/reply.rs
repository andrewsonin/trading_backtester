use {
    crate::{
        concrete::{
            traded_pair::{settlement::GetSettlementLag, TradedPair},
            types::{Direction, ObState, OrderID, Price, PriceStep, Size},
        },
        interface::message::{ExchangeToBroker, ExchangeToReplay},
        types::{
            DateTime,
            Id,
        },
    },
    std::rc::Rc,
};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BasicExchangeToBroker<
    BrokerID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
> {
    pub broker_id: BrokerID,
    pub exchange_dt: DateTime,
    pub content: BasicExchangeToBrokerReply<Symbol, Settlement>,
}

impl<
    BrokerID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
>
ExchangeToBroker
for BasicExchangeToBroker<BrokerID, Symbol, Settlement>
{
    type BrokerID = BrokerID;

    fn get_broker_id(&self) -> Self::BrokerID {
        self.broker_id
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BasicExchangeToReplay<Symbol: Id, Settlement: GetSettlementLag> {
    pub content: BasicExchangeToReplayReply<Symbol, Settlement>,
}

impl<Symbol: Id, Settlement: GetSettlementLag> ExchangeToReplay
for BasicExchangeToReplay<Symbol, Settlement> {}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BasicExchangeToBrokerReply<Symbol: Id, Settlement: GetSettlementLag>
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

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BasicExchangeToReplayReply<Symbol: Id, Settlement: GetSettlementLag>
{
    CannotOpenExchange(CannotOpenExchange),

    CannotStartTrades(CannotStartTrades<Symbol, Settlement>),

    OrderAccepted(OrderAccepted<Symbol, Settlement>),

    OrderPlacementDiscarded(OrderPlacementDiscarded<Symbol, Settlement>),

    OrderPartiallyExecuted(OrderPartiallyExecuted<Symbol, Settlement>),

    OrderExecuted(OrderExecuted<Symbol, Settlement>),

    MarketOrderNotFullyExecuted(MarketOrderNotFullyExecuted<Symbol, Settlement>),

    OrderCancelled(OrderCancelled<Symbol, Settlement>),

    CannotCancelOrder(CannotCancelOrder<Symbol, Settlement>),

    ExchangeEventNotification(ExchangeEventNotification<Symbol, Settlement>),

    CannotCloseExchange(CannotCloseExchange),

    CannotBroadcastObState(CannotBroadcastObState),

    CannotStopTrades(CannotStopTrades),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotOpenExchange {
    pub reason: InabilityToOpenExchangeReason,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotStartTrades<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub reason: InabilityToStartTrades,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderAccepted<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderPlacementDiscarded<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub reason: PlacementDiscardingReason,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderPartiallyExecuted<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub price: Price,
    pub size: Size,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderExecuted<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub price: Price,
    pub size: Size,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MarketOrderNotFullyExecuted<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub remaining_size: Size,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderCancelled<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub reason: CancellationReason,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotCancelOrder<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub reason: InabilityToCancelReason,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExchangeEventNotification<Symbol: Id, Settlement: GetSettlementLag>
{
    ExchangeOpen,

    TradesStarted { traded_pair: TradedPair<Symbol, Settlement>, price_step: PriceStep },

    OrderCancelled(LimitOrderEventInfo<Symbol, Settlement>),

    OrderPlaced(LimitOrderEventInfo<Symbol, Settlement>),

    TradeExecuted(MarketOrderEventInfo<Symbol, Settlement>),

    ObSnapshot(Rc<ObSnapshot<Symbol, Settlement>>),

    TradesStopped(TradedPair<Symbol, Settlement>),

    ExchangeClosed,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotCloseExchange {
    pub reason: InabilityToCloseExchangeReason,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotBroadcastObState {
    pub reason: InabilityToBroadcastObState,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotStopTrades {
    pub reason: InabilityToStopTrades,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToOpenExchangeReason {
    AlreadyOpen
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToStartTrades {
    AlreadyStarted,
    ExchangeClosed,
    WrongSpec,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum PlacementDiscardingReason
{
    OrderWithSuchIDAlreadySubmitted,

    ZeroSize,

    ExchangeClosed,

    BrokerNotConnectedToExchange,

    NoSuchTradedPair,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CancellationReason {
    BrokerRequested,
    TradesStopped,
    ExchangeClosed,
}

#[derive(derive_more::Display, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToCancelReason
{
    OrderHasNotBeenSubmitted,

    OrderAlreadyExecuted,

    ExchangeClosed,

    BrokerNotConnectedToExchange,

    NoSuchTradedPair,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToCloseExchangeReason {
    AlreadyClosed
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToBroadcastObState {
    ExchangeClosed,
    NoSuchTradedPair,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToStopTrades {
    ExchangeClosed,
    NoSuchTradedPair,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct LimitOrderEventInfo<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub direction: Direction,
    pub price: Price,
    pub size: Size,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct MarketOrderEventInfo<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub direction: Direction,
    pub price: Price,
    pub size: Size,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ObSnapshot<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub state: ObState,
}