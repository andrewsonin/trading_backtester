use {
    crate::{
        broker::{
            Broker,
            BrokerAction,
            BrokerActionKind,
            reply::{
                BrokerReply,
                BrokerToTrader,
                CancellationReason,
                CannotCancelOrder,
                InabilityToCancelReason,
                OrderCancelled,
                OrderPlacementDiscarded,
                PlacementDiscardingReason,
            },
            request::{BrokerRequest, BrokerToExchange},
        },
        exchange::reply::{
            CancellationReason as ExchangeCancellationReason,
            ExchangeEventNotification,
            ExchangeToBrokerReply,
            MarketOrderNotFullyExecuted,
            OrderAccepted,
            OrderExecuted,
            OrderPartiallyExecuted,
        },
        traded_pair::TradedPair,
        trader::{request::TraderRequest, subscriptions::{Subscription, SubscriptionList}},
        types::{Date, DateTime, Identifier, Named, OrderID, StdRng, TimeSync},
    },
    std::collections::{HashMap, HashSet},
};

pub struct BasicBroker<
    BrokerID: Identifier,
    TraderID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier
> {
    current_dt: DateTime,
    name: BrokerID,

    /// Subscription configurations for each Trader
    trader_configs: HashMap<
        TraderID,
        HashMap<(ExchangeID, TradedPair<Symbol>), SubscriptionList>
    >,
    /// Map between ExchangeID + TradedPair pair
    /// and Traders that are subscribed to the corresponding pairs
    traded_pairs_info: HashMap<
        (ExchangeID, TradedPair<Symbol>),
        Vec<(TraderID, SubscriptionList)>,
    >,

    /// Submitted to Internal Order ID map
    submitted_to_internal: HashMap<(TraderID, OrderID), OrderID>,
    /// Internal to Submitted Order ID map
    internal_to_submitted: HashMap<OrderID, (TraderID, OrderID)>,

    registered_exchanges: HashSet<ExchangeID>,
    next_internal_order_id: OrderID,
}

impl<BrokerID: Identifier, TraderID: Identifier, ExchangeID: Identifier, Symbol: Identifier>
TimeSync
for BasicBroker<BrokerID, TraderID, ExchangeID, Symbol>
{
    fn current_datetime_mut(&mut self) -> &mut DateTime {
        &mut self.current_dt
    }
}

impl<BrokerID: Identifier, TraderID: Identifier, ExchangeID: Identifier, Symbol: Identifier>
Named<BrokerID>
for BasicBroker<BrokerID, TraderID, ExchangeID, Symbol>
{
    fn get_name(&self) -> BrokerID {
        self.name
    }
}

impl<BrokerID: Identifier, TraderID: Identifier, ExchangeID: Identifier, Symbol: Identifier>
Broker<BrokerID, TraderID, ExchangeID, Symbol>
for BasicBroker<BrokerID, TraderID, ExchangeID, Symbol>
{
    fn process_trader_request(
        &mut self,
        request: TraderRequest<ExchangeID, Symbol>,
        trader_id: TraderID) -> Vec<BrokerAction<TraderID, ExchangeID, Symbol>>
    {
        let action = match request {
            TraderRequest::CancelLimitOrder(mut request, exchange_id) => {
                if self.registered_exchanges.contains(&exchange_id) {
                    if let Some(order_id) = self.submitted_to_internal.get(
                        &(trader_id, request.order_id)
                    ) {
                        request.order_id = *order_id;
                        Self::create_broker_request(
                            exchange_id,
                            BrokerRequest::CancelLimitOrder(request),
                        )
                    } else {
                        Self::create_broker_reply(
                            trader_id,
                            exchange_id,
                            self.current_dt,
                            BrokerReply::CannotCancelOrder(
                                CannotCancelOrder {
                                    traded_pair: request.traded_pair,
                                    order_id: request.order_id,
                                    reason: InabilityToCancelReason::OrderHasNotBeenSubmitted,
                                }
                            ),
                        )
                    }
                } else {
                    Self::create_broker_reply(
                        trader_id,
                        exchange_id,
                        self.current_dt,
                        BrokerReply::CannotCancelOrder(
                            CannotCancelOrder {
                                traded_pair: request.traded_pair,
                                order_id: request.order_id,
                                reason: InabilityToCancelReason::BrokerNotConnectedToExchange,
                            }
                        ),
                    )
                }
            }
            TraderRequest::PlaceLimitOrder(mut request, exchange_id) => {
                if self.registered_exchanges.contains(&exchange_id) {
                    self.internal_to_submitted.insert(
                        self.next_internal_order_id,
                        (trader_id, request.order_id),
                    );
                    self.submitted_to_internal.insert(
                        (trader_id, request.order_id),
                        self.next_internal_order_id,
                    );
                    request.order_id = self.next_internal_order_id;
                    self.next_internal_order_id += OrderID(1);
                    Self::create_broker_request(
                        exchange_id,
                        BrokerRequest::PlaceLimitOrder(request),
                    )
                } else {
                    Self::create_broker_reply(
                        trader_id,
                        exchange_id,
                        self.current_dt,
                        BrokerReply::OrderPlacementDiscarded(
                            OrderPlacementDiscarded {
                                traded_pair: request.traded_pair,
                                order_id: request.order_id,
                                reason: PlacementDiscardingReason::BrokerNotConnectedToExchange,
                            }
                        ),
                    )
                }
            }
            TraderRequest::PlaceMarketOrder(mut request, exchange_id) => {
                if self.registered_exchanges.contains(&exchange_id) {
                    self.internal_to_submitted.insert(
                        self.next_internal_order_id,
                        (trader_id, request.order_id),
                    );
                    self.submitted_to_internal.insert(
                        (trader_id, request.order_id),
                        self.next_internal_order_id,
                    );
                    request.order_id = self.next_internal_order_id;
                    self.next_internal_order_id += OrderID(1);
                    Self::create_broker_request(
                        exchange_id,
                        BrokerRequest::PlaceMarketOrder(request),
                    )
                } else {
                    Self::create_broker_reply(
                        trader_id,
                        exchange_id,
                        self.current_dt,
                        BrokerReply::OrderPlacementDiscarded(
                            OrderPlacementDiscarded {
                                traded_pair: request.traded_pair,
                                order_id: request.order_id,
                                reason: PlacementDiscardingReason::BrokerNotConnectedToExchange,
                            }
                        ),
                    )
                }
            }
        };
        vec![action]
    }

    fn process_exchange_reply(
        &mut self,
        reply: ExchangeToBrokerReply<Symbol>,
        exchange_id: ExchangeID,
        exchange_dt: DateTime) -> Vec<BrokerAction<TraderID, ExchangeID, Symbol>>
    {
        let message = match reply {
            ExchangeToBrokerReply::OrderAccepted(accepted) => {
                if let Some((trader_id, order_id)) = self.internal_to_submitted.get(
                    &accepted.order_id
                ) {
                    Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        exchange_dt,
                        BrokerReply::OrderAccepted(
                            OrderAccepted {
                                traded_pair: accepted.traded_pair,
                                order_id: *order_id,
                            }
                        ),
                    )
                } else {
                    panic!(
                        "Cannot find a corresponding submitted order id \
                        for the internal order id {}", accepted.order_id
                    )
                }
            }
            ExchangeToBrokerReply::OrderPlacementDiscarded(discarded) => {
                if let Some((trader_id, order_id)) = self.internal_to_submitted.get(
                    &discarded.order_id
                ) {
                    Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        exchange_dt,
                        BrokerReply::OrderPlacementDiscarded(
                            OrderPlacementDiscarded {
                                traded_pair: discarded.traded_pair,
                                order_id: *order_id,
                                reason: discarded.reason.into(),
                            }
                        ),
                    )
                } else {
                    panic!(
                        "Cannot find a corresponding submitted order id \
                        for the internal order id {}", discarded.order_id
                    )
                }
            }
            ExchangeToBrokerReply::OrderPartiallyExecuted(executed) => {
                if let Some((trader_id, order_id)) = self.internal_to_submitted.get(
                    &executed.order_id
                ) {
                    Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        exchange_dt,
                        BrokerReply::OrderPartiallyExecuted(
                            OrderPartiallyExecuted {
                                traded_pair: executed.traded_pair,
                                order_id: *order_id,
                                price: executed.price,
                                size: executed.size,
                            }
                        ),
                    )
                } else {
                    panic!(
                        "Cannot find a corresponding submitted order id \
                        for the internal order id {}", executed.order_id
                    )
                }
            }
            ExchangeToBrokerReply::OrderExecuted(executed) => {
                if let Some((trader_id, order_id)) = self.internal_to_submitted.get(
                    &executed.order_id
                ) {
                    Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        exchange_dt,
                        BrokerReply::OrderExecuted(
                            OrderExecuted {
                                traded_pair: executed.traded_pair,
                                order_id: *order_id,
                                price: executed.price,
                                size: executed.size,
                            }
                        ),
                    )
                } else {
                    panic!(
                        "Cannot find a corresponding submitted order id \
                        for the internal order id {}", executed.order_id
                    )
                }
            }
            ExchangeToBrokerReply::MarketOrderNotFullyExecuted(not_fully_exec) => {
                if let Some((trader_id, order_id)) = self.internal_to_submitted.get(
                    &not_fully_exec.order_id
                ) {
                    Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        exchange_dt,
                        BrokerReply::MarketOrderNotFullyExecuted(
                            MarketOrderNotFullyExecuted {
                                traded_pair: not_fully_exec.traded_pair,
                                order_id: *order_id,
                                remaining_size: not_fully_exec.remaining_size,
                            }
                        ),
                    )
                } else {
                    panic!(
                        "Cannot find a corresponding submitted order id \
                        for the internal order id {}", not_fully_exec.order_id
                    )
                }
            }
            ExchangeToBrokerReply::OrderCancelled(order_cancelled) => {
                if let Some((trader_id, order_id)) = self.internal_to_submitted.get(
                    &order_cancelled.order_id
                ) {
                    Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        exchange_dt,
                        BrokerReply::OrderCancelled(
                            OrderCancelled {
                                traded_pair: order_cancelled.traded_pair,
                                order_id: *order_id,
                                reason: match order_cancelled.reason {
                                    ExchangeCancellationReason::BrokerRequested => {
                                        CancellationReason::TraderRequested
                                    }
                                    ExchangeCancellationReason::ExchangeClosed => {
                                        CancellationReason::ExchangeClosed
                                    }
                                    ExchangeCancellationReason::TradesStopped => {
                                        CancellationReason::TradesStopped
                                    }
                                },
                            }
                        ),
                    )
                } else {
                    panic!(
                        "Cannot find a corresponding submitted order id \
                        for the internal order id {}", order_cancelled.order_id
                    )
                }
            }
            ExchangeToBrokerReply::CannotCancelOrder(cannot_cancel) => {
                if let Some((trader_id, order_id)) = self.internal_to_submitted.get(
                    &cannot_cancel.order_id
                ) {
                    Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        exchange_dt,
                        BrokerReply::CannotCancelOrder(
                            CannotCancelOrder {
                                traded_pair: cannot_cancel.traded_pair,
                                order_id: *order_id,
                                reason: cannot_cancel.reason.into(),
                            }
                        ),
                    )
                } else {
                    panic!(
                        "Cannot find a corresponding submitted order id \
                        for the internal order id {}", cannot_cancel.order_id
                    )
                }
            }
            ExchangeToBrokerReply::ExchangeEventNotification(notification) => {
                return self.handle_exchange_notification(notification, exchange_id, exchange_dt);
            }
        };
        vec![message]
    }

    fn wakeup(&mut self) -> Vec<BrokerAction<TraderID, ExchangeID, Symbol>> {
        unreachable!("{} :: Broker wakeups are not planned", self.current_dt)
    }

    fn broker_to_exchange_latency(&self, _: ExchangeID, _: &mut StdRng, _: DateTime) -> u64 { 0 }

    fn exchange_to_broker_latency(&self, _: ExchangeID, _: &mut StdRng, _: DateTime) -> u64 { 0 }

    fn upon_connection_to_exchange(&mut self, exchange_id: ExchangeID) {
        self.registered_exchanges.insert(exchange_id);
    }

    fn register_trader(
        &mut self,
        trader_id: TraderID,
        sub_cfgs: impl IntoIterator<Item=(ExchangeID, TradedPair<Symbol>, SubscriptionList)>)
    {
        self.trader_configs.insert(
            trader_id,
            sub_cfgs.into_iter()
                .inspect(
                    |(exchange, traded_pair, subscription_config)| {
                        if !self.registered_exchanges.contains(&exchange) {
                            panic!("Broker {} is not connected to Exchange {}",
                                   self.name, exchange)
                        };
                        self.traded_pairs_info
                            .entry((*exchange, *traded_pair))
                            .or_default()
                            .push((trader_id, *subscription_config))
                    }
                )
                .map(
                    |(exchange, traded_pair, subscription_config)|
                        ((exchange, traded_pair), subscription_config)
                ).collect(),
        );
    }
}

impl<BrokerID: Identifier, TraderID: Identifier, ExchangeID: Identifier, Symbol: Identifier>
BasicBroker<BrokerID, TraderID, ExchangeID, Symbol>
{
    pub fn new(name: BrokerID) -> Self {
        BasicBroker {
            current_dt: Date::from_ymd(1970, 01, 01).and_hms(0, 0, 0),
            name,
            trader_configs: Default::default(),
            traded_pairs_info: Default::default(),
            submitted_to_internal: Default::default(),
            internal_to_submitted: Default::default(),
            registered_exchanges: Default::default(),
            next_internal_order_id: OrderID(0),
        }
    }

    fn handle_exchange_notification(
        &mut self,
        notification: ExchangeEventNotification<Symbol>,
        exchange_id: ExchangeID,
        exchange_dt: DateTime) -> Vec<BrokerAction<TraderID, ExchangeID, Symbol>>
    {
        match notification {
            ExchangeEventNotification::ExchangeOpen => {
                self.trader_configs.keys().map(
                    |trader_id| Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        exchange_dt,
                        BrokerReply::ExchangeEventNotification(
                            ExchangeEventNotification::ExchangeOpen
                        ),
                    )
                ).collect()
            }
            ExchangeEventNotification::TradesStarted(traded_pair, price_step) => {
                self.trader_configs.keys().map(
                    |trader_id| Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        exchange_dt,
                        BrokerReply::ExchangeEventNotification(
                            ExchangeEventNotification::TradesStarted(traded_pair, price_step)
                        ),
                    )
                ).collect()
            }
            ExchangeEventNotification::OrderCancelled(cancelled) => {
                self.trader_configs.iter().filter_map(
                    |(trader_id, configs)| {
                        if let Some(config) = configs.get(&(exchange_id, cancelled.traded_pair)) {
                            if config.contains(Subscription::CancelledLimitOrders) {
                                let notification = Self::create_broker_reply(
                                    *trader_id,
                                    exchange_id,
                                    exchange_dt,
                                    BrokerReply::ExchangeEventNotification(
                                        ExchangeEventNotification::OrderCancelled(cancelled)
                                    ),
                                );
                                return Some(notification);
                            }
                        }
                        None
                    }
                ).collect()
            }
            ExchangeEventNotification::OrderPlaced(placed) => {
                self.trader_configs.iter().filter_map(
                    |(trader_id, configs)| {
                        if let Some(config) = configs.get(&(exchange_id, placed.traded_pair)) {
                            if config.contains(Subscription::NewLimitOrders) {
                                let notification = Self::create_broker_reply(
                                    *trader_id,
                                    exchange_id,
                                    exchange_dt,
                                    BrokerReply::ExchangeEventNotification(
                                        ExchangeEventNotification::OrderPlaced(placed)
                                    ),
                                );
                                return Some(notification);
                            }
                        }
                        None
                    }
                ).collect()
            }
            ExchangeEventNotification::TradeExecuted(trade) => {
                self.trader_configs.iter().filter_map(
                    |(trader_id, configs)| {
                        if let Some(config) = configs.get(&(exchange_id, trade.traded_pair)) {
                            if config.contains(Subscription::Trades) {
                                let notification = Self::create_broker_reply(
                                    *trader_id,
                                    exchange_id,
                                    exchange_dt,
                                    BrokerReply::ExchangeEventNotification(
                                        ExchangeEventNotification::TradeExecuted(trade)
                                    ),
                                );
                                return Some(notification);
                            }
                        }
                        None
                    }
                ).collect()
            }
            ExchangeEventNotification::ObSnapshot(ob_snapshot) => {
                self.trader_configs.iter().filter_map(
                    |(trader_id, configs)| {
                        if let Some(config) = configs.get(&(exchange_id, ob_snapshot.traded_pair)) {
                            if config.contains(Subscription::ObSnapshots) {
                                let ob_snapshot = Self::create_broker_reply(
                                    *trader_id,
                                    exchange_id,
                                    exchange_dt,
                                    BrokerReply::ExchangeEventNotification(
                                        ExchangeEventNotification::ObSnapshot(ob_snapshot.clone())
                                    ),
                                );
                                return Some(ob_snapshot);
                            }
                        }
                        None
                    }
                ).collect()
            }
            ExchangeEventNotification::TradesStopped(traded_pair) => {
                self.trader_configs.keys().map(
                    |trader_id| Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        exchange_dt,
                        BrokerReply::ExchangeEventNotification(
                            ExchangeEventNotification::TradesStopped(traded_pair)
                        ),
                    )
                ).collect()
            }
            ExchangeEventNotification::ExchangeClosed => {
                self.trader_configs.keys().map(
                    |trader_id| Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        exchange_dt,
                        BrokerReply::ExchangeEventNotification(
                            ExchangeEventNotification::ExchangeClosed
                        ),
                    )
                ).collect()
            }
        }
    }

    fn create_broker_reply(
        trader_id: TraderID,
        exchange_id: ExchangeID,
        event_dt: DateTime,
        content: BrokerReply<Symbol>) -> BrokerAction<TraderID, ExchangeID, Symbol>
    {
        BrokerAction {
            delay: 0,
            content: BrokerActionKind::BrokerToTrader(
                BrokerToTrader {
                    trader_id,
                    exchange_id,
                    event_dt,
                    content,
                }
            ),
        }
    }

    fn create_broker_request(
        exchange_id: ExchangeID,
        content: BrokerRequest<Symbol>) -> BrokerAction<TraderID, ExchangeID, Symbol>
    {
        BrokerAction {
            delay: 0,
            content: BrokerActionKind::BrokerToExchange(
                BrokerToExchange {
                    exchange_id,
                    content,
                }
            ),
        }
    }
}