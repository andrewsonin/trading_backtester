use {
    bitflags::bitflags,
    crate::{
        settlement::GetSettlementLag,
        traded_pair::TradedPair,
        types::Id,
    },
};

bitflags! {
    pub struct SubscriptionList: u8 {
        const TRADES                  = 0b00000001;
        const NEW_LIMIT_ORDERS        = 0b00000010;
        const CANCELLED_LIMIT_ORDERS  = 0b00000100;
        const OB_SNAPSHOTS            = 0b00001000;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SubscriptionConfig<
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
> {
    pub exchange: ExchangeID,
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub subscription: SubscriptionList,
}

impl SubscriptionList {
    #[inline]
    pub fn subscribe() -> Self {
        SubscriptionList::empty()
    }
    #[inline]
    pub fn to(mut self, subscription_list: SubscriptionList) -> Self {
        self |= subscription_list;
        self
    }
    #[inline]
    pub fn to_everything(self) -> Self {
        SubscriptionList::all()
    }
    #[inline]
    pub fn to_trades(mut self) -> Self {
        self |= SubscriptionList::TRADES;
        self
    }
    #[inline]
    pub fn to_new_limit_orders(mut self) -> Self {
        self |= SubscriptionList::NEW_LIMIT_ORDERS;
        self
    }
    #[inline]
    pub fn to_cancelled_limit_orders(mut self) -> Self {
        self |= SubscriptionList::CANCELLED_LIMIT_ORDERS;
        self
    }
    #[inline]
    pub fn to_ob_snapshots(mut self) -> Self {
        self |= SubscriptionList::OB_SNAPSHOTS;
        self
    }
}

impl<ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
SubscriptionConfig<ExchangeID, Symbol, Settlement>
{
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