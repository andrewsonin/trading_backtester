use bitmask::bitmask;

bitmask! {
    pub mask SubscriptionList: u8 where flags Subscription {
        Trades                = 0b00000001,
        NewLimitOrders        = 0b00000010,
        CancelledLimitOrders  = 0b00000100,
        ObSnapshots           = 0b00001000
    }
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