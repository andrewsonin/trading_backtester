use {
    crate::{
        broker::{
            Broker,
            BrokerToExchange,
            BrokerToItself,
            BrokerToTrader,
            concrete::BasicBroker,
            reply::BasicBrokerToTrader,
            request::BasicBrokerToExchange,
        },
        exchange::{
            concrete::BasicExchange,
            Exchange,
            ExchangeToBroker,
            ExchangeToItself,
            ExchangeToReplay,
            reply::{BasicExchangeToBroker, BasicExchangeToReplay},
        },
        replay::{
            concrete::{ExchangeSession, GetNextObSnapshotDelay, OneTickReplay, TradedPairLifetime},
            Replay,
            ReplayToExchange,
            ReplayToItself,
            request::BasicReplayToExchange,
        },
        settlement::GetSettlementLag,
        traded_pair::TradedPair,
        trader::{
            concrete::SpreadWriter,
            request::BasicTraderToBroker,
            subscriptions::SubscriptionConfig,
            Trader,
            TraderToBroker,
            TraderToItself,
        },
        types::{DateTime, Id, Nothing, PriceStep},
        utils::input::one_tick::{OneTickTradedPairReader, OneTickTrdPrlConfig},
    },
    std::path::{Path, PathBuf},
};

pub trait BuildExchange<
    ExchangeID: Id,
    BrokerID: Id,
    R2E: ReplayToExchange<ExchangeID=ExchangeID>,
    B2E: BrokerToExchange<ExchangeID=ExchangeID>,
    E2R: ExchangeToReplay,
    E2B: ExchangeToBroker<BrokerID=BrokerID>,
    E2E: ExchangeToItself
> {
    type E: Exchange<ExchangeID, BrokerID, R2E, B2E, E2R, E2B, E2E>;

    fn build(&self) -> Self::E;
}

pub trait BuildReplay<
    ExchangeID: Id,
    E2R: ExchangeToReplay,
    R2R: ReplayToItself,
    R2E: ReplayToExchange<ExchangeID=ExchangeID>
> {
    type R: Replay<ExchangeID, E2R, R2R, R2E>;

    fn build(&self) -> Self::R;
}

pub trait BuildBroker<
    BrokerID: Id,
    TraderID: Id,
    ExchangeID: Id,
    E2B: ExchangeToBroker<BrokerID=BrokerID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    B2E: BrokerToExchange<ExchangeID=ExchangeID>,
    B2T: BrokerToTrader<TraderID=TraderID>,
    B2B: BrokerToItself,
    SubCfg
> {
    type B: Broker<BrokerID, TraderID, ExchangeID, E2B, T2B, B2E, B2T, B2B, SubCfg>;

    fn build(&self) -> Self::B;
}

pub trait BuildTrader<
    TraderID: Id,
    BrokerID: Id,
    B2T: BrokerToTrader<TraderID=TraderID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    T2T: TraderToItself
> {
    type T: Trader<TraderID, BrokerID, B2T, T2B, T2T>;

    fn build(&self) -> Self::T;
}

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
    ExchangeID: Id,
    Symbol: Id,
    ObSnapshotDelay: Clone + GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>,
    Settlement: GetSettlementLag
>
BuildReplay<
    ExchangeID,
    BasicExchangeToReplay<Symbol, Settlement>,
    Nothing,
    BasicReplayToExchange<ExchangeID, Symbol, Settlement>
>
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
    ExchangeID: Id + InitBasicExchange,
    BrokerID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
>
BuildExchange<
    ExchangeID,
    BrokerID,
    BasicReplayToExchange<ExchangeID, Symbol, Settlement>,
    BasicBrokerToExchange<ExchangeID, Symbol, Settlement>,
    BasicExchangeToReplay<Symbol, Settlement>,
    BasicExchangeToBroker<BrokerID, Symbol, Settlement>,
    Nothing
>
for ExchangeID
{
    type E = BasicExchange<ExchangeID, BrokerID, Symbol, Settlement>;

    fn build(&self) -> Self::E {
        Self::E::new(*self)
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
BuildBroker<
    BrokerID,
    TraderID,
    ExchangeID,
    BasicExchangeToBroker<BrokerID, Symbol, Settlement>,
    BasicTraderToBroker<BrokerID, ExchangeID, Symbol, Settlement>,
    BasicBrokerToExchange<ExchangeID, Symbol, Settlement>,
    BasicBrokerToTrader<TraderID, ExchangeID, Symbol, Settlement>,
    Nothing,
    SubscriptionConfig<ExchangeID, Symbol, Settlement>
>
for BrokerID
{
    type B = BasicBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>;

    fn build(&self) -> Self::B {
        Self::B::new(*self)
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
        Self {
            name,
            file,
            price_step,
        }
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
BuildTrader<
    TraderID,
    BrokerID,
    BasicBrokerToTrader<TraderID, ExchangeID, Symbol, Settlement>,
    BasicTraderToBroker<BrokerID, ExchangeID, Symbol, Settlement>,
    Nothing
>
for SpreadWriterConfig<TraderID, PS, F>
{
    type T = SpreadWriter<TraderID>;

    fn build(&self) -> Self::T {
        Self::T::new(self.name, self.price_step, &self.file)
    }
}