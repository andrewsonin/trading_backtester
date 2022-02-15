use {
    bitflags::bitflags,
    crate::{concrete::traded_pair::{settlement::GetSettlementLag, TradedPair}, types::Id},
};

bitflags! {
    /// Bitflag containing information about the types of subscriptions to order book events.
    pub struct SubscriptionList: u8 {
        /// Subscription to trades.
        const TRADES                  = 0b00000001;
        /// Subscription to new limit orders.
        const NEW_LIMIT_ORDERS        = 0b00000010;
        /// Subscription to cancellations of limit orders.
        const CANCELLED_LIMIT_ORDERS  = 0b00000100;
        /// Subscription to order book snapshots.
        const OB_SNAPSHOTS            = 0b00001000;
    }
}

#[derive(Debug, Clone, Copy)]
/// Trader account config using by the [`BasicBroker`](crate::concrete::broker::BasicBroker).
pub struct SubscriptionConfig<ExchangeID, Symbol, Settlement>
    where ExchangeID: Id,
          Symbol: Id,
          Settlement: GetSettlementLag
{
    /// Exchange ID.
    pub exchange: ExchangeID,
    /// Traded pair.
    pub traded_pair: TradedPair<Symbol, Settlement>,
    /// Config for subscriptions to order book events.
    pub subscription: SubscriptionList,
}

impl SubscriptionList {
    #[inline]
    /// Creates empty `SubscriptionList`.
    pub fn subscribe() -> Self {
        SubscriptionList::empty()
    }
    #[inline]
    /// Merges `SubscriptionList` with another `SubscriptionList`.
    pub fn to(mut self, subscription_list: SubscriptionList) -> Self {
        self |= subscription_list;
        self
    }
    #[inline]
    /// Adds subscription to everything.
    pub fn to_everything(self) -> Self {
        SubscriptionList::all()
    }
    #[inline]
    /// Adds subscription to trades.
    pub fn to_trades(mut self) -> Self {
        self |= SubscriptionList::TRADES;
        self
    }
    #[inline]
    /// Adds subscription to new limit orders.
    pub fn to_new_limit_orders(mut self) -> Self {
        self |= SubscriptionList::NEW_LIMIT_ORDERS;
        self
    }
    #[inline]
    /// Adds subscription to cancelled limit orders.
    pub fn to_cancelled_limit_orders(mut self) -> Self {
        self |= SubscriptionList::CANCELLED_LIMIT_ORDERS;
        self
    }
    #[inline]
    /// Adds subscription to order book snapshots.
    pub fn to_ob_snapshots(mut self) -> Self {
        self |= SubscriptionList::OB_SNAPSHOTS;
        self
    }
}

impl<ExchangeID, Symbol, Settlement>
SubscriptionConfig<ExchangeID, Symbol, Settlement>
    where ExchangeID: Id,
          Symbol: Id,
          Settlement: GetSettlementLag
{
    /// Creates a new instance of the `SubscriptionConfig`.
    ///
    /// # Arguments
    ///
    /// * `exchange` — Exchange ID.
    /// * `traded_pair` — Traded pair.
    /// * `subscription` — List of subscriptions to order book events.
    pub fn new(
        exchange: ExchangeID,
        traded_pair: TradedPair<Symbol, Settlement>,
        subscription: SubscriptionList) -> Self
    {
        Self {
            exchange,
            traded_pair,
            subscription,
        }
    }
}