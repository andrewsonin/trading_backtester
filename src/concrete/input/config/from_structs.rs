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
                TradedPairLifetime,
            },
            traded_pair::{settlement::GetSettlementLag, TradedPair},
            trader::SpreadWriter,
            types::PriceStep,
        },
        types::{DateTime, Id},
    },
    std::path::{Path, PathBuf},
};

#[derive(Clone)]
/// OneTick traded pair reader config.
pub struct OneTickTradedPairReaderConfig<ExchangeID, Symbol, Settlement>
    where ExchangeID: Id,
          Symbol: Id,
          Settlement: GetSettlementLag
{
    /// Exchange ID.
    pub exchange_id: ExchangeID,
    /// Traded pair.
    pub traded_pair: TradedPair<Symbol, Settlement>,
    /// Path to file containing paths to files with PRL-ticks.
    pub prl_files: PathBuf,
    /// PRL-reader configuration.
    pub prl_args: OneTickTrdPrlConfig,
    /// Path to file containing paths to files with TRD-ticks.
    pub trd_files: PathBuf,
    /// TRD-reader configuration.
    pub trd_args: OneTickTrdPrlConfig,
    /// File for logging errors.
    pub err_log_file: Option<PathBuf>,
}

impl<ExchangeID, Symbol, Settlement>
From<&OneTickTradedPairReaderConfig<ExchangeID, Symbol, Settlement>>
for OneTickTradedPairReader<ExchangeID, Symbol, Settlement>
    where ExchangeID: Id,
          Symbol: Id,
          Settlement: GetSettlementLag
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
/// Initializer-config for [`OneTickReplay`].
pub struct OneTickReplayConfig<ExchangeID, Symbol, ObSnapshotDelay, Settlement>
    where ExchangeID: Id,
          Symbol: Id,
          ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>,
          Settlement: GetSettlementLag
{
    /// Start datetime.
    pub start_dt: DateTime,
    /// Traded pair configs.
    pub traded_pair_configs: Vec<OneTickTradedPairReaderConfig<ExchangeID, Symbol, Settlement>>,
    /// Exchange sessions.
    pub exchange_open_close_events: Vec<ExchangeSession<ExchangeID>>,
    /// Traded pair lifetimes.
    pub traded_pair_lifetimes: Vec<TradedPairLifetime<ExchangeID, Symbol, Settlement>>,
    /// OB-snapshot delay scheduler.
    pub ob_snapshot_delay_scheduler: ObSnapshotDelay,
}

impl<BrokerID, ExchangeID, Symbol, ObSnapshotDelay, Settlement>
From<&OneTickReplayConfig<ExchangeID, Symbol, ObSnapshotDelay, Settlement>>
for OneTickReplay<BrokerID, ExchangeID, Symbol, ObSnapshotDelay, Settlement>
    where BrokerID: Id,
          ExchangeID: Id,
          Symbol: Id,
          ObSnapshotDelay: Clone + GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>,
          Settlement: GetSettlementLag
{
    fn from(cfg: &OneTickReplayConfig<ExchangeID, Symbol, ObSnapshotDelay, Settlement>) -> Self {
        Self::new(
            cfg.start_dt,
            cfg.traded_pair_configs.iter().map(From::from),
            cfg.exchange_open_close_events.iter().cloned(),
            cfg.traded_pair_lifetimes.iter().cloned(),
            cfg.ob_snapshot_delay_scheduler.clone(),
        )
    }
}

impl<ExchangeID, BrokerID, Symbol, Settlement>
From<&ExchangeID>
for BasicExchange<ExchangeID, BrokerID, Symbol, Settlement>
    where ExchangeID: Id,
          BrokerID: Id,
          Symbol: Id,
          Settlement: GetSettlementLag
{
    fn from(exchange_id: &ExchangeID) -> Self {
        Self::new(*exchange_id)
    }
}

impl<BrokerID, TraderID, ExchangeID, Symbol, Settlement>
From<&BrokerID>
for BasicBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>
    where BrokerID: Id,
          TraderID: Id,
          ExchangeID: Id,
          Symbol: Id,
          Settlement: GetSettlementLag
{
    fn from(broker_id: &BrokerID) -> Self {
        Self::new(*broker_id)
    }
}

#[derive(Clone, Copy)]
/// Initializer-config for [`SpreadWriter`].
pub struct SpreadWriterConfig<TraderID, PS, F>
    where TraderID: Id,
          PS: Into<PriceStep> + Copy,
          F: AsRef<Path>
{
    /// ID of the `SpreadWriter`.
    pub name: TraderID,
    /// File to write spread into.
    pub file: F,
    /// Quoting price step.
    pub price_step: PS,
}

impl<TraderID, PS, F>
SpreadWriterConfig<TraderID, PS, F>
    where TraderID: Id,
          PS: Into<PriceStep> + Copy,
          F: AsRef<Path>
{
    /// Creates a new instance of the [`SpreadWriterConfig`].
    ///
    /// # Arguments
    ///
    /// * `name` — ID of the `SpreadWriter`.
    /// * `file` — File to write spread into.
    /// * `price_step` — Price quotation step.
    pub fn new(name: TraderID, file: F, price_step: PS) -> Self {
        Self { name, file, price_step }
    }
}

impl<TraderID, BrokerID, ExchangeID, Symbol, Settlement, PS, F>
From<&SpreadWriterConfig<TraderID, PS, F>>
for SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
    where TraderID: Id,
          BrokerID: Id,
          ExchangeID: Id,
          Symbol: Id,
          Settlement: GetSettlementLag,
          PS: Into<PriceStep> + Copy,
          F: AsRef<Path>
{
    fn from(cfg: &SpreadWriterConfig<TraderID, PS, F>) -> Self {
        Self::new(cfg.name, cfg.price_step, &cfg.file)
    }
}