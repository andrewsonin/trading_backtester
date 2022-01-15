use {
    crate::{
        settlement::GetSettlementLag,
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
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    pub broker_id: BrokerID,
    pub exchange_dt: DateTime,
    pub content: ExchangeToBrokerReply<Symbol, Settlement>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExchangeToReplay<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub content: ExchangeToReplayReply<Symbol, Settlement>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExchangeToBrokerReply<Symbol: Identifier, Settlement: GetSettlementLag>
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
pub enum ExchangeToReplayReply<Symbol: Identifier, Settlement: GetSettlementLag>
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

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotOpenExchange {
    pub reason: InabilityToOpenExchangeReason,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotStartTrades<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub reason: InabilityToStartTrades,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderAccepted<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderPlacementDiscarded<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub reason: PlacementDiscardingReason,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderPartiallyExecuted<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub price: Price,
    pub size: Size,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderExecuted<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub price: Price,
    pub size: Size,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MarketOrderNotFullyExecuted<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub remaining_size: Size,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderCancelled<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub reason: CancellationReason,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CannotCancelOrder<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub reason: InabilityToCancelReason,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExchangeEventNotification<Symbol: Identifier, Settlement: GetSettlementLag>
{
    ExchangeOpen,

    TradesStarted(TradedPair<Symbol, Settlement>, PriceStep),

    OrderCancelled(LimitOrderEventInfo<Symbol, Settlement>),

    OrderPlaced(LimitOrderEventInfo<Symbol, Settlement>),

    TradeExecuted(MarketOrderEventInfo<Symbol, Settlement>),

    ObSnapshot(Rc<ObSnapshot<Symbol, Settlement>>),

    TradesStopped(TradedPair<Symbol, Settlement>),

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
pub struct LimitOrderEventInfo<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub direction: Direction,
    pub price: Price,
    pub size: Size,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct MarketOrderEventInfo<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub direction: Direction,
    pub price: Price,
    pub size: Size,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ObSnapshot<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub state: ObState,
}