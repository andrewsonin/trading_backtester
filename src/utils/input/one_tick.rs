use {
    crate::{
        order::{LimitOrderCancelRequest, LimitOrderPlacingRequest, MarketOrderPlacingRequest},
        replay::{
            ReplayAction,
            ReplayActionKind,
            request::{BasicReplayRequest, BasicReplayToExchange},
        },
        settlement::GetSettlementLag,
        traded_pair::TradedPair,
        types::{DateTime, Direction, Id, Nothing, OrderID, Price, PriceStep, Size},
    },
    csv::{Reader, ReaderBuilder, StringRecord},
    std::{
        cmp::Ordering,
        collections::{hash_map::Entry::{Occupied, Vacant}, HashMap, VecDeque},
        fs::File,
        io::{BufRead, BufReader, Write},
        path::{Path, PathBuf},
        str::FromStr,
    },
};

pub struct OneTickTradedPairReader<
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
> {
    pub exchange_id: ExchangeID,
    pub traded_pair: TradedPair<Symbol, Settlement>,

    trd_reader: OneTickHistoryReader,
    prl_reader: OneTickHistoryReader,

    next_trd: Option<HistoryEntry>,
    next_prl: Option<HistoryEntry>,

    active_limit_orders: HashMap<OrderID, (OrderID, Size)>,
    pub limit_submitted_to_internal: HashMap<OrderID, OrderID>,

    pub err_log_file: Option<File>,
}

pub(crate) struct OneTickHistoryReader
{
    files_to_parse: VecDeque<PathBuf>,
    buffered_entries: VecDeque<HistoryEntry>,
    args: OneTickTrdPrlConfig,
}

#[derive(Copy, Clone)]
pub(crate) struct HistoryEntry {
    pub datetime: DateTime,
    pub size: Size,
    pub direction: Direction,
    pub price: Price,
    pub order_id: OrderID,
}

#[derive(Clone)]
pub struct OneTickTrdPrlConfig {
    pub datetime_colname: String,
    pub order_id_colname: String,
    pub price_colname: String,
    pub size_colname: String,
    pub buy_sell_flag_colname: String,
    pub datetime_format: String,
    pub csv_sep: char,
    pub price_step: f64,
}

pub(crate) struct OneTickHistoryEntryColumnIndexer {
    pub price_idx: usize,
    pub size_idx: usize,
    pub datetime_idx: usize,
    pub buy_sell_flag_idx: usize,
    pub order_id_idx: usize,
}

impl<ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
OneTickTradedPairReader<ExchangeID, Symbol, Settlement>
{
    pub fn new(
        exchange_id: ExchangeID,
        traded_pair: TradedPair<Symbol, Settlement>,
        prl_files: PathBuf,
        prl_args: OneTickTrdPrlConfig,
        trd_files: PathBuf,
        trd_args: OneTickTrdPrlConfig,
        err_log_file: Option<PathBuf>) -> Self
    {
        let mut prl_reader = OneTickHistoryReader::new(prl_files, prl_args);
        let mut trd_reader = OneTickHistoryReader::new(trd_files, trd_args);
        Self {
            exchange_id,
            next_prl: prl_reader.next(),
            next_trd: trd_reader.next(),
            trd_reader,
            prl_reader,
            active_limit_orders: Default::default(),
            traded_pair,
            err_log_file: if let Some(err_log_file) = err_log_file {
                let file = File::create(&err_log_file).unwrap_or_else(
                    |err| panic!("Cannot create file {err_log_file:?}. Error: {err}")
                );
                Some(file)
            } else {
                None
            },
            limit_submitted_to_internal: Default::default(),
        }
    }

    pub fn clear(&mut self) {
        self.active_limit_orders.clear();
        self.limit_submitted_to_internal.clear()
    }

    pub fn next(&mut self, next_order_id: &mut OrderID) -> Option<
        ReplayAction<Nothing, BasicReplayToExchange<ExchangeID, Symbol, Settlement>>
    > {
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

    fn create_replay_to_exchange(&self,
                                 datetime: DateTime,
                                 content: BasicReplayRequest<Symbol, Settlement>) -> ReplayAction<
        Nothing,
        BasicReplayToExchange<
            ExchangeID, Symbol, Settlement
        >
    > {
        ReplayAction {
            datetime,
            content: ReplayActionKind::ReplayToExchange(
                BasicReplayToExchange {
                    exchange_id: self.exchange_id,
                    content,
                }
            ),
        }
    }

    fn process_prl(
        &mut self,
        prl: HistoryEntry,
        next_order_id: &mut OrderID) -> Option<
        ReplayAction<Nothing, BasicReplayToExchange<ExchangeID, Symbol, Settlement>>
    > {
        let entry = self.active_limit_orders.entry(prl.order_id);
        if prl.size != Size(0) {
            if let Vacant(entry) = entry {
                let order_id = *next_order_id;
                *next_order_id += OrderID(1);
                entry.insert((order_id, prl.size));
                self.limit_submitted_to_internal.insert(order_id, prl.order_id);
                let replay_action = self.create_replay_to_exchange(
                    prl.datetime,
                    BasicReplayRequest::PlaceLimitOrder(
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
                let replay_action = self.create_replay_to_exchange(
                    prl.datetime,
                    BasicReplayRequest::CancelLimitOrder(
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
            ).unwrap_or_else(|err| panic!("Cannot write to file {err_log_file:?}. Error: {err}"))
        }
        None
    }

    fn process_trd(
        &mut self,
        mut trd: HistoryEntry,
        next_order_id: &mut OrderID) -> Option<
        ReplayAction<Nothing, BasicReplayToExchange<ExchangeID, Symbol, Settlement>>
    > {
        if let Some((_, size)) = self.active_limit_orders.get_mut(&trd.order_id) {
            if *size >= trd.size {
                *size -= trd.size
            } else {
                if let Some(err_log_file) = &mut self.err_log_file {
                    writeln!(
                        err_log_file,
                        "{} :: Remaining size ({size}) of the limit order with ID {} is less then \
                        the size ({}) of the matched market order with the same reference order ID",
                        trd.datetime,
                        trd.order_id,
                        trd.size
                    ).unwrap_or_else(
                        |err| panic!("Cannot write to file {err_log_file:?}. Error: {err}")
                    )
                }
                trd.size = *size;
                *size = Size(0)
            }
            let result = if trd.size != Size(0) {
                let order_id = *next_order_id;
                *next_order_id += OrderID(1);
                let replay_action = self.create_replay_to_exchange(
                    trd.datetime,
                    BasicReplayRequest::PlaceMarketOrder(
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
            ).unwrap_or_else(|err| panic!("Cannot write to file {err_log_file:?}. Error: {err}"))
        }
        None
    }
}

impl Iterator for OneTickHistoryReader {
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

impl OneTickHistoryReader
{
    fn new(files_to_parse: impl AsRef<Path>, args: OneTickTrdPrlConfig) -> Self
    {
        let files_to_parse = files_to_parse.as_ref();
        let files = {
            let files_to_parse = Path::new(files_to_parse);
            let file = File::open(files_to_parse).unwrap_or_else(
                |err| panic!("Cannot read the following file: {files_to_parse:?}. Error: {err}")
            );
            let files_to_parse_dir = files_to_parse.parent().unwrap_or_else(
                || panic!("Cannot get parent directory of the {files_to_parse:?}")
            );
            BufReader::new(&file)
                .lines()
                .filter_map(
                    |path| {
                        let path = path.ok()?;
                        let path = Path::new(&path);
                        let result = if path.is_relative() {
                            files_to_parse_dir.join(path)
                        } else {
                            PathBuf::from(path)
                        };
                        Some(result)
                    }
                )
                .collect()
        };
        let mut res = Self::new_for_vecdeque(files, args);
        if !res.buffer_next_file() {
            panic!("No history files provided in {files_to_parse:?}")
        }
        res
    }

    fn new_for_vecdeque(files_to_parse: VecDeque<PathBuf>, args: OneTickTrdPrlConfig) -> Self {
        Self {
            files_to_parse,
            buffered_entries: Default::default(),
            args,
        }
    }

    fn buffer_next_file(&mut self) -> bool
    {
        let file_to_read = if let Some(file_to_read) = self.files_to_parse.pop_front() {
            file_to_read
        } else {
            return false;
        };
        let mut cur_file_reader = ReaderBuilder::new()
            .delimiter(self.args.csv_sep as u8)
            .from_path(&file_to_read)
            .unwrap_or_else(
                |err| panic!("Cannot read the following file: {file_to_read:?}. Error: {err}")
            );
        let col_idx_info = OneTickHistoryEntryColumnIndexer::new(
            &mut cur_file_reader,
            &file_to_read,
            &self.args,
        );

        let price_step = PriceStep(self.args.price_step);
        let datetime_format = &self.args.datetime_format;

        let process_next_entry = |(record, row_n): (Result<StringRecord, csv::Error>, _)| {
            let record = record.unwrap_or_else(
                |err| panic!(
                    "Cannot parse {row_n}-th CSV-record for the file: {file_to_read:?}. \
                    Error: {err}"
                )
            );
            let datetime = &record[col_idx_info.datetime_idx];
            let order_id = &record[col_idx_info.order_id_idx];
            let price = &record[col_idx_info.price_idx];
            let size = &record[col_idx_info.size_idx];
            let bs_flag = &record[col_idx_info.buy_sell_flag_idx];

            HistoryEntry {
                datetime: DateTime::parse_from_str(datetime, datetime_format).unwrap_or_else(
                    |err| panic!(
                        "Cannot parse to NaiveDateTime: {datetime}. \
                        Datetime format used: {datetime_format}. Error: {err}"
                    )
                ),
                size: Size::from_str(size).unwrap_or_else(
                    |err| panic!("Cannot parse to Size (i64): {size}. Error: {err}")
                ),
                direction: match bs_flag {
                    "0" | "B" | "b" | "False" | "false" => Direction::Buy,
                    "1" | "S" | "s" | "True" | "true" => Direction::Sell,
                    _ => panic!("Cannot parse buy-sell flag: {bs_flag}")
                },
                price: Price::from_decimal_str(price, price_step),
                order_id: OrderID::from_str(order_id).unwrap_or_else(
                    |err| panic!("Cannot parse to OrderID (u64): {order_id}. Error: {err}")
                ),
            }
        };
        self.buffered_entries.extend(
            cur_file_reader.records().zip(2..).map(process_next_entry)
        );
        true
    }
}

impl OneTickHistoryEntryColumnIndexer
{
    pub fn new(csv_reader: &mut Reader<File>,
               path_for_debug: impl AsRef<Path>,
               args: &OneTickTrdPrlConfig) -> Self
    {
        let path_for_debug = path_for_debug.as_ref();

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
            .unwrap_or_else(
                |err| panic!(
                    "Cannot parse header of the CSV-file: {path_for_debug:?}. Error: {err}"
                )
            )
            .into_iter()
            .enumerate()
        {
            if header == order_id_colname {
                if order_id_idx.is_none() {
                    order_id_idx = Some(i)
                } else {
                    panic!("Duplicate column {order_id_colname} in the file: {path_for_debug:?}")
                }
            } else if header == datetime_colname {
                if datetime_idx.is_none() {
                    datetime_idx = Some(i)
                } else {
                    panic!("Duplicate column {datetime_colname} in the file: {path_for_debug:?}")
                }
            } else if header == size_colname {
                if size_idx.is_none() {
                    size_idx = Some(i)
                } else {
                    panic!("Duplicate column {size_colname} in the file: {path_for_debug:?}")
                }
            } else if header == price_colname {
                if price_idx.is_none() {
                    price_idx = Some(i)
                } else {
                    panic!("Duplicate column {price_colname} in the file: {path_for_debug:?}")
                }
            } else if header == bs_flag_colname {
                if buy_sell_flag_idx.is_none() {
                    buy_sell_flag_idx = Some(i)
                } else {
                    panic!("Duplicate column {bs_flag_colname} in the file: {path_for_debug:?}")
                }
            }
        };
        let price_idx = price_idx.unwrap_or_else(
            || panic!("Cannot find {price_colname} column in the CSV-file: {path_for_debug:?}")
        );
        let size_idx = size_idx.unwrap_or_else(
            || panic!("Cannot find {size_colname} column in the CSV-file: {path_for_debug:?}")
        );
        let datetime_idx = datetime_idx.unwrap_or_else(
            || panic!("Cannot find {datetime_colname} column in the CSV-file: {path_for_debug:?}")
        );
        let buy_sell_flag_idx = buy_sell_flag_idx.unwrap_or_else(
            || panic!("Cannot find {bs_flag_colname} column in the CSV-file: {path_for_debug:?}")
        );
        let order_id_idx = order_id_idx.unwrap_or_else(
            || panic!("Cannot find {order_id_colname} column in the CSV-file: {path_for_debug:?}")
        );
        Self {
            price_idx,
            size_idx,
            datetime_idx,
            buy_sell_flag_idx,
            order_id_idx,
        }
    }
}
