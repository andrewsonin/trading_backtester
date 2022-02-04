use {
    crate::{
        concrete::{
            input::one_tick::OneTickTradedPairReader,
            message::{
                exchange::reply::{
                    BasicExchangeToReplay,
                    BasicExchangeToReplayReply,
                    ExchangeEventNotification,
                },
                replay::request::{BasicReplayRequest, BasicReplayToExchange},
            },
            traded_pair::TradedPair,
            types::{OrderID, PriceStep},
        },
        interface::{
            message::{
                BrokerToReplay,
                ExchangeToReplay,
                ReplayToBroker,
                ReplayToExchange,
                ReplayToItself,
            },
            replay::{Replay, ReplayAction, ReplayActionKind},
        },
        types::{
            Date,
            DateTime,
            Duration,
            Id,
            NeverType,
            Nothing,
            TimeSync,
        },
        utils::{
            queue::{LessElementBinaryHeap, MessageReceiver},
        },
    },
    rand::Rng,
    settlement::GetSettlementLag,
    std::{
        cmp::Reverse,
        collections::{HashMap, HashSet},
        io::Write,
        marker::PhantomData,
        num::NonZeroU64,
    },
};

pub mod settlement;

pub trait GetNextObSnapshotDelay<
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
> {
    fn get_ob_snapshot_delay(
        &mut self,
        exchange_id: ExchangeID,
        traded_pair: TradedPair<Symbol, Settlement>,
        rng: &mut impl Rng,
        current_dt: DateTime) -> Option<NonZeroU64>;
}

pub struct OneTickReplay<
    BrokerID: Id,
    ExchangeID: Id,
    Symbol: Id,
    ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>,
    Settlement: GetSettlementLag
> {
    current_dt: DateTime,
    traded_pair_readers: Vec<OneTickTradedPairReader<ExchangeID, Symbol, Settlement>>,
    action_queue: LessElementBinaryHeap<
        (
            ReplayAction<
                Nothing,
                BasicReplayToExchange<ExchangeID, Symbol, Settlement>,
                NeverType<BrokerID>
            >,
            i64
        )
    >,

    active_traded_pairs: HashSet<(ExchangeID, TradedPair<Symbol, Settlement>)>,

    next_order_id: OrderID,

    ob_snapshot_delay_scheduler: ObSnapshotDelay,

    phantom: PhantomData<BrokerID>,
}

#[derive(Copy, Clone)]
pub struct ExchangeSession<ExchangeID: Id> {
    pub exchange_id: ExchangeID,
    pub open_dt: DateTime,
    pub close_dt: DateTime,
}

#[derive(Copy, Clone)]
pub struct TradedPairLifetime<
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag>
{
    pub exchange_id: ExchangeID,
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub price_step: PriceStep,
    pub start_dt: DateTime,
    pub stop_dt: Option<DateTime>,
}

impl<
    BrokerID: Id,
    ExchangeID: Id,
    Symbol: Id,
    ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>,
    Settlement: GetSettlementLag
>
OneTickReplay<BrokerID, ExchangeID, Symbol, ObSnapshotDelay, Settlement>
{
    pub fn new<TPR, EOC, TPC>(
        start_dt: DateTime,
        traded_pair_readers: TPR,
        exchange_open_close_events: EOC,
        traded_pair_creation_events: TPC,
        ob_snapshot_delay_scheduler: ObSnapshotDelay) -> Self
        where TPR: IntoIterator<Item=OneTickTradedPairReader<ExchangeID, Symbol, Settlement>>,
              EOC: IntoIterator<Item=ExchangeSession<ExchangeID>>,
              TPC: IntoIterator<Item=TradedPairLifetime<ExchangeID, Symbol, Settlement>>
    {
        let mut prev_dt: HashMap<ExchangeID, DateTime> = Default::default();
        let open_close_iterator = exchange_open_close_events.into_iter().map(
            |ExchangeSession { exchange_id, open_dt, close_dt }| {
                let prev_dt = prev_dt.entry(exchange_id).or_insert_with(
                    || {
                        if open_dt < start_dt {
                            panic!(
                                "Exchange {exchange_id} open_dt {open_dt} is less \
                                than start_dt {start_dt}"
                            )
                        };
                        start_dt
                    }
                );
                if open_dt < *prev_dt {
                    panic!(
                        "Exchange {exchange_id} open/close datetime pairs \
                        are not stored in the ascending order"
                    )
                }
                if close_dt < open_dt {
                    panic!(
                        "Exchange {exchange_id} close datetime {close_dt} is less than \
                        the corresponding exchange open datetime {open_dt}"
                    )
                }
                *prev_dt = close_dt;
                let open_event = ReplayAction {
                    datetime: open_dt,
                    content: ReplayActionKind::ReplayToExchange(
                        BasicReplayToExchange {
                            exchange_id,
                            content: BasicReplayRequest::ExchangeOpen,
                        }
                    ),
                };
                let close_event = ReplayAction {
                    datetime: close_dt,
                    content: ReplayActionKind::ReplayToExchange(
                        BasicReplayToExchange {
                            exchange_id,
                            content: BasicReplayRequest::ExchangeClosed,
                        }
                    ),
                };
                [open_event, close_event].into_iter()
            }
        );
        let traded_pair_creation_iterator = traded_pair_creation_events.into_iter().map(
            |TradedPairLifetime { exchange_id, traded_pair, price_step, start_dt, stop_dt }|
                {
                    let start_trades = ReplayAction {
                        datetime: start_dt,
                        content: ReplayActionKind::ReplayToExchange(
                            BasicReplayToExchange {
                                exchange_id,
                                content: BasicReplayRequest::StartTrades(traded_pair, price_step),
                            }
                        ),
                    };
                    if let Some(stop_dt) = stop_dt {
                        let stop_trades = ReplayAction {
                            datetime: stop_dt,
                            content: ReplayActionKind::ReplayToExchange(
                                BasicReplayToExchange {
                                    exchange_id,
                                    content: BasicReplayRequest::StopTrades(traded_pair),
                                }
                            ),
                        };
                        vec![start_trades, stop_trades]
                    } else {
                        vec![start_trades]
                    }
                }
        );
        let mut next_order_id = OrderID(0);
        let (first_events, traded_pair_readers): (Vec<_>, _) = traded_pair_readers.into_iter()
            .enumerate()
            .map(
                |(i, mut pair_reader)| {
                    let first_event = pair_reader.next(&mut next_order_id).unwrap_or_else(
                        || panic!("Traded pair reader {i} is empty")
                    );
                    (Reverse((first_event, i as i64)), pair_reader)
                }
            )
            .unzip();
        Self {
            current_dt: start_dt,
            action_queue: LessElementBinaryHeap(
                open_close_iterator
                    .flatten()
                    .chain(traded_pair_creation_iterator.flatten())
                    .map(|action| Reverse((action, -1)))
                    .chain(first_events)
                    .collect()
            ),
            traded_pair_readers,
            ob_snapshot_delay_scheduler,
            active_traded_pairs: Default::default(),
            next_order_id,
            phantom: Default::default(),
        }
    }
}

impl<
    BrokerID: Id,
    ExchangeID: Id,
    Symbol: Id,
    ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>,
    Settlement: GetSettlementLag
>
TimeSync for OneTickReplay<BrokerID, ExchangeID, Symbol, ObSnapshotDelay, Settlement>
{
    fn current_datetime_mut(&mut self) -> &mut DateTime {
        &mut self.current_dt
    }
}

impl<
    BrokerID: Id,
    ExchangeID: Id,
    Symbol: Id,
    ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>,
    Settlement: GetSettlementLag
>
Iterator for OneTickReplay<BrokerID, ExchangeID, Symbol, ObSnapshotDelay, Settlement>
{
    type Item = ReplayAction<
        Nothing,
        BasicReplayToExchange<ExchangeID, Symbol, Settlement>,
        NeverType<BrokerID>
    >;

    fn next(&mut self) -> Option<Self::Item>
    {
        if let Some((action, reader_idx)) = self.action_queue.pop() {
            if reader_idx != -1 {
                if let Some(next_action) = self.traded_pair_readers
                    .get_mut(reader_idx as usize)
                    .unwrap_or_else(|| unreachable!("Index is out of bounds"))
                    .next(&mut self.next_order_id)
                {
                    self.action_queue.push((next_action, reader_idx))
                }
            }
            Some(action)
        } else {
            None
        }
    }
}

impl<
    BrokerID: Id,
    ExchangeID: Id,
    Symbol: Id,
    ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>,
    Settlement: GetSettlementLag
>
Replay
for OneTickReplay<BrokerID, ExchangeID, Symbol, ObSnapshotDelay, Settlement>
{
    type ExchangeID = ExchangeID;
    type BrokerID = BrokerID;

    type E2R = BasicExchangeToReplay<Symbol, Settlement>;
    type B2R = Nothing;
    type R2R = Nothing;
    type R2E = BasicReplayToExchange<ExchangeID, Symbol, Settlement>;
    type R2B = NeverType<BrokerID>;

    fn wakeup<KerMsg: Ord>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl Fn(Self::Item) -> KerMsg,
        _: Self::R2R,
        _: &mut impl Rng,
    ) {
        unreachable!("{} :: Replay wakeups are not planned", self.current_dt)
    }

    fn handle_exchange_reply<KerMsg: Ord>(
        &mut self,
        mut message_receiver: MessageReceiver<KerMsg>,
        process_action: impl Fn(Self::Item) -> KerMsg,
        reply: Self::E2R,
        exchange_id: ExchangeID,
        rng: &mut impl Rng,
    ) {
        let mut get_ob_snapshot_delay = |traded_pair| {
            if let Some(delay) = self.ob_snapshot_delay_scheduler.get_ob_snapshot_delay(
                exchange_id, traded_pair, rng, self.current_dt,
            ) {
                let action = ReplayAction {
                    datetime: self.current_dt + Duration::nanoseconds(delay.get() as i64),
                    content: ReplayActionKind::ReplayToExchange(
                        BasicReplayToExchange {
                            exchange_id,
                            content: BasicReplayRequest::BroadcastObStateToBrokers(traded_pair),
                        }
                    ),
                };
                Some(action)
            } else {
                None
            }
        };
        match reply.content {
            BasicExchangeToReplayReply::ExchangeEventNotification(notification) => {
                match notification
                {
                    ExchangeEventNotification::ExchangeOpen => {
                        let action_iterator = self.active_traded_pairs.iter().filter_map(
                            |(tp_exchange_id, traded_pair)| if *tp_exchange_id != exchange_id {
                                None
                            } else {
                                get_ob_snapshot_delay(*traded_pair)
                            }
                        );
                        message_receiver.extend(action_iterator.map(process_action))
                    }
                    ExchangeEventNotification::TradesStarted(traded_pair, _price_step) => {
                        if !self.active_traded_pairs.insert((exchange_id, traded_pair)) {
                            panic!(
                                "Trades for traded pair already started: \
                                {exchange_id} {traded_pair:?}"
                            )
                        }
                        if let Some(action) = get_ob_snapshot_delay(traded_pair) {
                            message_receiver.push(process_action(action))
                        }
                    }
                    ExchangeEventNotification::ObSnapshot(snapshot) => {
                        if let Some(action) = get_ob_snapshot_delay(snapshot.traded_pair) {
                            message_receiver.push(process_action(action))
                        }
                    }
                    ExchangeEventNotification::TradesStopped(traded_pair) => {
                        if !self.active_traded_pairs.remove(&(exchange_id, traded_pair)) {
                            panic!(
                                "Trades for traded pair already stopped or not ever started: \
                                {exchange_id} {traded_pair:?}"
                            )
                        }
                        self.traded_pair_readers.iter_mut()
                            .filter(|reader| reader.exchange_id == exchange_id
                                && reader.traded_pair == traded_pair)
                            .for_each(OneTickTradedPairReader::clear)
                    }
                    _ => {}
                }
            }
            BasicExchangeToReplayReply::CannotCancelOrder(cannot_cancel) => {
                let reader = self.traded_pair_readers.iter_mut()
                    .skip_while(|reader| reader.exchange_id != exchange_id
                        || reader.traded_pair != cannot_cancel.traded_pair)
                    .next()
                    .unwrap_or_else(
                        || unreachable!(
                            "Cannot find corresponding traded pair reader for {cannot_cancel:?}"
                        )
                    );
                if let Some(err_log_file) = &mut reader.err_log_file {
                    if let Some(order_id) = reader.limit_submitted_to_internal
                        .get(&cannot_cancel.order_id)
                    {
                        writeln!(
                            err_log_file,
                            "{} :: Cannot cancel limit order with ID {order_id} since {}",
                            self.current_dt,
                            cannot_cancel.reason
                        )
                    } else {
                        writeln!(
                            err_log_file,
                            "{} :: Cannot cancel limit order with internal ID {} since {}",
                            self.current_dt,
                            cannot_cancel.order_id,
                            cannot_cancel.reason
                        )
                    }.unwrap_or_else(
                        |err| panic!("Cannot write to file {err_log_file:?}. Error: {err}")
                    )
                }
            }
            BasicExchangeToReplayReply::OrderPlacementDiscarded(_) |
            BasicExchangeToReplayReply::CannotOpenExchange(_) |
            BasicExchangeToReplayReply::CannotStartTrades(_) |
            BasicExchangeToReplayReply::CannotCloseExchange(_) |
            BasicExchangeToReplayReply::CannotStopTrades(_) => {
                panic!("{} :: {reply:?}. Exchange {exchange_id}", self.current_dt)
            }
            _ => {}
        }
    }

    fn handle_broker_reply<KerMsg: Ord>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl Fn(Self::Item) -> KerMsg,
        _: Self::B2R,
        _: Self::BrokerID,
        _: &mut impl Rng,
    ) {
        unreachable!(
            "{} :: OneTickReplay did not plan to communicate with brokers",
            self.current_dt
        )
    }
}

pub struct VoidReplay<
    BrokerID: Id,
    ExchangeID: Id,
    E2R: ExchangeToReplay,
    B2R: BrokerToReplay,
    R2R: ReplayToItself,
    R2E: ReplayToExchange<ExchangeID=ExchangeID>,
    R2B: ReplayToBroker<BrokerID=BrokerID>
> {
    current_dt: DateTime,
    phantom: PhantomData<(ExchangeID, BrokerID, E2R, B2R, R2R, R2E, R2B)>,
}

impl<
    BrokerID: Id,
    ExchangeID: Id,
    E2R: ExchangeToReplay,
    B2R: BrokerToReplay,
    R2R: ReplayToItself,
    R2E: ReplayToExchange<ExchangeID=ExchangeID>,
    R2B: ReplayToBroker<BrokerID=BrokerID>
>
VoidReplay<BrokerID, ExchangeID, E2R, B2R, R2R, R2E, R2B>
{
    pub fn new() -> Self {
        Self {
            current_dt: Date::from_ymd(1970, 1, 1).and_hms(0, 0, 0),
            phantom: Default::default(),
        }
    }
}

impl<
    BrokerID: Id,
    ExchangeID: Id,
    E2R: ExchangeToReplay,
    B2R: BrokerToReplay,
    R2R: ReplayToItself,
    R2E: ReplayToExchange<ExchangeID=ExchangeID>,
    R2B: ReplayToBroker<BrokerID=BrokerID>
>
TimeSync for VoidReplay<BrokerID, ExchangeID, E2R, B2R, R2R, R2E, R2B>
{
    fn current_datetime_mut(&mut self) -> &mut DateTime {
        &mut self.current_dt
    }
}

impl<
    BrokerID: Id,
    ExchangeID: Id,
    E2R: ExchangeToReplay,
    B2R: BrokerToReplay,
    R2R: ReplayToItself,
    R2E: ReplayToExchange<ExchangeID=ExchangeID>,
    R2B: ReplayToBroker<BrokerID=BrokerID>
>
Iterator for VoidReplay<BrokerID, ExchangeID, E2R, B2R, R2R, R2E, R2B>
{
    type Item = ReplayAction<R2R, R2E, R2B>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<
    BrokerID: Id,
    ExchangeID: Id,
    E2R: ExchangeToReplay,
    B2R: BrokerToReplay,
    R2R: ReplayToItself,
    R2E: ReplayToExchange<ExchangeID=ExchangeID>,
    R2B: ReplayToBroker<BrokerID=BrokerID>
>
Replay for VoidReplay<BrokerID, ExchangeID, E2R, B2R, R2R, R2E, R2B>
{
    type ExchangeID = ExchangeID;
    type BrokerID = BrokerID;

    type E2R = E2R;
    type B2R = B2R;
    type R2R = R2R;
    type R2E = R2E;
    type R2B = R2B;

    fn wakeup<KerMsg: Ord>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl Fn(Self::Item) -> KerMsg,
        _: Self::R2R,
        _: &mut impl Rng,
    ) {
        unreachable!("{} :: Replay wakeups are not planned", self.current_dt)
    }

    fn handle_exchange_reply<KerMsg: Ord>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl Fn(Self::Item) -> KerMsg,
        _: Self::E2R,
        _: Self::ExchangeID,
        _: &mut impl Rng,
    ) {}

    fn handle_broker_reply<KerMsg: Ord>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl Fn(Self::Item) -> KerMsg,
        _: Self::B2R,
        _: Self::BrokerID,
        _: &mut impl Rng)
    {}
}

pub type BasicVoidReplay<BrokerID, ExchangeID, Symbol, Settlement> = VoidReplay<
    BrokerID,
    ExchangeID,
    BasicExchangeToReplay<Symbol, Settlement>,
    Nothing,
    Nothing,
    BasicReplayToExchange<ExchangeID, Symbol, Settlement>,
    NeverType<BrokerID>
>;