use {
    crate::{
        broker::{
            Broker,
            BrokerAction,
            BrokerActionKind,
            BrokerToExchange,
            BrokerToItself,
            BrokerToReplay,
            BrokerToTrader,
            reply::{
                BasicBrokerReply,
                BasicBrokerToTrader,
                CancellationReason,
                CannotCancelOrder,
                InabilityToCancelReason,
                OrderCancelled,
                OrderPlacementDiscarded,
                PlacementDiscardingReason,
            },
            request::{BasicBrokerRequest, BasicBrokerToExchange},
        },
        exchange::{
            ExchangeToBroker,
            reply::{
                BasicExchangeToBroker,
                BasicExchangeToBrokerReply,
                CancellationReason as ExchangeCancellationReason,
                ExchangeEventNotification,
                MarketOrderNotFullyExecuted,
                OrderAccepted,
                OrderExecuted,
                OrderPartiallyExecuted,
            },
        },
        kernel::LatentActionProcessor,
        latency::{concrete::ConstantLatency, Latent},
        replay::ReplayToBroker,
        settlement::GetSettlementLag,
        traded_pair::TradedPair,
        trader::{
            request::{BasicTraderRequest, BasicTraderToBroker},
            subscriptions::{SubscriptionConfig, SubscriptionList},
            TraderToBroker,
        },
        types::{Agent, Date, DateTime, Id, Named, NeverType, Nothing, OrderID, TimeSync},
        utils::queue::MessageReceiver,
    },
    rand::Rng,
    std::{collections::{HashMap, HashSet}, marker::PhantomData, rc::Rc},
};

pub struct BasicBroker<
    BrokerID: Id,
    TraderID: Id,
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
> {
    current_dt: DateTime,
    name: BrokerID,

    /// Subscription configurations for each Trader
    trader_configs: HashMap<
        TraderID,
        HashMap<(ExchangeID, TradedPair<Symbol, Settlement>), SubscriptionList>
    >,
    /// Map between ExchangeID + TradedPair pair
    /// and Traders that are subscribed to the corresponding pairs
    traded_pairs_info: HashMap<
        (ExchangeID, TradedPair<Symbol, Settlement>),
        Vec<(TraderID, SubscriptionList)>,
    >,

    /// Submitted to Internal Order ID map
    submitted_to_internal: HashMap<(TraderID, OrderID), OrderID>,
    /// Internal to Submitted Order ID map
    internal_to_submitted: HashMap<OrderID, (TraderID, OrderID)>,

    registered_exchanges: HashSet<ExchangeID>,
    next_internal_order_id: OrderID,
}

impl<BrokerID: Id, TraderID: Id, ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
TimeSync
for BasicBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>
{
    fn current_datetime_mut(&mut self) -> &mut DateTime {
        &mut self.current_dt
    }
}

impl<BrokerID: Id, TraderID: Id, ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
Named<BrokerID>
for BasicBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>
{
    fn get_name(&self) -> BrokerID {
        self.name
    }
}

impl<BrokerID: Id, TraderID: Id, ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
Agent for BasicBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>
{
    type Action = BrokerAction<
        Nothing,
        BasicBrokerToExchange<ExchangeID, Symbol, Settlement>,
        BasicBrokerToTrader<TraderID, ExchangeID, Symbol, Settlement>,
        Nothing
    >;
}

impl<BrokerID: Id, TraderID: Id, ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
Latent for BasicBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>
{
    type OuterID = ExchangeID;
    type LatencyGenerator = ConstantLatency<ExchangeID, 0, 0>;

    fn get_latency_generator(&self) -> Self::LatencyGenerator {
        ConstantLatency::<ExchangeID, 0, 0>::new()
    }
}

impl<BrokerID: Id, TraderID: Id, ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
Broker
for BasicBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>
{
    type BrokerID = BrokerID;
    type TraderID = TraderID;
    type ExchangeID = ExchangeID;

    type R2B = NeverType<BrokerID>;
    type E2B = BasicExchangeToBroker<BrokerID, Symbol, Settlement>;
    type T2B = BasicTraderToBroker<BrokerID, ExchangeID, Symbol, Settlement>;
    type B2R = Nothing;
    type B2E = BasicBrokerToExchange<ExchangeID, Symbol, Settlement>;
    type B2T = BasicBrokerToTrader<TraderID, ExchangeID, Symbol, Settlement>;
    type B2B = Nothing;
    type SubCfg = SubscriptionConfig<ExchangeID, Symbol, Settlement>;

    fn wakeup<KerMsg: Ord>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
        _: Nothing,
        _: &mut impl Rng,
    ) {
        unreachable!("{} :: Broker wakeups are not planned", self.current_dt)
    }

    fn process_trader_request<KerMsg: Ord>(
        &mut self,
        mut message_receiver: MessageReceiver<KerMsg>,
        mut action_processor: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
        request: BasicTraderToBroker<BrokerID, ExchangeID, Symbol, Settlement>,
        trader_id: TraderID,
        rng: &mut impl Rng,
    ) {
        let action = match request.content {
            BasicTraderRequest::CancelLimitOrder(mut request, exchange_id) => {
                if self.registered_exchanges.contains(&exchange_id) {
                    if let Some(order_id) = self.submitted_to_internal.get(
                        &(trader_id, request.order_id)
                    ) {
                        request.order_id = *order_id;
                        Self::create_broker_request(
                            exchange_id,
                            BasicBrokerRequest::CancelLimitOrder(request),
                        )
                    } else {
                        Self::create_broker_reply(
                            trader_id,
                            exchange_id,
                            self.current_dt,
                            BasicBrokerReply::CannotCancelOrder(
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
                        BasicBrokerReply::CannotCancelOrder(
                            CannotCancelOrder {
                                traded_pair: request.traded_pair,
                                order_id: request.order_id,
                                reason: InabilityToCancelReason::BrokerNotConnectedToExchange,
                            }
                        ),
                    )
                }
            }
            BasicTraderRequest::PlaceLimitOrder(mut request, exchange_id) => {
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
                        BasicBrokerRequest::PlaceLimitOrder(request),
                    )
                } else {
                    Self::create_broker_reply(
                        trader_id,
                        exchange_id,
                        self.current_dt,
                        BasicBrokerReply::OrderPlacementDiscarded(
                            OrderPlacementDiscarded {
                                traded_pair: request.traded_pair,
                                order_id: request.order_id,
                                reason: PlacementDiscardingReason::BrokerNotConnectedToExchange,
                            }
                        ),
                    )
                }
            }
            BasicTraderRequest::PlaceMarketOrder(mut request, exchange_id) => {
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
                        BasicBrokerRequest::PlaceMarketOrder(request),
                    )
                } else {
                    Self::create_broker_reply(
                        trader_id,
                        exchange_id,
                        self.current_dt,
                        BasicBrokerReply::OrderPlacementDiscarded(
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
        message_receiver.push(
            action_processor.process_action(action, self.get_latency_generator(), rng)
        )
    }

    fn process_exchange_reply<KerMsg: Ord>(
        &mut self,
        mut message_receiver: MessageReceiver<KerMsg>,
        mut action_processor: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
        reply: BasicExchangeToBroker<BrokerID, Symbol, Settlement>,
        exchange_id: ExchangeID,
        rng: &mut impl Rng,
    ) {
        let message = match reply.content {
            BasicExchangeToBrokerReply::OrderAccepted(accepted) => {
                if let Some((trader_id, order_id)) = self.internal_to_submitted.get(
                    &accepted.order_id
                ) {
                    Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        reply.exchange_dt,
                        BasicBrokerReply::OrderAccepted(
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
            BasicExchangeToBrokerReply::OrderPlacementDiscarded(discarded) => {
                if let Some((trader_id, order_id)) = self.internal_to_submitted.get(
                    &discarded.order_id
                ) {
                    Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        reply.exchange_dt,
                        BasicBrokerReply::OrderPlacementDiscarded(
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
            BasicExchangeToBrokerReply::OrderPartiallyExecuted(executed) => {
                if let Some((trader_id, order_id)) = self.internal_to_submitted.get(
                    &executed.order_id
                ) {
                    Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        reply.exchange_dt,
                        BasicBrokerReply::OrderPartiallyExecuted(
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
            BasicExchangeToBrokerReply::OrderExecuted(executed) => {
                if let Some((trader_id, order_id)) = self.internal_to_submitted.get(
                    &executed.order_id
                ) {
                    Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        reply.exchange_dt,
                        BasicBrokerReply::OrderExecuted(
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
            BasicExchangeToBrokerReply::MarketOrderNotFullyExecuted(not_fully_exec) => {
                if let Some((trader_id, order_id)) = self.internal_to_submitted.get(
                    &not_fully_exec.order_id
                ) {
                    Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        reply.exchange_dt,
                        BasicBrokerReply::MarketOrderNotFullyExecuted(
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
            BasicExchangeToBrokerReply::OrderCancelled(order_cancelled) => {
                if let Some((trader_id, order_id)) = self.internal_to_submitted.get(
                    &order_cancelled.order_id
                ) {
                    Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        reply.exchange_dt,
                        BasicBrokerReply::OrderCancelled(
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
            BasicExchangeToBrokerReply::CannotCancelOrder(cannot_cancel) => {
                if let Some((trader_id, order_id)) = self.internal_to_submitted.get(
                    &cannot_cancel.order_id
                ) {
                    Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        reply.exchange_dt,
                        BasicBrokerReply::CannotCancelOrder(
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
            BasicExchangeToBrokerReply::ExchangeEventNotification(notification) => {
                self.handle_exchange_notification(
                    message_receiver,
                    action_processor,
                    notification,
                    exchange_id,
                    reply.exchange_dt,
                    rng,
                );
                return;
            }
        };
        message_receiver.push(
            action_processor.process_action(message, self.get_latency_generator(), rng)
        )
    }

    fn process_replay_request<KerMsg: Ord>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
        _: Self::R2B,
        _: &mut impl Rng,
    ) {
        unreachable!("{} :: Did not plan to communicate with brokers", self.current_dt)
    }

    fn upon_connection_to_exchange(&mut self, exchange_id: ExchangeID) {
        self.registered_exchanges.insert(exchange_id);
    }

    fn register_trader(
        &mut self,
        trader_id: TraderID,
        sub_cfgs: impl IntoIterator<Item=SubscriptionConfig<ExchangeID, Symbol, Settlement>>,
    ) {
        self.trader_configs.insert(
            trader_id,
            sub_cfgs.into_iter()
                .inspect(
                    |SubscriptionConfig { exchange, traded_pair, subscription }| {
                        if !self.registered_exchanges.contains(&exchange) {
                            panic!("Broker {} is not connected to Exchange {exchange}", self.name)
                        };
                        self.traded_pairs_info
                            .entry((*exchange, *traded_pair))
                            .or_default()
                            .push((trader_id, *subscription))
                    }
                )
                .map(
                    |SubscriptionConfig { exchange, traded_pair, subscription }|
                        ((exchange, traded_pair), subscription)
                ).collect(),
        );
    }
}

impl<BrokerID: Id, TraderID: Id, ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
BasicBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>
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

    fn handle_exchange_notification<KerMsg: Ord, RNG: Rng>(
        &mut self,
        mut message_receiver: MessageReceiver<KerMsg>,
        mut action_processor: impl LatentActionProcessor<<Self as Agent>::Action, <Self as Broker>::ExchangeID, KerMsg=KerMsg>,
        notification: ExchangeEventNotification<Symbol, Settlement>,
        exchange_id: ExchangeID,
        exchange_dt: DateTime,
        rng: &mut RNG,
    ) {
        let process_action = |action|
            action_processor.process_action(
                action,
                self.get_latency_generator(),
                rng,
            );
        match notification {
            ExchangeEventNotification::ExchangeOpen => {
                let action_iterator = self.trader_configs.keys().map(
                    |trader_id| Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        exchange_dt,
                        BasicBrokerReply::ExchangeEventNotification(
                            ExchangeEventNotification::ExchangeOpen
                        ),
                    )
                );
                message_receiver.extend(action_iterator.map(process_action))
            }
            ExchangeEventNotification::TradesStarted(traded_pair, price_step) => {
                let action_iterator = self.trader_configs.keys().map(
                    |trader_id| Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        exchange_dt,
                        BasicBrokerReply::ExchangeEventNotification(
                            ExchangeEventNotification::TradesStarted(traded_pair, price_step)
                        ),
                    )
                );
                message_receiver.extend(action_iterator.map(process_action))
            }
            ExchangeEventNotification::OrderCancelled(cancelled) => {
                let action_iterator = self.trader_configs.iter().filter_map(
                    |(trader_id, configs)| {
                        if let Some(config) = configs.get(&(exchange_id, cancelled.traded_pair)) {
                            if config.contains(SubscriptionList::CANCELLED_LIMIT_ORDERS) {
                                let notification = Self::create_broker_reply(
                                    *trader_id,
                                    exchange_id,
                                    exchange_dt,
                                    BasicBrokerReply::ExchangeEventNotification(
                                        ExchangeEventNotification::OrderCancelled(cancelled)
                                    ),
                                );
                                return Some(notification);
                            }
                        }
                        None
                    }
                );
                message_receiver.extend(action_iterator.map(process_action))
            }
            ExchangeEventNotification::OrderPlaced(placed) => {
                let action_iterator = self.trader_configs.iter().filter_map(
                    |(trader_id, configs)| {
                        if let Some(config) = configs.get(&(exchange_id, placed.traded_pair)) {
                            if config.contains(SubscriptionList::NEW_LIMIT_ORDERS) {
                                let notification = Self::create_broker_reply(
                                    *trader_id,
                                    exchange_id,
                                    exchange_dt,
                                    BasicBrokerReply::ExchangeEventNotification(
                                        ExchangeEventNotification::OrderPlaced(placed)
                                    ),
                                );
                                return Some(notification);
                            }
                        }
                        None
                    }
                );
                message_receiver.extend(action_iterator.map(process_action))
            }
            ExchangeEventNotification::TradeExecuted(trade) => {
                let action_iterator = self.trader_configs.iter().filter_map(
                    |(trader_id, configs)| {
                        if let Some(config) = configs.get(&(exchange_id, trade.traded_pair)) {
                            if config.contains(SubscriptionList::TRADES) {
                                let notification = Self::create_broker_reply(
                                    *trader_id,
                                    exchange_id,
                                    exchange_dt,
                                    BasicBrokerReply::ExchangeEventNotification(
                                        ExchangeEventNotification::TradeExecuted(trade)
                                    ),
                                );
                                return Some(notification);
                            }
                        }
                        None
                    }
                );
                message_receiver.extend(action_iterator.map(process_action))
            }
            ExchangeEventNotification::ObSnapshot(ob_snapshot) => {
                let action_iterator = self.trader_configs.iter().filter_map(
                    |(trader_id, configs)| {
                        if let Some(config) = configs.get(&(exchange_id, ob_snapshot.traded_pair)) {
                            if config.contains(SubscriptionList::OB_SNAPSHOTS) {
                                let ob_snapshot = Self::create_broker_reply(
                                    *trader_id,
                                    exchange_id,
                                    exchange_dt,
                                    BasicBrokerReply::ExchangeEventNotification(
                                        ExchangeEventNotification::ObSnapshot(
                                            Rc::clone(&ob_snapshot)
                                        )
                                    ),
                                );
                                return Some(ob_snapshot);
                            }
                        }
                        None
                    }
                );
                message_receiver.extend(action_iterator.map(process_action))
            }
            ExchangeEventNotification::TradesStopped(traded_pair) => {
                let action_iterator = self.trader_configs.keys().map(
                    |trader_id| Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        exchange_dt,
                        BasicBrokerReply::ExchangeEventNotification(
                            ExchangeEventNotification::TradesStopped(traded_pair)
                        ),
                    )
                );
                message_receiver.extend(action_iterator.map(process_action))
            }
            ExchangeEventNotification::ExchangeClosed => {
                let action_iterator = self.trader_configs.keys().map(
                    |trader_id| Self::create_broker_reply(
                        *trader_id,
                        exchange_id,
                        exchange_dt,
                        BasicBrokerReply::ExchangeEventNotification(
                            ExchangeEventNotification::ExchangeClosed
                        ),
                    )
                );
                message_receiver.extend(action_iterator.map(process_action))
            }
        }
    }

    fn create_broker_reply(
        trader_id: TraderID,
        exchange_id: ExchangeID,
        event_dt: DateTime,
        content: BasicBrokerReply<Symbol, Settlement>) -> <Self as Agent>::Action
    {
        BrokerAction {
            delay: 0,
            content: BrokerActionKind::BrokerToTrader(
                BasicBrokerToTrader {
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
        content: BasicBrokerRequest<Symbol, Settlement>) -> <Self as Agent>::Action
    {
        BrokerAction {
            delay: 0,
            content: BrokerActionKind::BrokerToExchange(
                BasicBrokerToExchange {
                    exchange_id,
                    content,
                }
            ),
        }
    }
}

pub struct VoidBroker<
    BrokerID: Id,
    TraderID: Id,
    ExchangeID: Id,
    R2B: ReplayToBroker<BrokerID=BrokerID>,
    E2B: ExchangeToBroker<BrokerID=BrokerID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    B2R: BrokerToReplay,
    B2E: BrokerToExchange<ExchangeID=ExchangeID>,
    B2T: BrokerToTrader<TraderID=TraderID>,
    B2B: BrokerToItself,
    SubCfg
> {
    current_dt: DateTime,
    broker_id: BrokerID,
    phantom: PhantomData<(TraderID, ExchangeID, R2B, E2B, T2B, B2R, B2E, B2T, B2B, SubCfg)>,
}

impl<
    BrokerID: Id,
    TraderID: Id,
    ExchangeID: Id,
    R2B: ReplayToBroker<BrokerID=BrokerID>,
    E2B: ExchangeToBroker<BrokerID=BrokerID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    B2R: BrokerToReplay,
    B2E: BrokerToExchange<ExchangeID=ExchangeID>,
    B2T: BrokerToTrader<TraderID=TraderID>,
    B2B: BrokerToItself,
    SubCfg
>
VoidBroker<BrokerID, TraderID, ExchangeID, R2B, E2B, T2B, B2R, B2E, B2T, B2B, SubCfg>
{
    pub fn new(broker_id: BrokerID) -> Self {
        Self {
            current_dt: Date::from_ymd(1970, 1, 1).and_hms(0, 0, 0),
            broker_id,
            phantom: Default::default(),
        }
    }
}

impl<
    BrokerID: Id,
    TraderID: Id,
    ExchangeID: Id,
    R2B: ReplayToBroker<BrokerID=BrokerID>,
    E2B: ExchangeToBroker<BrokerID=BrokerID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    B2R: BrokerToReplay,
    B2E: BrokerToExchange<ExchangeID=ExchangeID>,
    B2T: BrokerToTrader<TraderID=TraderID>,
    B2B: BrokerToItself,
    SubCfg
>
TimeSync for VoidBroker<BrokerID, TraderID, ExchangeID, R2B, E2B, T2B, B2R, B2E, B2T, B2B, SubCfg>
{
    fn current_datetime_mut(&mut self) -> &mut DateTime {
        &mut self.current_dt
    }
}

impl<
    BrokerID: Id,
    TraderID: Id,
    ExchangeID: Id,
    R2B: ReplayToBroker<BrokerID=BrokerID>,
    E2B: ExchangeToBroker<BrokerID=BrokerID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    B2R: BrokerToReplay,
    B2E: BrokerToExchange<ExchangeID=ExchangeID>,
    B2T: BrokerToTrader<TraderID=TraderID>,
    B2B: BrokerToItself,
    SubCfg
>
Latent
for VoidBroker<BrokerID, TraderID, ExchangeID, R2B, E2B, T2B, B2R, B2E, B2T, B2B, SubCfg>
{
    type OuterID = ExchangeID;
    type LatencyGenerator = ConstantLatency<ExchangeID, 0, 0>;

    fn get_latency_generator(&self) -> Self::LatencyGenerator {
        ConstantLatency::new()
    }
}

impl<
    BrokerID: Id,
    TraderID: Id,
    ExchangeID: Id,
    R2B: ReplayToBroker<BrokerID=BrokerID>,
    E2B: ExchangeToBroker<BrokerID=BrokerID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    B2R: BrokerToReplay,
    B2E: BrokerToExchange<ExchangeID=ExchangeID>,
    B2T: BrokerToTrader<TraderID=TraderID>,
    B2B: BrokerToItself,
    SubCfg
>
Named<BrokerID>
for VoidBroker<BrokerID, TraderID, ExchangeID, R2B, E2B, T2B, B2R, B2E, B2T, B2B, SubCfg>
{
    fn get_name(&self) -> BrokerID {
        self.broker_id
    }
}

impl<
    BrokerID: Id,
    TraderID: Id,
    ExchangeID: Id,
    R2B: ReplayToBroker<BrokerID=BrokerID>,
    E2B: ExchangeToBroker<BrokerID=BrokerID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    B2R: BrokerToReplay,
    B2E: BrokerToExchange<ExchangeID=ExchangeID>,
    B2T: BrokerToTrader<TraderID=TraderID>,
    B2B: BrokerToItself, SubCfg
>
Agent
for VoidBroker<BrokerID, TraderID, ExchangeID, R2B, E2B, T2B, B2R, B2E, B2T, B2B, SubCfg> {
    type Action = BrokerAction<B2R, B2E, B2T, B2B>;
}

impl<
    BrokerID: Id,
    TraderID: Id,
    ExchangeID: Id,
    R2B: ReplayToBroker<BrokerID=BrokerID>,
    E2B: ExchangeToBroker<BrokerID=BrokerID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    B2R: BrokerToReplay,
    B2E: BrokerToExchange<ExchangeID=ExchangeID>,
    B2T: BrokerToTrader<TraderID=TraderID>,
    B2B: BrokerToItself,
    SubCfg
>
Broker for VoidBroker<BrokerID, TraderID, ExchangeID, R2B, E2B, T2B, B2R, B2E, B2T, B2B, SubCfg>
{
    type BrokerID = BrokerID;
    type TraderID = TraderID;
    type ExchangeID = ExchangeID;

    type R2B = R2B;
    type E2B = E2B;
    type T2B = T2B;
    type B2R = B2R;
    type B2E = B2E;
    type B2T = B2T;
    type B2B = B2B;
    type SubCfg = SubCfg;

    fn wakeup<KerMsg: Ord>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
        _: Self::B2B,
        _: &mut impl Rng,
    ) {
        unreachable!("{} :: Broker wakeups are not planned", self.current_dt)
    }

    fn process_trader_request<KerMsg: Ord>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
        _: Self::T2B,
        _: Self::TraderID,
        _: &mut impl Rng,
    ) {}

    fn process_exchange_reply<KerMsg: Ord>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
        _: Self::E2B,
        _: Self::ExchangeID,
        _: &mut impl Rng,
    ) {}

    fn process_replay_request<KerMsg: Ord>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
        _: Self::R2B,
        _: &mut impl Rng,
    ) {}

    fn upon_connection_to_exchange(&mut self, _: Self::ExchangeID) {}

    fn register_trader(&mut self, _: Self::TraderID, _: impl IntoIterator<Item=Self::SubCfg>) {}
}

pub type BasicVoidBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement> = VoidBroker<
    BrokerID, TraderID, ExchangeID,
    NeverType<BrokerID>,
    BasicExchangeToBroker<BrokerID, Symbol, Settlement>,
    BasicTraderToBroker<BrokerID, ExchangeID, Symbol, Settlement>,
    Nothing,
    BasicBrokerToExchange<ExchangeID, Symbol, Settlement>,
    BasicBrokerToTrader<TraderID, ExchangeID, Symbol, Settlement>,
    Nothing,
    SubscriptionConfig<ExchangeID, Symbol, Settlement>
>;