use {
    crate::{
        exchange::reply::{
            ExchangeEventNotification,
            ExchangeToReplay,
            ExchangeToReplayReply,
            ObSnapshot,
        },
        order::{LimitOrderCancelRequest, LimitOrderPlacingRequest, MarketOrderPlacingRequest},
        replay::{Replay, ReplayAction, request::{ReplayRequest, ReplayToExchange}},
        traded_pair::TradedPair,
        types::{
            DateTime,
            Direction,
            Duration,
            Identifier,
            OrderID, Price,
            PriceStep,
            ReaderBuilder,
            Size,
            StdRng,
            TimeSync,
        },
        utils::{
            ExpectWith,
            queue::LessElementBinaryHeap,
        },
    },
    csv::{Reader, StringRecord},
    std::{
        cmp::{Ordering, Reverse},
        collections::{hash_map::Entry::{Occupied, Vacant}, HashMap, HashSet, VecDeque},
        fs::File,
        io::{BufRead, BufReader, Write},
        num::NonZeroU64,
        str::FromStr,
    },
};

pub trait GetNextObSnapshotDelay<ExchangeID: Identifier, Symbol: Identifier>
{
    fn get_ob_snapshot_delay(
        &mut self,
        exchange_id: ExchangeID,
        traded_pair: TradedPair<Symbol>,
        rng: &mut StdRng,
        current_dt: DateTime) -> Option<NonZeroU64>;
}

pub struct OneTickReplay<
    ExchangeID: Identifier,
    Symbol: Identifier,
    ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol>,
    ObSnapshotInspector: FnMut(DateTime, ExchangeID, &ObSnapshot<Symbol>) -> ()
> {
    current_dt: DateTime,
    traded_pair_readers: Vec<OneTickTradedPairReader<ExchangeID, Symbol>>,
    action_queue: LessElementBinaryHeap<(ReplayAction<ExchangeID, Symbol>, i64)>,

    ob_snapshot_delay_scheduler: ObSnapshotDelay,
    active_traded_pairs: HashSet<(ExchangeID, TradedPair<Symbol>)>,

    next_order_id: OrderID,

    inspect_ob_snapshot: ObSnapshotInspector,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct HistoryEntry {
    pub datetime: DateTime,
    pub size: Size,
    pub direction: Direction,
    pub price: Price,
    pub order_id: OrderID,
}

#[derive(Debug)]
pub(crate) struct ExchangeSession<ExchangeID: Identifier> {
    pub exchange_id: ExchangeID,
    pub open_dt: DateTime,
    pub close_dt: DateTime,
}

#[derive(Debug)]
pub(crate) struct TradedPairLifetime<ExchangeID: Identifier, Symbol: Identifier> {
    pub exchange_id: ExchangeID,
    pub traded_pair: TradedPair<Symbol>,
    pub price_step: PriceStep,
    pub start_dt: DateTime,
    pub stop_dt: Option<DateTime>,
}

pub(crate) struct TrdPrlInfo {
    pub datetime_colname: String,
    pub order_id_colname: String,
    pub price_colname: String,
    pub size_colname: String,
    pub buy_sell_flag_colname: String,
    pub datetime_format: String,
    pub csv_sep: char,
    pub price_step: f64,
}

impl<
    ExchangeID: Identifier,
    Symbol: Identifier,
    ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol>,
    ObSnapshotInspector: FnMut(DateTime, ExchangeID, &ObSnapshot<Symbol>) -> ()
>
OneTickReplay<ExchangeID, Symbol, ObSnapshotDelay, ObSnapshotInspector>
{
    pub(crate) fn new(
        ob_snapshot_delay_scheduler: ObSnapshotDelay,
        start_dt: DateTime,
        traded_pair_readers: impl IntoIterator<Item=OneTickTradedPairReader<ExchangeID, Symbol>>,
        exchange_open_close_events: impl IntoIterator<Item=ExchangeSession<ExchangeID>>,
        traded_pair_creation_events: impl IntoIterator<Item=TradedPairLifetime<ExchangeID, Symbol>>,
        inspect_ob_snapshot: ObSnapshotInspector,
    ) -> Self
    {
        let mut prev_dt: HashMap<ExchangeID, DateTime> = Default::default();
        let open_close_iterator = exchange_open_close_events.into_iter().map(
            |ExchangeSession { exchange_id, open_dt, close_dt }| {
                let prev_dt = prev_dt.entry(exchange_id).or_insert_with(
                    || {
                        if open_dt < start_dt {
                            panic!(
                                "Exchange {} open_dt {} is less than start_dt {}",
                                exchange_id, open_dt, start_dt
                            )
                        };
                        start_dt
                    }
                );
                if open_dt < *prev_dt {
                    panic!(
                        "Exchange {} open/close datetime pairs \
                        are not stored in the ascending order",
                        exchange_id
                    )
                }
                if close_dt < open_dt {
                    panic!(
                        "Exchange {} close datetime {} is less than \
                        the corresponding exchange open datetime {}",
                        exchange_id, close_dt, open_dt
                    )
                }
                *prev_dt = close_dt;
                let open_event = ReplayAction {
                    datetime: open_dt,
                    content: ReplayToExchange {
                        exchange_id,
                        content: ReplayRequest::ExchangeOpen,
                    },
                };
                let close_event = ReplayAction {
                    datetime: close_dt,
                    content: ReplayToExchange {
                        exchange_id,
                        content: ReplayRequest::ExchangeClosed,
                    },
                };
                [open_event, close_event].into_iter()
            }
        );
        let traded_pair_creation_iterator = traded_pair_creation_events.into_iter().map(
            |TradedPairLifetime { exchange_id, traded_pair, price_step, start_dt, stop_dt }|
                {
                    let start_trades = ReplayAction {
                        datetime: start_dt,
                        content: ReplayToExchange {
                            exchange_id,
                            content: ReplayRequest::StartTrades(traded_pair, price_step),
                        },
                    };
                    if let Some(stop_dt) = stop_dt {
                        let stop_trades = ReplayAction {
                            datetime: stop_dt,
                            content: ReplayToExchange {
                                exchange_id,
                                content: ReplayRequest::StopTrades(traded_pair),
                            },
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
                    let first_event = pair_reader.next(&mut next_order_id).expect_with(
                        || panic!("Traded pair reader {} is empty", i)
                    );
                    (Reverse((first_event, i as i64)), pair_reader)
                }
            ).unzip();
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
            inspect_ob_snapshot,
        }
    }
}

impl<
    ExchangeID: Identifier,
    Symbol: Identifier,
    ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol>,
    ObSnapshotInspector: FnMut(DateTime, ExchangeID, &ObSnapshot<Symbol>) -> ()
>
TimeSync for OneTickReplay<ExchangeID, Symbol, ObSnapshotDelay, ObSnapshotInspector>
{
    fn current_datetime_mut(&mut self) -> &mut DateTime {
        &mut self.current_dt
    }
}

impl<
    ExchangeID: Identifier,
    Symbol: Identifier,
    ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol>,
    ObSnapshotInspector: FnMut(DateTime, ExchangeID, &ObSnapshot<Symbol>) -> ()
> Iterator for OneTickReplay<ExchangeID, Symbol, ObSnapshotDelay, ObSnapshotInspector>
{
    type Item = ReplayAction<ExchangeID, Symbol>;

    fn next(&mut self) -> Option<Self::Item>
    {
        if let Some((action, reader_idx)) = self.action_queue.pop() {
            if reader_idx != -1 {
                if let Some(next_action) = self.traded_pair_readers
                    .get_mut(reader_idx as usize)
                    .expect_with(|| unreachable!("Index is out of bounds"))
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
    ExchangeID: Identifier,
    Symbol: Identifier,
    ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol>,
    ObSnapshotInspector: FnMut(DateTime, ExchangeID, &ObSnapshot<Symbol>) -> ()
>
Replay<ExchangeID, Symbol>
for OneTickReplay<ExchangeID, Symbol, ObSnapshotDelay, ObSnapshotInspector>
{
    fn handle_exchange_reply(
        &mut self,
        reply: ExchangeToReplay<Symbol>,
        exchange_id: ExchangeID,
        rng: &mut StdRng) -> Vec<ReplayAction<ExchangeID, Symbol>>
    {
        let mut get_ob_snapshot_delay = |traded_pair| {
            if let Some(delay) = self.ob_snapshot_delay_scheduler.get_ob_snapshot_delay(
                exchange_id, traded_pair, rng, self.current_dt,
            ) {
                let action = ReplayAction {
                    datetime: self.current_dt + Duration::nanoseconds(delay.get() as i64),
                    content: ReplayToExchange {
                        exchange_id,
                        content: ReplayRequest::BroadcastObStateToBrokers(traded_pair),
                    },
                };
                Some(action)
            } else {
                None
            }
        };
        match reply.content {
            ExchangeToReplayReply::ExchangeEventNotification(notification) => {
                match notification
                {
                    ExchangeEventNotification::ExchangeOpen => {
                        return self.active_traded_pairs.iter().filter_map(
                            |(tp_exchange_id, traded_pair)| if *tp_exchange_id != exchange_id {
                                None
                            } else {
                                get_ob_snapshot_delay(*traded_pair)
                            }
                        ).collect();
                    }
                    ExchangeEventNotification::TradesStarted(traded_pair, _price_step) => {
                        if !self.active_traded_pairs.insert((exchange_id, traded_pair)) {
                            panic!(
                                "Trades for traded pair already started: {} {:?}",
                                exchange_id,
                                traded_pair
                            )
                        }
                        if let Some(action) = get_ob_snapshot_delay(traded_pair) {
                            return vec![action];
                        }
                    }
                    ExchangeEventNotification::ObSnapshot(snapshot) => {
                        (self.inspect_ob_snapshot)(self.current_dt, exchange_id, &snapshot);
                        if let Some(action) = get_ob_snapshot_delay(snapshot.traded_pair) {
                            return vec![action];
                        }
                    }
                    ExchangeEventNotification::TradesStopped(traded_pair) => {
                        if !self.active_traded_pairs.remove(&(exchange_id, traded_pair)) {
                            panic!(
                                "Trades for traded pair already stopped or not ever started: \
                                {} {:?}",
                                exchange_id,
                                traded_pair
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
            ExchangeToReplayReply::CannotCancelOrder(cannot_cancel) => {
                let reader = self.traded_pair_readers.iter_mut()
                    .filter(|reader| reader.exchange_id == exchange_id
                        && reader.traded_pair == cannot_cancel.traded_pair)
                    .next()
                    .expect_with(
                        || unreachable!(
                            "Cannot find corresponding traded pair reader for {:?}", cannot_cancel
                        )
                    );
                if let Some(err_log_file) = &mut reader.err_log_file {
                    if let Some(order_id) = reader.limit_submitted_to_internal
                        .get(&cannot_cancel.order_id)
                    {
                        writeln!(
                            err_log_file,
                            "{} :: Cannot cancel limit order with ID {} since {}",
                            self.current_dt,
                            order_id,
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
                    }.expect_with(|| panic!("Cannot write to file {:?}", err_log_file))
                }
            }
            ExchangeToReplayReply::OrderPlacementDiscarded(_) |
            ExchangeToReplayReply::CannotOpenExchange(_) |
            ExchangeToReplayReply::CannotStartTrades(_) |
            ExchangeToReplayReply::CannotCloseExchange(_) |
            ExchangeToReplayReply::CannotStopTrades(_) => {
                panic!("{} :: {:?}. Exchange {}", self.current_dt, reply, exchange_id)
            }
            _ => {}
        }
        vec![]
    }
}

pub(crate) struct OneTickTradedPairReader<
    ExchangeID: Identifier,
    Symbol: Identifier
> {
    exchange_id: ExchangeID,
    traded_pair: TradedPair<Symbol>,

    trd_reader: HistoryReader,
    prl_reader: HistoryReader,

    next_trd: Option<HistoryEntry>,
    next_prl: Option<HistoryEntry>,

    active_limit_orders: HashMap<OrderID, (OrderID, Size)>,
    limit_submitted_to_internal: HashMap<OrderID, OrderID>,

    err_log_file: Option<File>,
}

impl<ExchangeID: Identifier, Symbol: Identifier>
OneTickTradedPairReader<ExchangeID, Symbol>
{
    pub fn new(
        exchange_id: ExchangeID,
        traded_pair: TradedPair<Symbol>,
        prl_files: &str,
        prl_args: TrdPrlInfo,
        trd_files: &str,
        trd_args: TrdPrlInfo,
        err_log_file: Option<&str>) -> Self
    {
        let mut prl_reader = HistoryReader::new(prl_files, prl_args);
        let mut trd_reader = HistoryReader::new(trd_files, trd_args);
        Self {
            exchange_id,
            next_prl: prl_reader.next(),
            next_trd: trd_reader.next(),
            trd_reader,
            prl_reader,
            active_limit_orders: Default::default(),
            traded_pair,
            err_log_file: if let Some(err_log_file) = err_log_file {
                let file = File::create(err_log_file).expect_with(
                    || panic!("Cannot create file {}", err_log_file)
                );
                Some(file)
            } else {
                None
            },
            limit_submitted_to_internal: Default::default(),
        }
    }

    fn clear(&mut self) {
        self.active_limit_orders.clear();
        self.limit_submitted_to_internal.clear()
    }

    fn next(&mut self, next_order_id: &mut OrderID) -> Option<ReplayAction<ExchangeID, Symbol>>
    {
        loop {
            let res;
            match (&self.next_prl, &self.next_trd)
            {
                (Some(prl), Some(trd)) => {
                    let cmp = prl.datetime.cmp(&trd.datetime);
                    if cmp == Ordering::Less
                        || cmp == Ordering::Equal && prl.order_id < trd.order_id
                    {
                        let prl = *prl;
                        res = self.process_prl(prl, next_order_id);
                        self.next_prl = self.prl_reader.next()
                    } else {
                        let trd = *trd;
                        res = self.process_trd(trd, next_order_id);
                        self.next_trd = self.trd_reader.next()
                    }
                }
                (Some(prl), _) => {
                    let prl = *prl;
                    res = self.process_prl(prl, next_order_id);
                    self.next_prl = self.prl_reader.next()
                }
                (_, Some(trd)) => {
                    let trd = *trd;
                    res = self.process_trd(trd, next_order_id);
                    self.next_trd = self.trd_reader.next()
                }
                _ => { return None; }
            }
            if res.is_some() {
                return res;
            }
        }
    }

    fn create_replay_action(&self,
                            datetime: DateTime,
                            content: ReplayRequest<Symbol>) -> ReplayAction<ExchangeID, Symbol> {
        ReplayAction {
            datetime,
            content: ReplayToExchange {
                exchange_id: self.exchange_id,
                content,
            },
        }
    }

    fn process_prl(
        &mut self,
        prl: HistoryEntry,
        next_order_id: &mut OrderID) -> Option<ReplayAction<ExchangeID, Symbol>>
    {
        let entry = self.active_limit_orders.entry(prl.order_id);
        if prl.size != Size(0) {
            if let Vacant(entry) = entry {
                let order_id = *next_order_id;
                *next_order_id += OrderID(1);
                entry.insert((order_id, prl.size));
                self.limit_submitted_to_internal.insert(order_id, prl.order_id);
                let replay_action = self.create_replay_action(
                    prl.datetime,
                    ReplayRequest::PlaceLimitOrder(
                        LimitOrderPlacingRequest {
                            traded_pair: self.traded_pair,
                            order_id,
                            direction: prl.direction,
                            price: prl.price,
                            size: prl.size,
                            dummy: false,
                        }
                    ),
                );
                return Some(replay_action);
            }
        } else if let Occupied(entry) = entry {
            let (order_id, size) = entry.get();
            let (order_id, size) = (*order_id, *size);
            if size != Size(0) {
                let replay_action = self.create_replay_action(
                    prl.datetime,
                    ReplayRequest::CancelLimitOrder(
                        LimitOrderCancelRequest {
                            traded_pair: self.traded_pair,
                            order_id,
                        }
                    ),
                );
                return Some(replay_action);
            }
        } else if let Some(err_log_file) = &mut self.err_log_file {
            writeln!(
                err_log_file,
                "{} :: Cannot cancel limit order with ID {} since it has not been submitted",
                prl.datetime,
                prl.order_id
            ).expect_with(|| panic!("Cannot write to file {:?}", err_log_file))
        }
        None
    }

    fn process_trd(
        &mut self,
        mut trd: HistoryEntry,
        next_order_id: &mut OrderID) -> Option<ReplayAction<ExchangeID, Symbol>>
    {
        if let Some((_, size)) = self.active_limit_orders.get_mut(&trd.order_id) {
            if *size >= trd.size {
                *size -= trd.size
            } else {
                if let Some(err_log_file) = &mut self.err_log_file {
                    writeln!(
                        err_log_file,
                        "{} :: Remaining size ({}) of the limit order with ID {} is less then \
                        the size ({}) of the matched market order with the same reference order ID",
                        trd.datetime,
                        size,
                        trd.order_id,
                        trd.size
                    ).expect_with(|| panic!("Cannot write to file {:?}", err_log_file))
                }
                trd.size = *size;
                *size = Size(0)
            }
            let result = if trd.size != Size(0) {
                let order_id = *next_order_id;
                *next_order_id += OrderID(1);
                let replay_action = self.create_replay_action(
                    trd.datetime,
                    ReplayRequest::PlaceMarketOrder(
                        MarketOrderPlacingRequest {
                            traded_pair: self.traded_pair,
                            order_id,
                            direction: trd.direction,
                            size: trd.size,
                            dummy: false,
                        }
                    ),
                );
                Some(replay_action)
            } else {
                None
            };
            return result;
        }
        if let Some(err_log_file) = &mut self.err_log_file {
            writeln!(
                err_log_file,
                "{} :: Cannot match marker order with reference order ID {} and size ({}) \
                since corresponding limit order has not been submitted",
                trd.datetime,
                trd.order_id,
                trd.size
            ).expect_with(|| panic!("Cannot write to file {:?}", err_log_file))
        }
        None
    }
}

struct HistoryReader
{
    files_to_parse: VecDeque<String>,
    pub buffered_entries: VecDeque<HistoryEntry>,
    args: TrdPrlInfo,
}

impl Iterator for HistoryReader {
    type Item = HistoryEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let next_entry = self.buffered_entries.pop_front();
        if next_entry.is_some() {
            return next_entry;
        };
        self.buffer_next_file();
        self.buffered_entries.pop_front()
    }
}

impl HistoryReader
{
    fn new(files_to_parse: &str, args: TrdPrlInfo) -> Self
    {
        let files = {
            let file = File::open(files_to_parse).expect_with(
                || panic!("Cannot read the following file: {}", files_to_parse)
            );
            BufReader::new(&file).lines().filter_map(Result::ok).collect()
        };
        let mut res = Self::new_for_vecdeque(files, args);
        if !res.buffer_next_file() {
            panic!("No history files provided in {}", files_to_parse)
        }
        res
    }

    fn new_for_vecdeque(files_to_parse: VecDeque<String>, args: TrdPrlInfo) -> Self {
        Self {
            files_to_parse,
            buffered_entries: Default::default(),
            args,
        }
    }

    fn buffer_next_file(&mut self) -> bool
    {
        let file_to_read = match self.files_to_parse.pop_front() {
            Some(file_to_read) => { file_to_read }
            _ => { return false; }
        };
        let mut cur_file_reader = ReaderBuilder::new()
            .delimiter(self.args.csv_sep as u8)
            .from_path(&file_to_read)
            .expect_with(|| panic!("Cannot read the following file: {}", file_to_read));
        let col_idx_info = HistoryEntryColumnIndexer::new(
            &mut cur_file_reader,
            &file_to_read,
            &self.args,
        );

        let price_step = PriceStep(self.args.price_step);
        let datetime_format = &self.args.datetime_format;

        let process_next_entry = |(record, row_n): (Result<StringRecord, csv::Error>, _)| {
            let record = record.expect_with(
                || panic!("Cannot parse {}-th CSV-record for the file: {}",
                          row_n,
                          file_to_read)
            );
            let datetime = &record[col_idx_info.datetime_idx];
            let order_id = &record[col_idx_info.order_id_idx];
            let price = &record[col_idx_info.price_idx];
            let size = &record[col_idx_info.size_idx];
            let bs_flag = &record[col_idx_info.buy_sell_flag_idx];

            HistoryEntry {
                datetime: DateTime::parse_from_str(datetime, datetime_format).expect_with(
                    || panic!(
                        "Cannot parse to NaiveDateTime: {}. Datetime format used: {}",
                        datetime,
                        datetime_format
                    )
                ),
                size: Size::from_str(size).expect_with(
                    || panic!("Cannot parse to Size (i64): {}", size)
                ),
                direction: match bs_flag {
                    "0" | "B" | "b" | "False" | "false" => Direction::Buy,
                    "1" | "S" | "s" | "True" | "true" => Direction::Sell,
                    _ => panic!("Cannot parse buy-sell flag: {}", bs_flag)
                },
                price: Price::from_decimal_str(price, price_step),
                order_id: OrderID::from_str(order_id).expect_with(
                    || panic!("Cannot parse to OrderID (u64): {}", order_id)
                ),
            }
        };
        self.buffered_entries.extend(
            cur_file_reader.records().zip(2..).map(process_next_entry)
        );
        true
    }
}

pub(crate) struct HistoryEntryColumnIndexer {
    pub price_idx: usize,
    pub size_idx: usize,
    pub datetime_idx: usize,
    pub buy_sell_flag_idx: usize,
    pub order_id_idx: usize,
}

impl HistoryEntryColumnIndexer
{
    pub fn new(csv_reader: &mut Reader<File>,
               path_for_debug: &str,
               args: &TrdPrlInfo) -> Self
    {
        let mut order_id_idx = None;
        let mut datetime_idx = None;
        let mut size_idx = None;
        let mut price_idx = None;
        let mut buy_sell_flag_idx = None;

        let order_id_colname = &args.order_id_colname;
        let datetime_colname = &args.datetime_colname;
        let size_colname = &args.size_colname;
        let price_colname = &args.price_colname;
        let bs_flag_colname = &args.buy_sell_flag_colname;

        for (i, header) in csv_reader
            .headers()
            .expect_with(|| panic!("Cannot parse header of the CSV-file: {}", path_for_debug))
            .into_iter()
            .enumerate()
        {
            if header == order_id_colname {
                if order_id_idx.is_none() {
                    order_id_idx = Some(i)
                } else {
                    panic!("Duplicate column {} in the file: {}", order_id_colname, path_for_debug)
                }
            } else if header == datetime_colname {
                if datetime_idx.is_none() {
                    datetime_idx = Some(i)
                } else {
                    panic!("Duplicate column {} in the file: {}", datetime_colname, path_for_debug)
                }
            } else if header == size_colname {
                if size_idx.is_none() {
                    size_idx = Some(i)
                } else {
                    panic!("Duplicate column {} in the file: {}", size_colname, path_for_debug)
                }
            } else if header == price_colname {
                if price_idx.is_none() {
                    price_idx = Some(i)
                } else {
                    panic!("Duplicate column {} in the file: {}", price_colname, path_for_debug)
                }
            } else if header == bs_flag_colname {
                if buy_sell_flag_idx.is_none() {
                    buy_sell_flag_idx = Some(i)
                } else {
                    panic!("Duplicate column {} in the file: {}", bs_flag_colname, path_for_debug)
                }
            }
        };
        Self {
            price_idx: price_idx.expect_with(
                || panic!("Cannot find {} column in the CSV-file: {}", price_colname, path_for_debug)
            ),
            size_idx: size_idx.expect_with(
                || panic!("Cannot find {} column in the CSV-file: {}", size_colname, path_for_debug)
            ),
            datetime_idx: datetime_idx.expect_with(
                || panic!("Cannot find {} column in the CSV-file: {}", datetime_colname, path_for_debug)
            ),
            buy_sell_flag_idx: buy_sell_flag_idx.expect_with(
                || panic!("Cannot find {} column in the CSV-file: {}", bs_flag_colname, path_for_debug)
            ),
            order_id_idx: order_id_idx.expect_with(
                || panic!("Cannot find {} column in the CSV-file: {}", order_id_colname, path_for_debug)
            ),
        }
    }
}