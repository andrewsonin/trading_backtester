use {
    crate::{
        broker::{BrokerToExchange, BrokerToItself, BrokerToTrader},
        exchange::{ExchangeToBroker, ExchangeToItself, ExchangeToReplay},
        kernel::KernelBuilder,
        replay::{ReplayToExchange, ReplayToItself},
        trader::{TraderToBroker, TraderToItself},
        types::{DateTime, Id},
        utils::{
            input::config::from_structs::{BuildBroker, BuildExchange, BuildReplay, BuildTrader},
            rand::{Rng, rngs::StdRng, SeedableRng},
        },
    },
    rayon::{iter::{IntoParallelIterator, ParallelIterator}, ThreadPoolBuilder},
    std::marker::PhantomData,
};

#[derive(Clone)]
pub struct ThreadConfig<
    TraderID: Id,
    BrokerID: Id,
    ExchangeID: Id,
    R2R: ReplayToItself,
    R2E: ReplayToExchange<ExchangeID=ExchangeID>,
    B2E: BrokerToExchange<ExchangeID=ExchangeID>,
    B2T: BrokerToTrader<TraderID=TraderID>,
    B2B: BrokerToItself,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    T2T: TraderToItself,
    E2R: ExchangeToReplay,
    E2B: ExchangeToBroker<BrokerID=BrokerID>,
    E2E: ExchangeToItself,
    ReplayConfig: BuildReplay<ExchangeID, E2R, R2R, R2E>,
    TraderConfigs: IntoIterator<Item=(TraderConfig, ConnectedBrokers)>,
    TraderConfig,
    ConnectedBrokers,
    SubCfg,
    SubscriptionConfigs
>
    where TraderConfig: BuildTrader<TraderID, BrokerID, B2T, T2B, T2T>,
          ConnectedBrokers: IntoIterator<Item=(BrokerID, SubscriptionConfigs)>,
          SubscriptionConfigs: IntoIterator<Item=SubCfg>
{
    rng_seed: u64,
    replay_config: ReplayConfig,
    trader_configs: TraderConfigs,
    phantom_data: PhantomData<
        (TraderID, ExchangeID, T2B, T2T, B2E, B2T, B2B, E2R, E2B, E2E, R2R, R2E)
    >,
}

impl<
    TraderID: Id,
    BrokerID: Id,
    ExchangeID: Id,
    R2R: ReplayToItself,
    R2E: ReplayToExchange<ExchangeID=ExchangeID>,
    B2E: BrokerToExchange<ExchangeID=ExchangeID>,
    B2T: BrokerToTrader<TraderID=TraderID>,
    B2B: BrokerToItself,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    T2T: TraderToItself,
    E2R: ExchangeToReplay,
    E2B: ExchangeToBroker<BrokerID=BrokerID>,
    E2E: ExchangeToItself,
    ReplayConfig: BuildReplay<ExchangeID, E2R, R2R, R2E>,
    TraderConfigs: IntoIterator<Item=(TraderConfig, ConnectedBrokers)>,
    TraderConfig,
    ConnectedBrokers,
    SubCfg,
    SubscriptionConfigs
>
ThreadConfig<
    TraderID, BrokerID, ExchangeID,
    R2R, R2E, B2E, B2T, B2B, T2B, T2T, E2R, E2B, E2E,
    ReplayConfig, TraderConfigs, TraderConfig, ConnectedBrokers,
    SubCfg, SubscriptionConfigs
>
    where TraderConfig: BuildTrader<TraderID, BrokerID, B2T, T2B, T2T>,
          ConnectedBrokers: IntoIterator<Item=(BrokerID, SubscriptionConfigs)>,
          SubscriptionConfigs: IntoIterator<Item=SubCfg>
{
    pub fn new(rng_seed: u64, replay_config: ReplayConfig, trader_configs: TraderConfigs) -> Self {
        Self {
            rng_seed,
            replay_config,
            trader_configs,
            phantom_data: Default::default(),
        }
    }
}

pub struct ParallelBacktester<
    TraderID: Id,
    BrokerID: Id,
    ExchangeID: Id,
    R2R: ReplayToItself,
    R2E: ReplayToExchange<ExchangeID=ExchangeID>,
    B2E: BrokerToExchange<ExchangeID=ExchangeID>,
    B2T: BrokerToTrader<TraderID=TraderID>,
    B2B: BrokerToItself,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    T2T: TraderToItself,
    E2R: ExchangeToReplay,
    E2B: ExchangeToBroker<BrokerID=BrokerID>,
    E2E: ExchangeToItself,
    ExchangeConfig,
    ReplayConfig,
    BrokerConfig,
    TraderConfig,
    ExchangeConfigs: IntoIterator<Item=ExchangeConfig>,
    BrokerConfigs: IntoIterator<Item=(BrokerConfig, ConnectedExchanges)>,
    PerThreadConfigs: IntoIterator<
        Item=ThreadConfig<
            TraderID, BrokerID, ExchangeID,
            R2R, R2E, B2E, B2T, B2B, T2B, T2T, E2R, E2B, E2E,
            ReplayConfig, TraderConfigs, TraderConfig, ConnectedBrokers,
            SubCfg, SubscriptionConfigs
        >
    >,
    TraderConfigs: IntoIterator<Item=(TraderConfig, ConnectedBrokers)>,
    ConnectedExchanges: IntoIterator<Item=ExchangeID>,
    ConnectedBrokers,
    SubCfg,
    SubscriptionConfigs: IntoIterator<Item=SubCfg>,
    RNG: SeedableRng + Rng
>
    where
        ExchangeConfig: Sync + BuildExchange<ExchangeID, BrokerID, R2E, B2E, E2R, E2B, E2E>,
        BrokerConfig: Sync + BuildBroker<BrokerID, TraderID, ExchangeID, E2B, T2B, B2E, B2T, B2B, SubCfg>,
        ReplayConfig: Send + BuildReplay<ExchangeID, E2R, R2R, R2E>,
        TraderConfig: Send + BuildTrader<TraderID, BrokerID, B2T, T2B, T2T>,
        ConnectedBrokers: Send + IntoIterator<Item=(BrokerID, SubscriptionConfigs)>
{
    num_threads: usize,
    exchange_configs: ExchangeConfigs,
    broker_configs: BrokerConfigs,
    per_thread_configs: PerThreadConfigs,
    date_range: (DateTime, DateTime),
    trader_id: PhantomData<TraderID>,
    rng: PhantomData<RNG>,
}

impl<
    TraderID: Id,
    BrokerID: Id,
    ExchangeID: Id,
    R2R: ReplayToItself,
    R2E: ReplayToExchange<ExchangeID=ExchangeID>,
    B2E: BrokerToExchange<ExchangeID=ExchangeID>,
    B2T: BrokerToTrader<TraderID=TraderID>,
    B2B: BrokerToItself,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    T2T: TraderToItself,
    E2R: ExchangeToReplay,
    E2B: ExchangeToBroker<BrokerID=BrokerID>,
    E2E: ExchangeToItself,
    ExchangeConfig,
    ReplayConfig,
    BrokerConfig,
    TraderConfig,
    ExchangeConfigs: IntoIterator<Item=ExchangeConfig>,
    BrokerConfigs: IntoIterator<Item=(BrokerConfig, ConnectedExchanges)>,
    PerThreadConfigs: IntoIterator<
        Item=ThreadConfig<
            TraderID, BrokerID, ExchangeID,
            R2R, R2E, B2E, B2T, B2B, T2B, T2T, E2R, E2B, E2E,
            ReplayConfig, TraderConfigs, TraderConfig, ConnectedBrokers,
            SubCfg, SubscriptionConfigs
        >
    >,
    TraderConfigs: IntoIterator<Item=(TraderConfig, ConnectedBrokers)>,
    ConnectedExchanges: IntoIterator<Item=ExchangeID>,
    ConnectedBrokers,
    SubCfg,
    SubscriptionConfigs: IntoIterator<Item=SubCfg>,
>
ParallelBacktester<
    TraderID, BrokerID, ExchangeID,
    R2R, R2E, B2E, B2T, B2B, T2B, T2T, E2R, E2B, E2E,
    ExchangeConfig, ReplayConfig, BrokerConfig, TraderConfig,
    ExchangeConfigs, BrokerConfigs, PerThreadConfigs, TraderConfigs,
    ConnectedExchanges, ConnectedBrokers,
    SubCfg, SubscriptionConfigs, StdRng
>
    where
        ExchangeConfig: Sync + BuildExchange<ExchangeID, BrokerID, R2E, B2E, E2R, E2B, E2E>,
        BrokerConfig: Sync + BuildBroker<BrokerID, TraderID, ExchangeID, E2B, T2B, B2E, B2T, B2B, SubCfg>,
        ReplayConfig: Send + BuildReplay<ExchangeID, E2R, R2R, R2E>,
        TraderConfig: Send + BuildTrader<TraderID, BrokerID, B2T, T2B, T2T>,
        ConnectedBrokers: Send + IntoIterator<Item=(BrokerID, SubscriptionConfigs)>
{
    pub fn new(
        exchange_configs: ExchangeConfigs,
        broker_configs: BrokerConfigs,
        per_thread_configs: PerThreadConfigs,
        date_range: (DateTime, DateTime)) -> Self
    {
        Self {
            num_threads: 0,
            exchange_configs,
            broker_configs,
            per_thread_configs,
            date_range,
            trader_id: Default::default(),
            rng: Default::default(),
        }
    }

    pub fn with_rng<RNG: SeedableRng + Rng>(self) -> ParallelBacktester<
        TraderID, BrokerID, ExchangeID,
        R2R, R2E, B2E, B2T, B2B, T2B, T2T, E2R, E2B, E2E,
        ExchangeConfig, ReplayConfig, BrokerConfig, TraderConfig,
        ExchangeConfigs, BrokerConfigs, PerThreadConfigs, TraderConfigs,
        ConnectedExchanges, ConnectedBrokers,
        SubCfg, SubscriptionConfigs, RNG
    > {
        let Self {
            num_threads,
            exchange_configs,
            broker_configs,
            per_thread_configs,
            date_range,
            ..
        } = self;
        ParallelBacktester {
            num_threads,
            exchange_configs,
            broker_configs,
            per_thread_configs,
            date_range,
            trader_id: Default::default(),
            rng: Default::default(),
        }
    }
}

impl<
    TraderID: Id,
    BrokerID: Id,
    ExchangeID: Id,
    R2R: ReplayToItself,
    R2E: ReplayToExchange<ExchangeID=ExchangeID>,
    B2E: BrokerToExchange<ExchangeID=ExchangeID>,
    B2T: BrokerToTrader<TraderID=TraderID>,
    B2B: BrokerToItself,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    T2T: TraderToItself,
    E2R: ExchangeToReplay,
    E2B: ExchangeToBroker<BrokerID=BrokerID>,
    E2E: ExchangeToItself,
    ExchangeConfig,
    ReplayConfig,
    BrokerConfig,
    TraderConfig,
    ExchangeConfigs: IntoIterator<Item=ExchangeConfig>,
    BrokerConfigs: IntoIterator<Item=(BrokerConfig, ConnectedExchanges)>,
    PerThreadConfigs: IntoIterator<
        Item=ThreadConfig<
            TraderID, BrokerID, ExchangeID,
            R2R, R2E, B2E, B2T, B2B, T2B, T2T, E2R, E2B, E2E,
            ReplayConfig, TraderConfigs, TraderConfig, ConnectedBrokers,
            SubCfg, SubscriptionConfigs
        >
    >,
    TraderConfigs: IntoIterator<Item=(TraderConfig, ConnectedBrokers)>,
    ConnectedExchanges: IntoIterator<Item=ExchangeID>,
    ConnectedBrokers,
    SubCfg,
    SubscriptionConfigs: IntoIterator<Item=SubCfg>,
    RNG: Rng + SeedableRng
>
ParallelBacktester<
    TraderID, BrokerID, ExchangeID,
    R2R, R2E, B2E, B2T, B2B, T2B, T2T, E2R, E2B, E2E,
    ExchangeConfig, ReplayConfig, BrokerConfig, TraderConfig, ExchangeConfigs, BrokerConfigs,
    PerThreadConfigs, TraderConfigs,
    ConnectedExchanges, ConnectedBrokers,
    SubCfg, SubscriptionConfigs, RNG
>
    where
        ExchangeConfig: Sync + BuildExchange<ExchangeID, BrokerID, R2E, B2E, E2R, E2B, E2E>,
        BrokerConfig: Sync + BuildBroker<BrokerID, TraderID, ExchangeID, E2B, T2B, B2E, B2T, B2B, SubCfg>,
        ReplayConfig: Send + BuildReplay<ExchangeID, E2R, R2R, R2E>,
        TraderConfig: Send + BuildTrader<TraderID, BrokerID, B2T, T2B, T2T>,
        ConnectedBrokers: Send + IntoIterator<Item=(BrokerID, SubscriptionConfigs)>
{
    pub fn with_num_threads(mut self, num_threads: usize) -> Self {
        self.num_threads = num_threads;
        self
    }

    pub fn run_simulation(self) {
        let Self {
            num_threads,
            exchange_configs,
            broker_configs,
            per_thread_configs,
            date_range,
            ..
        } = self;
        let exchange_configs: Vec<_> = exchange_configs.into_iter().collect();
        let broker_configs: Vec<(_, Vec<_>)> = broker_configs.into_iter()
            .map(
                |(broker_cfg, connected_exchanges)|
                    (broker_cfg, connected_exchanges.into_iter().collect())
            )
            .collect();
        let per_thread_configs: Vec<(_, _, Vec<_>)> = per_thread_configs.into_iter()
            .map(
                |ThreadConfig { rng_seed, replay_config, trader_configs, .. }|
                    (rng_seed, replay_config, trader_configs.into_iter().collect())
            )
            .collect();

        let job = || per_thread_configs.into_par_iter().for_each(
            |(rng_seed, replay_config, trader_configs)| {
                let exchanges = exchange_configs.iter().map(BuildExchange::build);
                let brokers = broker_configs.iter().map(
                    |(broker_cfg, connected_exchanges)|
                        (BuildBroker::build(broker_cfg), connected_exchanges.iter().cloned())
                );
                let traders = trader_configs.into_iter().map(
                    |(trader_config, connected_brokers)|
                        (BuildTrader::build(&trader_config), connected_brokers)
                );
                let replay = BuildReplay::build(&replay_config);
                KernelBuilder::new(exchanges, brokers, traders, replay, date_range)
                    .with_rng::<RNG>()
                    .with_seed(rng_seed)
                    .build()
                    .run_simulation()
            }
        );
        if num_threads == 0 {
            job()
        } else {
            ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .unwrap_or_else(
                    |err| panic!(
                        "Cannot build ThreadPool \
                        with the following number of threads to use: {num_threads}. \
                        Error: {err}"
                    )
                )
                .install(job)
        }
    }
}