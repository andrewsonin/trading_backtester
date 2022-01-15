use {
    crate::{
        broker::{
            Broker,
            concrete::BasicBroker,
        },
        exchange::{
            concrete::BasicExchange,
            Exchange,
        },
        replay::{
            concrete::{ExchangeSession, GetNextObSnapshotDelay, OneTickReplay, TradedPairLifetime},
            Replay,
        },
        traded_pair::TradedPair,
        trader::{concrete::SpreadWriter, Trader},
        types::{DateTime, Identifier, PriceStep},
        utils::input::one_tick::{OneTickTradedPairReader, TrdPrlConfig},
    },
    std::path::{Path, PathBuf},
};

pub trait BuildExchange<
    ExchangeID: Identifier,
    BrokerID: Identifier,
    Symbol: Identifier
> {
    type E: Exchange<ExchangeID, BrokerID, Symbol>;

    fn build(&self) -> Self::E;
}

pub trait BuildReplay<
    ExchangeID: Identifier,
    Symbol: Identifier
> {
    type R: Replay<ExchangeID, Symbol>;

    fn build(&self) -> Self::R;
}

pub trait BuildBroker<
    BrokerID: Identifier,
    TraderID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier
> {
    type B: Broker<BrokerID, TraderID, ExchangeID, Symbol>;

    fn build(&self) -> Self::B;
}

pub trait BuildTrader<
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier
> {
    type T: Trader<TraderID, BrokerID, ExchangeID, Symbol>;

    fn build(&self) -> Self::T;
}

#[derive(Clone)]
pub struct OneTickTradedPairReaderConfig<ExchangeID: Identifier, Symbol: Identifier>
{
    pub exchange_id: ExchangeID,
    pub traded_pair: TradedPair<Symbol>,
    pub prl_files: PathBuf,
    pub prl_args: TrdPrlConfig,
    pub trd_files: PathBuf,
    pub trd_args: TrdPrlConfig,
    pub err_log_file: Option<PathBuf>,
}

impl<ExchangeID: Identifier, Symbol: Identifier>
From<&OneTickTradedPairReaderConfig<ExchangeID, Symbol>>
for OneTickTradedPairReader<ExchangeID, Symbol>
{
    fn from(config: &OneTickTradedPairReaderConfig<ExchangeID, Symbol>) -> Self {
        OneTickTradedPairReader::new(
            config.exchange_id,
            config.traded_pair,
            config.prl_files.clone(),
            config.prl_args.clone(),
            config.trd_files.clone(),
            config.trd_args.clone(),
            config.err_log_file.clone(),
        )
    }
}

#[derive(Clone)]
pub struct OneTickReplayConfig<
    ExchangeID: Identifier,
    Symbol: Identifier,
    ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol>
> {
    pub start_dt: DateTime,
    pub traded_pair_configs: Vec<OneTickTradedPairReaderConfig<ExchangeID, Symbol>>,
    pub exchange_open_close_events: Vec<ExchangeSession<ExchangeID>>,
    pub traded_pair_creation_events: Vec<TradedPairLifetime<ExchangeID, Symbol>>,
    pub ob_snapshot_delay_scheduler: ObSnapshotDelay,
}

impl<
    ExchangeID: Identifier,
    Symbol: Identifier,
    ObSnapshotDelay: Clone + GetNextObSnapshotDelay<ExchangeID, Symbol>
>
BuildReplay<ExchangeID, Symbol>
for OneTickReplayConfig<ExchangeID, Symbol, ObSnapshotDelay>
{
    type R = OneTickReplay<ExchangeID, Symbol, ObSnapshotDelay>;

    fn build(&self) -> Self::R {
        Self::R::new(
            self.start_dt,
            self.traded_pair_configs.iter().map(From::from),
            self.exchange_open_close_events.iter().cloned(),
            self.traded_pair_creation_events.iter().cloned(),
            self.ob_snapshot_delay_scheduler.clone(),
        )
    }
}

pub trait InitBasicExchange {}

impl<ExchangeID: Identifier + InitBasicExchange, BrokerID: Identifier, Symbol: Identifier>
BuildExchange<ExchangeID, BrokerID, Symbol>
for ExchangeID
{
    type E = BasicExchange<ExchangeID, BrokerID, Symbol>;

    fn build(&self) -> Self::E {
        Self::E::new(*self)
    }
}

pub trait InitBasicBroker {}

impl<
    BrokerID: Identifier + InitBasicBroker,
    TraderID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier
>
BuildBroker<BrokerID, TraderID, ExchangeID, Symbol>
for BrokerID
{
    type B = BasicBroker<BrokerID, TraderID, ExchangeID, Symbol>;

    fn build(&self) -> Self::B {
        Self::B::new(*self)
    }
}

#[derive(Clone, Copy)]
pub struct SpreadWriterConfig<TraderID: Identifier, PS: Into<PriceStep> + Copy, F: AsRef<Path>> {
    pub name: TraderID,
    pub file: F,
    pub price_step: PS,
}

impl<
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    PS: Into<PriceStep> + Copy,
    F: AsRef<Path>
>
BuildTrader<TraderID, BrokerID, ExchangeID, Symbol>
for SpreadWriterConfig<TraderID, PS, F>
{
    type T = SpreadWriter<TraderID>;

    fn build(&self) -> Self::T {
        Self::T::new(self.name, self.price_step, &self.file)
    }
}