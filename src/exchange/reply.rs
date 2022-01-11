use {
    crate::{
        traded_pair::TradedPair,
        types::{
            DateTime,
            Direction,
            Identifier,
            ObState,
            OrderID,
            Price,
            PriceStep,
            Size,
        },
    },
    std::rc::Rc,
};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExchangeToBroker<
    BrokerID: Identifier,
    Symbol: Identifier
> {
    pub broker_id: BrokerID,
    pub exchange_dt: DateTime,
    pub content: ExchangeToBrokerReply<Symbol>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExchangeToReplay<Symbol: Identifier> {
    pub content: ExchangeToReplayReply<Symbol>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExchangeToBrokerReply<Symbol: Identifier>
{
    OrderAccepted(OrderAccepted<Symbol>),

    OrderPlacementDiscarded(OrderPlacementDiscarded<Symbol>),

    OrderPartiallyExecuted(OrderPartiallyExecuted<Symbol>),

    OrderExecuted(OrderExecuted<Symbol>),

    MarketOrderNotFullyExecuted(MarketOrderNotFullyExecuted<Symbol>),

    OrderCancelled(OrderCancelled<Symbol>),

    CannotCancelOrder(CannotCancelOrder<Symbol>),

    ExchangeEventNotification(ExchangeEventNotification<Symbol>),
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExchangeToReplayReply<Symbol: Identifier>
{
    CannotOpenExchange(CannotOpenExchange),

    CannotStartTrades(CannotStartTrades<Symbol>),

    OrderAccepted(OrderAccepted<Symbol>),

    OrderPlacementDiscarded(OrderPlacementDiscarded<Symbol>),

    OrderPartiallyExecuted(OrderPartiallyExecuted<Symbol>),

    OrderExecuted(OrderExecuted<Symbol>),

    MarketOrderNotFullyExecuted(MarketOrderNotFullyExecuted<Symbol>),

    OrderCancelled(OrderCancelled<Symbol>),

    CannotCancelOrder(CannotCancelOrder<Symbol>),

    ExchangeEventNotification(ExchangeEventNotification<Symbol>),

    CannotCloseExchange(CannotCloseExchange),

    CannotBroadcastObState(CannotBroadcastObState),

    CannotStopTrades(CannotStopTrades),
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotOpenExchange {
    pub reason: InabilityToOpenExchangeReason,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotStartTrades<Symbol: Identifier> {
    pub traded_pair: TradedPair<Symbol>,
    pub reason: InabilityToStartTrades,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderAccepted<Symbol: Identifier> {
    pub traded_pair: TradedPair<Symbol>,
    pub order_id: OrderID,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderPlacementDiscarded<Symbol: Identifier> {
    pub traded_pair: TradedPair<Symbol>,
    pub order_id: OrderID,
    pub reason: PlacementDiscardingReason,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderPartiallyExecuted<Symbol: Identifier> {
    pub traded_pair: TradedPair<Symbol>,
    pub order_id: OrderID,
    pub price: Price,
    pub size: Size,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderExecuted<Symbol: Identifier> {
    pub traded_pair: TradedPair<Symbol>,
    pub order_id: OrderID,
    pub price: Price,
    pub size: Size,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MarketOrderNotFullyExecuted<Symbol: Identifier> {
    pub traded_pair: TradedPair<Symbol>,
    pub order_id: OrderID,
    pub remaining_size: Size,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderCancelled<Symbol: Identifier> {
    pub traded_pair: TradedPair<Symbol>,
    pub order_id: OrderID,
    pub reason: CancellationReason,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotCancelOrder<Symbol: Identifier> {
    pub traded_pair: TradedPair<Symbol>,
    pub order_id: OrderID,
    pub reason: InabilityToCancelReason,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExchangeEventNotification<Symbol: Identifier>
{
    ExchangeOpen,

    TradesStarted(TradedPair<Symbol>, PriceStep),

    OrderCancelled(LimitOrderEventInfo<Symbol>),

    OrderPlaced(LimitOrderEventInfo<Symbol>),

    TradeExecuted(MarketOrderEventInfo<Symbol>),

    ObSnapshot(Rc<ObSnapshot<Symbol>>),

    TradesStopped(TradedPair<Symbol>),

    ExchangeClosed,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotCloseExchange {
    pub reason: InabilityToCloseExchangeReason,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotBroadcastObState {
    pub reason: InabilityToBroadcastObState,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotStopTrades {
    pub reason: InabilityToStopTrades,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToOpenExchangeReason {
    AlreadyOpen
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToStartTrades {
    AlreadyStarted,
    ExchangeClosed,
    WrongSpec,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PlacementDiscardingReason
{
    OrderWithSuchIDAlreadySubmitted,

    ZeroSize,

    ExchangeClosed,

    BrokerNotConnectedToExchange,

    NoSuchTradedPair,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CancellationReason {
    BrokerRequested,
    TradesStopped,
    ExchangeClosed,
}

#[derive(derive_more::Display, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToCancelReason
{
    OrderHasNotBeenSubmitted,

    OrderAlreadyExecuted,

    ExchangeClosed,

    BrokerNotConnectedToExchange,

    NoSuchTradedPair,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToCloseExchangeReason {
    AlreadyClosed
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToBroadcastObState {
    ExchangeClosed,
    NoSuchTradedPair,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToStopTrades {
    ExchangeClosed,
    NoSuchTradedPair,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct LimitOrderEventInfo<Symbol: Identifier> {
    pub traded_pair: TradedPair<Symbol>,
    pub order_id: OrderID,
    pub direction: Direction,
    pub price: Price,
    pub size: Size,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct MarketOrderEventInfo<Symbol: Identifier> {
    pub traded_pair: TradedPair<Symbol>,
    pub direction: Direction,
    pub price: Price,
    pub size: Size,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ObSnapshot<Symbol: Identifier> {
    pub traded_pair: TradedPair<Symbol>,
    pub state: ObState,
}