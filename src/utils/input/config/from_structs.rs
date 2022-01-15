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
        settlement::GetSettlementLag,
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
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    type E: Exchange<ExchangeID, BrokerID, Symbol, Settlement>;

    fn build(&self) -> Self::E;
}

pub trait BuildReplay<
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    type R: Replay<ExchangeID, Symbol, Settlement>;

    fn build(&self) -> Self::R;
}

pub trait BuildBroker<
    BrokerID: Identifier,
    TraderID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    type B: Broker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>;

    fn build(&self) -> Self::B;
}

pub trait BuildTrader<
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    type T: Trader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>;

    fn build(&self) -> Self::T;
}

#[derive(Clone)]
pub struct OneTickTradedPairReaderConfig<
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    pub exchange_id: ExchangeID,
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub prl_files: PathBuf,
    pub prl_args: TrdPrlConfig,
    pub trd_files: PathBuf,
    pub trd_args: TrdPrlConfig,
    pub err_log_file: Option<PathBuf>,
}

impl<ExchangeID: Identifier, Symbol: Identifier, Settlement: GetSettlementLag>
From<&OneTickTradedPairReaderConfig<ExchangeID, Symbol, Settlement>>
for OneTickTradedPairReader<ExchangeID, Symbol, Settlement>
{
    fn from(config: &OneTickTradedPairReaderConfig<ExchangeID, Symbol, Settlement>) -> Self {
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
    ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>,
    Settlement: GetSettlementLag
> {
    pub start_dt: DateTime,
    pub traded_pair_configs: Vec<OneTickTradedPairReaderConfig<ExchangeID, Symbol, Settlement>>,
    pub exchange_open_close_events: Vec<ExchangeSession<ExchangeID>>,
    pub traded_pair_creation_events: Vec<TradedPairLifetime<ExchangeID, Symbol, Settlement>>,
    pub ob_snapshot_delay_scheduler: ObSnapshotDelay,
}

impl<
    ExchangeID: Identifier,
    Symbol: Identifier,
    ObSnapshotDelay: Clone + GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>,
    Settlement: GetSettlementLag
>
BuildReplay<ExchangeID, Symbol, Settlement>
for OneTickReplayConfig<ExchangeID, Symbol, ObSnapshotDelay, Settlement>
{
    type R = OneTickReplay<ExchangeID, Symbol, ObSnapshotDelay, Settlement>;

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

impl<
    ExchangeID: Identifier + InitBasicExchange,
    BrokerID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
>
BuildExchange<ExchangeID, BrokerID, Symbol, Settlement>
for ExchangeID
{
    type E = BasicExchange<ExchangeID, BrokerID, Symbol, Settlement>;

    fn build(&self) -> Self::E {
        Self::E::new(*self)
    }
}

pub trait InitBasicBroker {}

impl<
    BrokerID: Identifier + InitBasicBroker,
    TraderID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
>
BuildBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>
for BrokerID
{
    type B = BasicBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>;

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

impl<TraderID: Identifier, PS: Into<PriceStep> + Copy, F: AsRef<Path>>
SpreadWriterConfig<TraderID, PS, F>
{
    pub fn new(name: TraderID, file: F, price_step: PS) -> Self {
        Self {
            name,
            file,
            price_step,
        }
    }
}

impl<
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag,
    PS: Into<PriceStep> + Copy,
    F: AsRef<Path>
>
BuildTrader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
for SpreadWriterConfig<TraderID, PS, F>
{
    type T = SpreadWriter<TraderID>;

    fn build(&self) -> Self::T {
        Self::T::new(self.name, self.price_step, &self.file)
    }
}