use {
    crate::{
        broker::request::BrokerRequest,
        exchange::{
            Exchange, ExchangeAction, ExchangeActionKind,
            reply::{
                CancellationReason,
                CannotBroadcastObState,
                CannotCancelOrder,
                CannotCloseExchange,
                CannotOpenExchange,
                CannotStartTrades,
                CannotStopTrades,
                ExchangeEventNotification,
                ExchangeToBroker,
                ExchangeToBrokerReply,
                ExchangeToReplay,
                ExchangeToReplayReply,
                InabilityToBroadcastObState,
                InabilityToCancelReason,
                InabilityToCloseExchangeReason,
                InabilityToOpenExchangeReason,
                InabilityToStartTrades,
                InabilityToStopTrades,
                LimitOrderEventInfo,
                MarketOrderEventInfo,
                MarketOrderNotFullyExecuted,
                ObSnapshot,
                OrderAccepted,
                OrderCancelled,
                OrderExecuted,
                OrderPartiallyExecuted,
                OrderPlacementDiscarded,
                PlacementDiscardingReason,
            },
        },
        order::{LimitOrderCancelRequest, LimitOrderPlacingRequest, MarketOrderPlacingRequest},
        order_book::{OrderBook, OrderBookEvent, OrderBookEventKind},
        replay::request::ReplayRequest,
        settlement::GetSettlementLag,
        traded_pair::TradedPair,
        types::{
            Date,
            DateTime,
            Direction,
            Identifier,
            Named,
            OrderID,
            PriceStep,
            Size,
            TimeSync,
        },
        utils::{ExpectWith, queue::MessageReceiver},
    },
    std::{
        collections::{hash_map::Entry::*, HashMap},
        iter::{once, once_with},
        rc::Rc,
    },
};

pub struct BasicExchange<
    ExchangeID: Identifier,
    BrokerID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    current_dt: DateTime,
    name: ExchangeID,

    /// [Broker -> [Submitted Order ID -> Internal Order ID]]
    broker_to_order_id: HashMap<
        BrokerID,
        HashMap<(TradedPair<Symbol, Settlement>, OrderID), OrderID>
    >,
    /// [Submitted Order ID -> Internal Order ID]
    replay_order_ids: HashMap<(TradedPair<Symbol, Settlement>, OrderID), OrderID>,

    /// [Internal Order ID ->
    /// (Submitted Order ID, Whether it came from broker ( Broker ID ) or replay (None) )]
    internal_to_submitted: HashMap<OrderID, (OrderID, Option<BrokerID>)>,

    next_order_id: OrderID,
    order_books: HashMap<TradedPair<Symbol, Settlement>, (OrderBook, PriceStep)>,
    is_open: bool,
}

impl<
    ExchangeID: Identifier,
    BrokerID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
>
TimeSync
for BasicExchange<ExchangeID, BrokerID, Symbol, Settlement>
{
    fn current_datetime_mut(&mut self) -> &mut DateTime {
        &mut self.current_dt
    }
}

impl<
    ExchangeID: Identifier,
    BrokerID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
>
Named<ExchangeID>
for BasicExchange<ExchangeID, BrokerID, Symbol, Settlement>
{
    fn get_name(&self) -> ExchangeID {
        self.name
    }
}

impl<
    ExchangeID: Identifier,
    BrokerID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
>
Exchange<ExchangeID, BrokerID, Symbol, Settlement>
for BasicExchange<ExchangeID, BrokerID, Symbol, Settlement>
{
    fn process_broker_request<KM: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KM>,
        process_action: impl FnMut(ExchangeAction<BrokerID, Symbol, Settlement>) -> KM,
        request: BrokerRequest<Symbol, Settlement>,
        broker_id: BrokerID,
    ) {
        let get_broker_id = || broker_id;
        match request
        {
            BrokerRequest::CancelLimitOrder(request) => {
                self.try_cancel_limit_order::<_, _, _, false>(
                    message_receiver, process_action, request, get_broker_id,
                )
            }
            BrokerRequest::PlaceLimitOrder(order) => {
                self.try_place_limit_order::<_, _, _, false>(
                    message_receiver, process_action, order, get_broker_id,
                )
            }
            BrokerRequest::PlaceMarketOrder(order) => {
                self.try_place_market_order::<_, _, _, false>(
                    message_receiver, process_action, order, get_broker_id,
                )
            }
        }
    }

    fn process_replay_request<KM: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KM>,
        process_action: impl FnMut(ExchangeAction<BrokerID, Symbol, Settlement>) -> KM,
        request: ReplayRequest<Symbol, Settlement>,
    ) {
        let get_broker_id_plug = || unreachable!("Replay does not have BrokerID");
        match request
        {
            ReplayRequest::ExchangeOpen => {
                self.try_open(message_receiver, process_action)
            }
            ReplayRequest::StartTrades(traded_pair, price_step) => {
                self.try_start_trades(message_receiver, process_action, traded_pair, price_step)
            }
            ReplayRequest::PlaceMarketOrder(order) => {
                self.try_place_market_order::<_, _, _, true>(
                    message_receiver, process_action, order, get_broker_id_plug,
                )
            }
            ReplayRequest::PlaceLimitOrder(order) => {
                self.try_place_limit_order::<_, _, _, true>(
                    message_receiver, process_action, order, get_broker_id_plug,
                )
            }
            ReplayRequest::CancelLimitOrder(request) => {
                self.try_cancel_limit_order::<_, _, _, true>(
                    message_receiver, process_action, request, get_broker_id_plug,
                )
            }
            ReplayRequest::StopTrades(traded_pair) => {
                self.try_stop_trades(message_receiver, process_action, traded_pair)
            }
            ReplayRequest::ExchangeClosed => {
                self.try_close(message_receiver, process_action)
            }
            ReplayRequest::BroadcastObStateToBrokers(traded_pair) => {
                self.try_broadcast_ob_state(message_receiver, process_action, traded_pair)
            }
        }
    }

    fn connect_broker(&mut self, broker_id: BrokerID) {
        self.broker_to_order_id.insert(broker_id, Default::default());
    }
}

impl<
    ExchangeID: Identifier,
    BrokerID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
>
BasicExchange<ExchangeID, BrokerID, Symbol, Settlement>
{
    pub fn new(name: ExchangeID) -> Self
    {
        BasicExchange {
            current_dt: Date::from_ymd(1970, 01, 01).and_hms(0, 0, 0),
            name,
            broker_to_order_id: Default::default(),
            replay_order_ids: Default::default(),
            internal_to_submitted: Default::default(),
            next_order_id: OrderID(0),
            order_books: Default::default(),
            is_open: false,
        }
    }

    fn try_broadcast_ob_state<KM: Ord>(
        &self,
        mut message_receiver: MessageReceiver<KM>,
        mut process_action: impl FnMut(ExchangeAction<BrokerID, Symbol, Settlement>) -> KM,
        traded_pair: TradedPair<Symbol, Settlement>,
    ) {
        if !self.is_open {
            let reply = Self::create_replay_reply(
                ExchangeToReplayReply::CannotBroadcastObState(
                    CannotBroadcastObState {
                        reason: InabilityToBroadcastObState::ExchangeClosed
                    }
                )
            );
            message_receiver.push(process_action(reply))
        } else if let Some((order_book, _price_step)) = self.order_books.get(&traded_pair) {
            let ob_snapshot = Rc::new(
                ObSnapshot { traded_pair, state: order_book.get_ob_state() }
            );
            let action_iterator = once_with(
                || Self::create_replay_reply(
                    ExchangeToReplayReply::ExchangeEventNotification(
                        ExchangeEventNotification::ObSnapshot(ob_snapshot.clone())
                    )
                )
            ).chain(
                self.broker_to_order_id.keys().map(
                    |broker_id| self.create_broker_reply(
                        *broker_id,
                        ExchangeToBrokerReply::ExchangeEventNotification(
                            ExchangeEventNotification::ObSnapshot(ob_snapshot.clone())
                        ),
                    )
                )
            );
            message_receiver.extend(action_iterator.map(process_action))
        } else {
            let reply = Self::create_replay_reply(
                ExchangeToReplayReply::CannotBroadcastObState(
                    CannotBroadcastObState {
                        reason: InabilityToBroadcastObState::NoSuchTradedPair
                    }
                )
            );
            message_receiver.push(process_action(reply))
        }
    }

    fn try_cancel_limit_order<
        KM: Ord,
        ProcessAction: FnMut(ExchangeAction<BrokerID, Symbol, Settlement>) -> KM,
        GetBrokerID: Fn() -> BrokerID,
        const REPLAY: bool
    >(
        &mut self,
        mut message_receiver: MessageReceiver<KM>,
        mut process_action: ProcessAction,
        request: LimitOrderCancelRequest<Symbol, Settlement>,
        get_broker_id: GetBrokerID,
    ) {
        if !self.is_open {
            let cannot_cancel_order = CannotCancelOrder {
                traded_pair: request.traded_pair,
                order_id: request.order_id,
                reason: InabilityToCancelReason::ExchangeClosed,
            };
            let reply = if REPLAY {
                Self::create_replay_reply(
                    ExchangeToReplayReply::CannotCancelOrder(cannot_cancel_order)
                )
            } else {
                self.create_broker_reply(
                    get_broker_id(),
                    ExchangeToBrokerReply::CannotCancelOrder(cannot_cancel_order),
                )
            };
            message_receiver.push(process_action(reply));
            return;
        };
        let order_id_map = if REPLAY {
            &self.replay_order_ids
        } else if let Some(order_id_map) = self.broker_to_order_id.get(&get_broker_id()) {
            order_id_map
        } else {
            let cannot_cancel_order = CannotCancelOrder {
                traded_pair: request.traded_pair,
                order_id: request.order_id,
                reason: InabilityToCancelReason::BrokerNotConnectedToExchange,
            };
            let reply = self.create_broker_reply(
                get_broker_id(),
                ExchangeToBrokerReply::CannotCancelOrder(cannot_cancel_order),
            );
            message_receiver.push(process_action(reply));
            return;
        };
        let cannot_cancel_order = if let Some(internal_order_id) = order_id_map.get(
            &(request.traded_pair, request.order_id)
        ) {
            if let Some((order_book, _price_step)) = self.order_books.get_mut(&request.traded_pair)
            {
                if let Some((limit_order, direction, price)) = order_book.cancel_limit_order(
                    *internal_order_id
                ) {
                    let order_cancelled = OrderCancelled {
                        traded_pair: request.traded_pair,
                        order_id: request.order_id,
                        reason: CancellationReason::BrokerRequested,
                    };
                    let broker_notification_iterator = self.broker_to_order_id.keys().map(
                        |broker_id| self.create_broker_reply(
                            *broker_id,
                            ExchangeToBrokerReply::ExchangeEventNotification(
                                ExchangeEventNotification::OrderCancelled(LimitOrderEventInfo {
                                    traded_pair: request.traded_pair,
                                    order_id: limit_order.id,
                                    direction,
                                    price,
                                    size: limit_order.size,
                                })
                            ),
                        )
                    );
                    if REPLAY {
                        let replay_reply = || Self::create_replay_reply(
                            ExchangeToReplayReply::OrderCancelled(order_cancelled)
                        );
                        let action_iterator = once_with(replay_reply)
                            .chain(broker_notification_iterator);
                        message_receiver.extend(action_iterator.map(process_action))
                    } else {
                        let replay_notification = || Self::create_replay_reply(
                            ExchangeToReplayReply::ExchangeEventNotification(
                                ExchangeEventNotification::OrderCancelled(
                                    LimitOrderEventInfo {
                                        traded_pair: request.traded_pair,
                                        order_id: limit_order.id,
                                        direction,
                                        price,
                                        size: limit_order.size,
                                    }
                                )
                            )
                        );
                        let broker_reply = || self.create_broker_reply(
                            get_broker_id(),
                            ExchangeToBrokerReply::OrderCancelled(order_cancelled),
                        );
                        let action_iterator = once_with(replay_notification)
                            .chain(once_with(broker_reply))
                            .chain(broker_notification_iterator);
                        message_receiver.extend(action_iterator.map(process_action))
                    };
                    return;
                } else {
                    InabilityToCancelReason::OrderAlreadyExecuted
                }
            } else {
                InabilityToCancelReason::BrokerNotConnectedToExchange
            }
        } else {
            InabilityToCancelReason::OrderHasNotBeenSubmitted
        };
        let cannot_cancel_order = CannotCancelOrder {
            traded_pair: request.traded_pair,
            order_id: request.order_id,
            reason: cannot_cancel_order,
        };
        let reply = if REPLAY {
            Self::create_replay_reply(
                ExchangeToReplayReply::CannotCancelOrder(cannot_cancel_order)
            )
        } else {
            self.create_broker_reply(
                get_broker_id(),
                ExchangeToBrokerReply::CannotCancelOrder(cannot_cancel_order),
            )
        };
        message_receiver.push(process_action(reply))
    }

    fn try_stop_trades<KM: Ord>(
        &mut self,
        mut message_receiver: MessageReceiver<KM>,
        mut process_action: impl FnMut(ExchangeAction<BrokerID, Symbol, Settlement>) -> KM,
        traded_pair: TradedPair<Symbol, Settlement>,
    ) {
        if !self.is_open {
            let reply = Self::create_replay_reply(
                ExchangeToReplayReply::CannotStopTrades(
                    CannotStopTrades {
                        reason: InabilityToStopTrades::ExchangeClosed
                    }
                )
            );
            message_receiver.push(process_action(reply))
        } else if let Occupied(entry) = self.order_books.entry(traded_pair) {
            let (ob, _price_step) = entry.remove();
            let order_cancel_iterator = ob.get_all_ids().into_iter().map(
                |internal_order_id| {
                    let (order_id, from) = self.internal_to_submitted
                        .get(&internal_order_id)
                        .expect_with(
                            || unreachable!(
                                "Cannot find limit order with internal ID: {internal_order_id}"
                            )
                        );
                    let order_cancelled = OrderCancelled {
                        traded_pair,
                        order_id: *order_id,
                        reason: CancellationReason::TradesStopped,
                    };
                    if let Some(broker_id) = from {
                        self.create_broker_reply(
                            *broker_id,
                            ExchangeToBrokerReply::OrderCancelled(order_cancelled),
                        )
                    } else {
                        Self::create_replay_reply(
                            ExchangeToReplayReply::OrderCancelled(order_cancelled)
                        )
                    }
                }
            );
            let trades_stopped_iterator = self.broker_to_order_id.keys().map(
                |broker_id| self.create_broker_reply(
                    *broker_id,
                    ExchangeToBrokerReply::ExchangeEventNotification(
                        ExchangeEventNotification::TradesStopped(traded_pair)
                    ),
                )
            ).chain(
                once_with(
                    || Self::create_replay_reply(
                        ExchangeToReplayReply::ExchangeEventNotification(
                            ExchangeEventNotification::TradesStopped(traded_pair)
                        )
                    )
                )
            );
            let action_iterator = order_cancel_iterator.chain(trades_stopped_iterator);
            message_receiver.extend(action_iterator.map(process_action))
        } else {
            let reply = Self::create_replay_reply(
                ExchangeToReplayReply::CannotStopTrades(
                    CannotStopTrades {
                        reason: InabilityToStopTrades::NoSuchTradedPair
                    }
                )
            );
            message_receiver.push(process_action(reply))
        }
    }

    fn create_replay_reply(content: ExchangeToReplayReply<Symbol, Settlement>) -> ExchangeAction<
        BrokerID, Symbol, Settlement
    > {
        ExchangeAction {
            delay: 0,
            content: ExchangeActionKind::ExchangeToReplay(ExchangeToReplay { content }),
        }
    }

    fn create_broker_reply(
        &self,
        broker_id: BrokerID,
        content: ExchangeToBrokerReply<Symbol, Settlement>) -> ExchangeAction<
        BrokerID, Symbol, Settlement
    > {
        ExchangeAction {
            delay: 0,
            content: ExchangeActionKind::ExchangeToBroker(
                ExchangeToBroker {
                    broker_id,
                    exchange_dt: self.current_dt,
                    content,
                }
            ),
        }
    }

    fn try_open<KM: Ord>(
        &mut self,
        mut message_receiver: MessageReceiver<KM>,
        mut process_action: impl FnMut(ExchangeAction<BrokerID, Symbol, Settlement>) -> KM,
    ) {
        if self.is_open {
            let reply = Self::create_replay_reply(
                ExchangeToReplayReply::CannotOpenExchange(
                    CannotOpenExchange {
                        reason: InabilityToOpenExchangeReason::AlreadyOpen
                    }
                )
            );
            message_receiver.push(process_action(reply))
        } else {
            self.is_open = true;
            let action_iterator = once_with(
                || Self::create_replay_reply(
                    ExchangeToReplayReply::ExchangeEventNotification(
                        ExchangeEventNotification::ExchangeOpen
                    )
                )
            ).chain(
                self.broker_to_order_id.keys().map(
                    |broker_id| self.create_broker_reply(
                        *broker_id,
                        ExchangeToBrokerReply::ExchangeEventNotification(
                            ExchangeEventNotification::ExchangeOpen
                        ),
                    )
                )
            );
            message_receiver.extend(action_iterator.map(process_action))
        }
    }

    fn try_close<KM: Ord>(
        &mut self,
        mut message_receiver: MessageReceiver<KM>,
        mut process_action: impl FnMut(ExchangeAction<BrokerID, Symbol, Settlement>) -> KM,
    ) {
        if self.is_open
        {
            self.is_open = false;
            let broker_notification_iterator = self.broker_to_order_id.iter().map(
                |(broker_id, submitted_to_internal)|
                    once_with(
                        || self.create_broker_reply(
                            *broker_id,
                            ExchangeToBrokerReply::ExchangeEventNotification(
                                ExchangeEventNotification::ExchangeClosed
                            ),
                        )
                    ).chain(
                        submitted_to_internal.keys().map(
                            |(traded_pair, order_id)| self.create_broker_reply(
                                *broker_id,
                                ExchangeToBrokerReply::OrderCancelled(
                                    OrderCancelled {
                                        traded_pair: *traded_pair,
                                        order_id: *order_id,
                                        reason: CancellationReason::ExchangeClosed,
                                    }
                                ),
                            ),
                        )
                    )
            );
            let broker_notification_iterator = broker_notification_iterator.flatten();
            let replay_notification_iterator = once(
                Self::create_replay_reply(
                    ExchangeToReplayReply::ExchangeEventNotification(
                        ExchangeEventNotification::ExchangeClosed
                    )
                )
            ).chain(
                self.replay_order_ids.keys().map(
                    |(traded_pair, order_id)| Self::create_replay_reply(
                        ExchangeToReplayReply::OrderCancelled(
                            OrderCancelled {
                                traded_pair: *traded_pair,
                                order_id: *order_id,
                                reason: CancellationReason::ExchangeClosed,
                            }
                        )
                    )
                )
            );
            let action_iterator = broker_notification_iterator.chain(replay_notification_iterator);
            message_receiver.extend(action_iterator.map(process_action));
            self.broker_to_order_id.values_mut().for_each(HashMap::clear);
            self.replay_order_ids.clear();
            self.internal_to_submitted.clear();
            self.order_books.values_mut().for_each(|(ob, _price_step)| ob.clear());
            self.next_order_id = OrderID(0);
        } else {
            let reply = Self::create_replay_reply(
                ExchangeToReplayReply::CannotCloseExchange(
                    CannotCloseExchange {
                        reason: InabilityToCloseExchangeReason::AlreadyClosed
                    }
                )
            );
            message_receiver.push(process_action(reply))
        }
    }

    fn try_start_trades<KM: Ord>(
        &mut self,
        mut message_receiver: MessageReceiver<KM>,
        mut process_action: impl FnMut(ExchangeAction<BrokerID, Symbol, Settlement>) -> KM,
        traded_pair: TradedPair<Symbol, Settlement>,
        price_step: PriceStep,
    ) {
        if !self.is_open {
            let reply = Self::create_replay_reply(
                ExchangeToReplayReply::CannotStartTrades(
                    CannotStartTrades {
                        traded_pair,
                        reason: InabilityToStartTrades::ExchangeClosed,
                    }
                )
            );
            message_receiver.push(process_action(reply))
        } else if let Vacant(entry) = self.order_books.entry(traded_pair) {
            entry.insert((OrderBook::new(), price_step));
            let broker_notification_iterator = self.broker_to_order_id.keys().map(
                |broker_id| self.create_broker_reply(
                    *broker_id,
                    ExchangeToBrokerReply::ExchangeEventNotification(
                        ExchangeEventNotification::TradesStarted(traded_pair, price_step)
                    ),
                )
            );
            let action_iterator = once_with(
                || Self::create_replay_reply(
                    ExchangeToReplayReply::ExchangeEventNotification(
                        ExchangeEventNotification::TradesStarted(traded_pair, price_step)
                    )
                )
            )
                .chain(broker_notification_iterator);
            message_receiver.extend(action_iterator.map(process_action))
        } else {
            let reply = Self::create_replay_reply(
                ExchangeToReplayReply::CannotStartTrades(
                    CannotStartTrades {
                        traded_pair,
                        reason: InabilityToStartTrades::AlreadyStarted,
                    }
                )
            );
            message_receiver.push(process_action(reply))
        }
    }

    fn try_place_market_order<
        KM: Ord,
        ProcessAction: FnMut(ExchangeAction<BrokerID, Symbol, Settlement>) -> KM,
        GetBrokerID: Fn() -> BrokerID,
        const REPLAY: bool
    >(
        &mut self,
        mut message_receiver: MessageReceiver<KM>,
        mut process_action: ProcessAction,
        order: MarketOrderPlacingRequest<Symbol, Settlement>,
        get_broker_id: GetBrokerID,
    ) {
        if !self.is_open {
            let order_discarded = OrderPlacementDiscarded {
                traded_pair: order.traded_pair,
                order_id: order.order_id,
                reason: PlacementDiscardingReason::ExchangeClosed,
            };
            let reply = if REPLAY {
                Self::create_replay_reply(
                    ExchangeToReplayReply::OrderPlacementDiscarded(order_discarded)
                )
            } else {
                self.create_broker_reply(
                    get_broker_id(),
                    ExchangeToBrokerReply::OrderPlacementDiscarded(order_discarded),
                )
            };
            message_receiver.push(process_action(reply));
            return;
        }
        if order.size == Size(0) {
            let order_discarded = OrderPlacementDiscarded {
                traded_pair: order.traded_pair,
                order_id: order.order_id,
                reason: PlacementDiscardingReason::ZeroSize,
            };
            let reply = if REPLAY {
                Self::create_replay_reply(
                    ExchangeToReplayReply::OrderPlacementDiscarded(order_discarded)
                )
            } else {
                self.create_broker_reply(
                    get_broker_id(),
                    ExchangeToBrokerReply::OrderPlacementDiscarded(order_discarded),
                )
            };
            message_receiver.push(process_action(reply));
            return;
        }
        let order_id_map = if REPLAY {
            &mut self.replay_order_ids
        } else if let Some(order_id_map) = self.broker_to_order_id.get_mut(&get_broker_id()) {
            order_id_map
        } else {
            let order_discarded = OrderPlacementDiscarded {
                traded_pair: order.traded_pair,
                order_id: order.order_id,
                reason: PlacementDiscardingReason::BrokerNotConnectedToExchange,
            };
            let reply = self.create_broker_reply(
                get_broker_id(),
                ExchangeToBrokerReply::OrderPlacementDiscarded(order_discarded),
            );
            message_receiver.push(process_action(reply));
            return;
        };
        let order_id_map = if let Vacant(entry) = order_id_map.entry(
            (order.traded_pair, order.order_id)
        ) {
            entry
        } else {
            let order_discarded = OrderPlacementDiscarded {
                traded_pair: order.traded_pair,
                order_id: order.order_id,
                reason: PlacementDiscardingReason::OrderWithSuchIDAlreadySubmitted,
            };
            let reply = if REPLAY {
                Self::create_replay_reply(
                    ExchangeToReplayReply::OrderPlacementDiscarded(order_discarded)
                )
            } else {
                self.create_broker_reply(
                    get_broker_id(),
                    ExchangeToBrokerReply::OrderPlacementDiscarded(order_discarded),
                )
            };
            message_receiver.push(process_action(reply));
            return;
        };
        if let Some((order_book, _price_step)) = self.order_books.get_mut(&order.traded_pair)
        {
            let internal_order_id = self.next_order_id;
            self.next_order_id += OrderID(1);
            self.internal_to_submitted.insert(
                internal_order_id,
                (order.order_id, if REPLAY { None } else { Some(get_broker_id()) }),
            );
            order_id_map.insert(internal_order_id);

            let mut remaining_size = order.size;
            match (order.dummy, order.direction) {
                (false, Direction::Buy) => {
                    let order_book_events = order_book.insert_market_order::<false, true>(
                        order.size
                    );
                    order_book_events.into_iter()
                        .for_each(
                            |event| self.interpret_ob_event::<_, _, _, false, true, REPLAY>(
                                &mut message_receiver,
                                &mut process_action,
                                &mut remaining_size,
                                event,
                                order.traded_pair,
                                order.order_id,
                                &get_broker_id,
                            )
                        );
                }
                (false, Direction::Sell) => {
                    let order_book_events = order_book.insert_market_order::<false, false>(
                        order.size
                    );
                    order_book_events.into_iter()
                        .for_each(
                            |event| self.interpret_ob_event::<_, _, _, false, false, REPLAY>(
                                &mut message_receiver,
                                &mut process_action,
                                &mut remaining_size,
                                event,
                                order.traded_pair,
                                order.order_id,
                                &get_broker_id,
                            )
                        );
                }
                (true, Direction::Buy) => {
                    let order_book_events = order_book.insert_market_order::<true, true>(
                        order.size
                    );
                    order_book_events.into_iter()
                        .for_each(
                            |event| self.interpret_ob_event::<_, _, _, true, true, REPLAY>(
                                &mut message_receiver,
                                &mut process_action,
                                &mut remaining_size,
                                event,
                                order.traded_pair,
                                order.order_id,
                                &get_broker_id,
                            )
                        );
                }
                (true, Direction::Sell) => {
                    let order_book_events = order_book.insert_market_order::<true, false>(
                        order.size
                    );
                    order_book_events.into_iter()
                        .for_each(
                            |event| self.interpret_ob_event::<_, _, _, true, false, REPLAY>(
                                &mut message_receiver,
                                &mut process_action,
                                &mut remaining_size,
                                event,
                                order.traded_pair,
                                order.order_id,
                                &get_broker_id,
                            )
                        );
                }
            }
            if remaining_size != Size(0) {
                let not_fully_executed = MarketOrderNotFullyExecuted {
                    traded_pair: order.traded_pair,
                    order_id: order.order_id,
                    remaining_size,
                };
                let notification = if REPLAY {
                    Self::create_replay_reply(
                        ExchangeToReplayReply::MarketOrderNotFullyExecuted(
                            not_fully_executed
                        )
                    )
                } else {
                    self.create_broker_reply(
                        get_broker_id(),
                        ExchangeToBrokerReply::MarketOrderNotFullyExecuted(
                            not_fully_executed
                        ),
                    )
                };
                message_receiver.push(process_action(notification))
            }
        } else {
            let order_discarded = OrderPlacementDiscarded {
                traded_pair: order.traded_pair,
                order_id: order.order_id,
                reason: PlacementDiscardingReason::NoSuchTradedPair,
            };
            let reply = if REPLAY {
                Self::create_replay_reply(
                    ExchangeToReplayReply::OrderPlacementDiscarded(order_discarded)
                )
            } else {
                self.create_broker_reply(
                    get_broker_id(),
                    ExchangeToBrokerReply::OrderPlacementDiscarded(order_discarded),
                )
            };
            message_receiver.push(process_action(reply))
        }
    }

    fn try_place_limit_order<
        KM: Ord,
        ProcessAction: FnMut(ExchangeAction<BrokerID, Symbol, Settlement>) -> KM,
        GetBrokerID: Fn() -> BrokerID,
        const REPLAY: bool
    >(
        &mut self,
        mut message_receiver: MessageReceiver<KM>,
        mut process_action: ProcessAction,
        order: LimitOrderPlacingRequest<Symbol, Settlement>,
        get_broker_id: GetBrokerID,
    ) {
        if !self.is_open {
            let order_discarded = OrderPlacementDiscarded {
                traded_pair: order.traded_pair,
                order_id: order.order_id,
                reason: PlacementDiscardingReason::ExchangeClosed,
            };
            let reply = if REPLAY {
                Self::create_replay_reply(
                    ExchangeToReplayReply::OrderPlacementDiscarded(order_discarded)
                )
            } else {
                self.create_broker_reply(
                    get_broker_id(),
                    ExchangeToBrokerReply::OrderPlacementDiscarded(order_discarded),
                )
            };
            message_receiver.push(process_action(reply));
            return;
        }
        if order.size == Size(0) {
            let order_discarded = OrderPlacementDiscarded {
                traded_pair: order.traded_pair,
                order_id: order.order_id,
                reason: PlacementDiscardingReason::ZeroSize,
            };
            let reply = if REPLAY {
                Self::create_replay_reply(
                    ExchangeToReplayReply::OrderPlacementDiscarded(order_discarded)
                )
            } else {
                self.create_broker_reply(
                    get_broker_id(),
                    ExchangeToBrokerReply::OrderPlacementDiscarded(order_discarded),
                )
            };
            message_receiver.push(process_action(reply));
            return;
        }
        let order_id_map = if REPLAY {
            &mut self.replay_order_ids
        } else if let Some(order_id_map) = self.broker_to_order_id.get_mut(&get_broker_id()) {
            order_id_map
        } else {
            let order_discarded = OrderPlacementDiscarded {
                traded_pair: order.traded_pair,
                order_id: order.order_id,
                reason: PlacementDiscardingReason::BrokerNotConnectedToExchange,
            };
            let reply = self.create_broker_reply(
                get_broker_id(),
                ExchangeToBrokerReply::OrderPlacementDiscarded(order_discarded),
            );
            message_receiver.push(process_action(reply));
            return;
        };
        let order_id_map = if let Vacant(entry) = order_id_map.entry(
            (order.traded_pair, order.order_id)
        ) {
            entry
        } else {
            let order_discarded = OrderPlacementDiscarded {
                traded_pair: order.traded_pair,
                order_id: order.order_id,
                reason: PlacementDiscardingReason::OrderWithSuchIDAlreadySubmitted,
            };
            let reply = if REPLAY {
                Self::create_replay_reply(
                    ExchangeToReplayReply::OrderPlacementDiscarded(order_discarded)
                )
            } else {
                self.create_broker_reply(
                    get_broker_id(),
                    ExchangeToBrokerReply::OrderPlacementDiscarded(order_discarded),
                )
            };
            message_receiver.push(process_action(reply));
            return;
        };
        if let Some((order_book, _price_step)) = self.order_books.get_mut(&order.traded_pair)
        {
            let internal_order_id = self.next_order_id;
            self.next_order_id += OrderID(1);
            self.internal_to_submitted.insert(
                internal_order_id,
                (order.order_id, if REPLAY { None } else { Some(get_broker_id()) }),
            );
            order_id_map.insert(internal_order_id);

            let mut remaining_size = order.size;
            match (order.dummy, order.direction) {
                (false, Direction::Buy) => {
                    let order_book_events = order_book.insert_limit_order::<false, true>(
                        self.current_dt, internal_order_id, order.price, order.size,
                    );
                    order_book_events.into_iter()
                        .for_each(
                            |event| self.interpret_ob_event::<_, _, _, false, true, REPLAY>(
                                &mut message_receiver,
                                &mut process_action,
                                &mut remaining_size,
                                event,
                                order.traded_pair,
                                order.order_id,
                                &get_broker_id,
                            )
                        );
                }
                (false, Direction::Sell) => {
                    let order_book_events = order_book.insert_limit_order::<false, false>(
                        self.current_dt, internal_order_id, order.price, order.size,
                    );
                    order_book_events.into_iter()
                        .for_each(
                            |event| self.interpret_ob_event::<_, _, _, false, false, REPLAY>(
                                &mut message_receiver,
                                &mut process_action,
                                &mut remaining_size,
                                event,
                                order.traded_pair,
                                order.order_id,
                                &get_broker_id,
                            )
                        );
                }
                (true, Direction::Buy) => {
                    let order_book_events = order_book.insert_limit_order::<true, true>(
                        self.current_dt, internal_order_id, order.price, order.size,
                    );
                    order_book_events.into_iter()
                        .for_each(
                            |event| self.interpret_ob_event::<_, _, _, true, true, REPLAY>(
                                &mut message_receiver,
                                &mut process_action,
                                &mut remaining_size,
                                event,
                                order.traded_pair,
                                order.order_id,
                                &get_broker_id,
                            )
                        );
                }
                (true, Direction::Sell) => {
                    let order_book_events = order_book.insert_limit_order::<true, false>(
                        self.current_dt, internal_order_id, order.price, order.size,
                    );
                    order_book_events.into_iter()
                        .for_each(
                            |event| self.interpret_ob_event::<_, _, _, true, false, REPLAY>(
                                &mut message_receiver,
                                &mut process_action,
                                &mut remaining_size,
                                event,
                                order.traded_pair,
                                order.order_id,
                                &get_broker_id,
                            )
                        );
                }
            }
            let order_accepted = OrderAccepted {
                traded_pair: order.traded_pair,
                order_id: order.order_id,
            };
            let reply = if REPLAY {
                Self::create_replay_reply(
                    ExchangeToReplayReply::OrderAccepted(order_accepted)
                )
            } else {
                self.create_broker_reply(
                    get_broker_id(),
                    ExchangeToBrokerReply::OrderAccepted(order_accepted),
                )
            };
            message_receiver.push(process_action(reply))
        } else {
            let order_discarded = OrderPlacementDiscarded {
                traded_pair: order.traded_pair,
                order_id: order.order_id,
                reason: PlacementDiscardingReason::NoSuchTradedPair,
            };
            let reply = if REPLAY {
                Self::create_replay_reply(
                    ExchangeToReplayReply::OrderPlacementDiscarded(order_discarded)
                )
            } else {
                self.create_broker_reply(
                    get_broker_id(),
                    ExchangeToBrokerReply::OrderPlacementDiscarded(order_discarded),
                )
            };
            message_receiver.push(process_action(reply))
        }
    }

    fn interpret_ob_event<
        KM: Ord,
        ProcessAction: FnMut(ExchangeAction<BrokerID, Symbol, Settlement>) -> KM,
        GetBrokerID: Fn() -> BrokerID,
        const DUMMY: bool,
        const BUY: bool,
        const REPLAY: bool
    >(
        &self,
        message_receiver: &mut MessageReceiver<KM>,
        mut process_action: ProcessAction,
        remaining_size: &mut Size,
        event: OrderBookEvent,
        traded_pair: TradedPair<Symbol, Settlement>,
        new_order_id: OrderID,
        get_broker_id: &GetBrokerID,
    ) {
        let create_broker_notification = || ExchangeToBrokerReply::ExchangeEventNotification(
            ExchangeEventNotification::TradeExecuted(
                MarketOrderEventInfo {
                    traded_pair,
                    direction: if BUY { Direction::Buy } else { Direction::Sell },
                    price: event.price,
                    size: event.size,
                }
            )
        );
        let create_replay_notification = || ExchangeToReplayReply::ExchangeEventNotification(
            ExchangeEventNotification::TradeExecuted(
                MarketOrderEventInfo {
                    traded_pair,
                    direction: if BUY { Direction::Buy } else { Direction::Sell },
                    price: event.price,
                    size: event.size,
                }
            )
        );

        match event.kind
        {
            OrderBookEventKind::OldOrderExecuted(order_id) => {
                if let Some((order_id, from)) = self.internal_to_submitted.get(&order_id) {
                    let order_executed = OrderExecuted {
                        traded_pair,
                        order_id: *order_id,
                        price: event.price,
                        size: event.size,
                    };
                    let notification = if let Some(broker_id) = from {
                        self.create_broker_reply(
                            *broker_id,
                            ExchangeToBrokerReply::OrderExecuted(order_executed),
                        )
                    } else {
                        Self::create_replay_reply(
                            ExchangeToReplayReply::OrderExecuted(order_executed)
                        )
                    };
                    message_receiver.push(process_action(notification))
                } else {
                    panic!("Cannot find limit order with internal ID {order_id}")
                }
            }
            OrderBookEventKind::OldOrderPartiallyExecuted(order_id) => {
                if let Some((order_id, from)) = self.internal_to_submitted.get(&order_id) {
                    let order_partially_executed = OrderPartiallyExecuted {
                        traded_pair,
                        order_id: *order_id,
                        price: event.price,
                        size: event.size,
                    };
                    let notification = if let Some(broker_id) = from {
                        self.create_broker_reply(
                            *broker_id,
                            ExchangeToBrokerReply::OrderPartiallyExecuted(
                                order_partially_executed
                            ),
                        )
                    } else {
                        Self::create_replay_reply(
                            ExchangeToReplayReply::OrderPartiallyExecuted(
                                order_partially_executed
                            )
                        )
                    };
                    message_receiver.push(process_action(notification))
                } else {
                    panic!("Cannot find limit order with internal ID {order_id}")
                }
            }
            OrderBookEventKind::NewOrderPartiallyExecuted => {
                *remaining_size -= event.size;
                let order_partially_executed = OrderPartiallyExecuted {
                    traded_pair,
                    order_id: new_order_id,
                    price: event.price,
                    size: event.size,
                };
                let reply = if REPLAY {
                    Self::create_replay_reply(
                        ExchangeToReplayReply::OrderPartiallyExecuted(
                            order_partially_executed
                        )
                    )
                } else {
                    self.create_broker_reply(
                        get_broker_id(),
                        ExchangeToBrokerReply::OrderPartiallyExecuted(
                            order_partially_executed
                        ),
                    )
                };
                if DUMMY {
                    message_receiver.push(process_action(reply))
                } else if REPLAY {
                    let broker_notification_iterator = self.broker_to_order_id.keys().map(
                        |broker_id| self.create_broker_reply(
                            *broker_id,
                            create_broker_notification(),
                        )
                    );
                    message_receiver.extend(
                        once(reply)
                            .chain(broker_notification_iterator)
                            .map(process_action)
                    )
                } else {
                    let replay_notification = Self::create_replay_reply(
                        create_replay_notification()
                    );
                    let broker_notification_iterator = self.broker_to_order_id.keys()
                        .map(
                            |broker_id| self.create_broker_reply(
                                *broker_id,
                                create_broker_notification(),
                            )
                        );
                    message_receiver.extend(
                        [reply, replay_notification]
                            .into_iter()
                            .chain(broker_notification_iterator)
                            .map(process_action)
                    )
                }
            }
            OrderBookEventKind::NewOrderExecuted => {
                *remaining_size -= event.size;
                let order_executed = OrderExecuted {
                    traded_pair,
                    order_id: new_order_id,
                    price: event.price,
                    size: event.size,
                };
                let reply = if REPLAY {
                    Self::create_replay_reply(
                        ExchangeToReplayReply::OrderExecuted(order_executed)
                    )
                } else {
                    self.create_broker_reply(
                        get_broker_id(),
                        ExchangeToBrokerReply::OrderExecuted(order_executed),
                    )
                };
                if DUMMY {
                    message_receiver.push(process_action(reply))
                } else if REPLAY {
                    let broker_notification_iterator = self.broker_to_order_id.keys().map(
                        |broker_id| self.create_broker_reply(
                            *broker_id,
                            create_broker_notification(),
                        )
                    );
                    message_receiver.extend(
                        once(reply)
                            .chain(broker_notification_iterator)
                            .map(process_action)
                    )
                } else {
                    let replay_notification = Self::create_replay_reply(
                        create_replay_notification()
                    );
                    let broker_notification_iterator = self.broker_to_order_id.keys()
                        .map(
                            |broker_id| self.create_broker_reply(
                                *broker_id,
                                create_broker_notification(),
                            )
                        );
                    message_receiver.extend(
                        [reply, replay_notification]
                            .into_iter()
                            .chain(broker_notification_iterator)
                            .map(process_action)
                    )
                }
            }
        }
    }
}