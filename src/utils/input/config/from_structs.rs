use {
    crate::{
        broker::concrete::BasicBroker,
        exchange::concrete::BasicExchange,
        replay::concrete::{
            ExchangeSession, GetNextObSnapshotDelay, OneTickReplay, TradedPairLifetime
        },
        traded_pair::TradedPair,
        trader::concrete::{SpreadWriter, VoidTrader},
        types::{DateTime, Identifier, PriceStep},
        utils::input::{config::FromConfig, one_tick::{OneTickTradedPairReader, TrdPrlInfo}},
    },
    std::path::Path,
};

pub struct OneTickTradedPairReaderConfig<ExchangeID: Identifier, Symbol: Identifier>
{
    pub exchange_id: ExchangeID,
    pub traded_pair: TradedPair<Symbol>,
    pub prl_files: String,
    pub prl_args: TrdPrlInfo,
    pub trd_files: String,
    pub trd_args: TrdPrlInfo,
    pub err_log_file: Option<String>,
}

impl<ExchangeID: Identifier, Symbol: Identifier>
FromConfig<OneTickTradedPairReaderConfig<ExchangeID, Symbol>>
for OneTickTradedPairReader<ExchangeID, Symbol>
{
    fn from_config(config: &OneTickTradedPairReaderConfig<ExchangeID, Symbol>) -> Self {
        OneTickTradedPairReader::new(
            config.exchange_id,
            config.traded_pair,
            &config.prl_files,
            config.prl_args.clone(),
            &config.trd_files,
            config.trd_args.clone(),
            if let Some(file) = &config.err_log_file {
                Some(&file)
            } else {
                None
            },
        )
    }
}

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
FromConfig<OneTickReplayConfig<ExchangeID, Symbol, ObSnapshotDelay>>
for OneTickReplay<ExchangeID, Symbol, ObSnapshotDelay>
{
    fn from_config(config: &OneTickReplayConfig<ExchangeID, Symbol, ObSnapshotDelay>) -> Self
    {
        Self::new(
            config.start_dt,
            config.traded_pair_configs.iter().map(FromConfig::from_config),
            config.exchange_open_close_events.iter().cloned(),
            config.traded_pair_creation_events.iter().cloned(),
            config.ob_snapshot_delay_scheduler.clone(),
        )
    }
}

impl<ExchangeID: Identifier, BrokerID: Identifier, Symbol: Identifier>
FromConfig<ExchangeID>
for BasicExchange<ExchangeID, BrokerID, Symbol>
{
    fn from_config(config: &ExchangeID) -> Self {
        Self::new(*config)
    }
}

impl<BrokerID: Identifier, TraderID: Identifier, ExchangeID: Identifier, Symbol: Identifier>
FromConfig<BrokerID>
for BasicBroker<BrokerID, TraderID, ExchangeID, Symbol>
{
    fn from_config(config: &BrokerID) -> Self {
        Self::new(*config)
    }
}

impl<TraderID: Identifier> FromConfig<TraderID> for VoidTrader<TraderID> {
    fn from_config(config: &TraderID) -> Self {
        VoidTrader::new(*config)
    }
}

pub struct SpreadWriterConfig<TraderID: Identifier, PS: Into<PriceStep> + Copy, F: AsRef<Path>> {
    pub name: TraderID,
    pub file: F,
    pub price_step: PS,
}

impl<TraderID: Identifier, PS: Into<PriceStep> + Copy, F: AsRef<Path>>
FromConfig<SpreadWriterConfig<TraderID, PS, F>>
for SpreadWriter<TraderID>
{
    fn from_config(config: &SpreadWriterConfig<TraderID, PS, F>) -> Self {
        Self::new(config.name, config.price_step, &config.file)
    }
}