use {
    bitmask::bitmask,
    crate::{
        settlement::GetSettlementLag,
        traded_pair::TradedPair,
        types::Id,
    },
};

bitmask! {
    #[derive(Debug)]
    pub mask SubscriptionList: u8 where flags Subscription {
        Trades                = 0b00000001,
        NewLimitOrders        = 0b00000010,
        CancelledLimitOrders  = 0b00000100,
        ObSnapshots           = 0b00001000
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
    pub fn subscribe() -> Self {
        SubscriptionList::none()
    }
    pub fn to(mut self, subscription_list: SubscriptionList) -> Self {
        self |= subscription_list;
        self
    }
    pub fn to_everything(self) -> Self {
        SubscriptionList::all()
    }
    pub fn to_trades(mut self) -> Self {
        self |= Subscription::Trades;
        self
    }
    pub fn to_new_limit_orders(mut self) -> Self {
        self |= Subscription::NewLimitOrders;
        self
    }
    pub fn to_cancelled_limit_orders(mut self) -> Self {
        self |= Subscription::CancelledLimitOrders;
        self
    }
    pub fn to_ob_snapshots(mut self) -> Self {
        self |= Subscription::ObSnapshots;
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