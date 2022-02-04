use {
    crate::{
        concrete::{
            broker::BasicBroker,
            exchange::BasicExchange,
            input::one_tick::{OneTickTradedPairReader, OneTickTrdPrlConfig},
            replay::{
                ExchangeSession,
                GetNextObSnapshotDelay,
                OneTickReplay,
                settlement::GetSettlementLag,
                TradedPairLifetime,
            },
            traded_pair::TradedPair,
            trader::SpreadWriter,
            types::PriceStep,
        },
        types::{DateTime, Id},
    },
    std::path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct OneTickTradedPairReaderConfig<
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
> {
    pub exchange_id: ExchangeID,
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub prl_files: PathBuf,
    pub prl_args: OneTickTrdPrlConfig,
    pub trd_files: PathBuf,
    pub trd_args: OneTickTrdPrlConfig,
    pub err_log_file: Option<PathBuf>,
}

impl<ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
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
    ExchangeID: Id,
    Symbol: Id,
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
    BrokerID: Id,
    ExchangeID: Id,
    Symbol: Id,
    ObSnapshotDelay: Clone + GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>,
    Settlement: GetSettlementLag
>
From<&OneTickReplayConfig<ExchangeID, Symbol, ObSnapshotDelay, Settlement>>
for OneTickReplay<BrokerID, ExchangeID, Symbol, ObSnapshotDelay, Settlement>
{
    fn from(cfg: &OneTickReplayConfig<ExchangeID, Symbol, ObSnapshotDelay, Settlement>) -> Self {
        Self::new(
            cfg.start_dt,
            cfg.traded_pair_configs.iter().map(From::from),
            cfg.exchange_open_close_events.iter().cloned(),
            cfg.traded_pair_creation_events.iter().cloned(),
            cfg.ob_snapshot_delay_scheduler.clone(),
        )
    }
}

pub trait InitBasicExchange {}

impl<
    ExchangeID: Id + InitBasicExchange,
    BrokerID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
>
From<&ExchangeID>
for BasicExchange<ExchangeID, BrokerID, Symbol, Settlement>
{
    fn from(exchange_id: &ExchangeID) -> Self {
        Self::new(*exchange_id)
    }
}

pub trait InitBasicBroker {}

impl<
    BrokerID: Id + InitBasicBroker,
    TraderID: Id,
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
>
From<&BrokerID>
for BasicBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>
{
    fn from(broker_id: &BrokerID) -> Self {
        Self::new(*broker_id)
    }
}

#[derive(Clone, Copy)]
pub struct SpreadWriterConfig<TraderID: Id, PS: Into<PriceStep> + Copy, F: AsRef<Path>> {
    pub name: TraderID,
    pub file: F,
    pub price_step: PS,
}

impl<TraderID: Id, PS: Into<PriceStep> + Copy, F: AsRef<Path>>
SpreadWriterConfig<TraderID, PS, F>
{
    pub fn new(name: TraderID, file: F, price_step: PS) -> Self {
        Self { name, file, price_step }
    }
}

impl<
    TraderID: Id,
    BrokerID: Id,
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag,
    PS: Into<PriceStep> + Copy,
    F: AsRef<Path>
>
From<&SpreadWriterConfig<TraderID, PS, F>>
for SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
{
    fn from(cfg: &SpreadWriterConfig<TraderID, PS, F>) -> Self {
        Self::new(cfg.name, cfg.price_step, &cfg.file)
    }
}