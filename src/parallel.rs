use {
    crate::{
        kernel::KernelBuilder,
        settlement::GetSettlementLag,
        trader::subscriptions::SubscriptionConfig,
        types::{DateTime, Identifier},
        utils::{
            ExpectWith,
            input::config::from_structs::{BuildBroker, BuildExchange, BuildReplay, BuildTrader},
            rand::{Rng, rngs::StdRng, SeedableRng},
        },
    },
    rayon::{iter::{IntoParallelIterator, ParallelIterator}, ThreadPoolBuilder},
    std::marker::PhantomData,
};

#[derive(Clone)]
pub struct ThreadConfig<
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag,
    ReplayConfig: BuildReplay<ExchangeID, Symbol, Settlement>,
    TraderConfigs: IntoIterator<Item=(TraderConfig, ConnectedBrokers)>,
    TraderConfig,
    ConnectedBrokers,
    SubscriptionConfigs
>
    where TraderConfig: BuildTrader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>,
          ConnectedBrokers: IntoIterator<Item=(BrokerID, SubscriptionConfigs)>,
          SubscriptionConfigs: IntoIterator<Item=SubscriptionConfig<ExchangeID, Symbol, Settlement>>
{
    pub rng_seed: u64,
    pub replay_config: ReplayConfig,
    pub trader_configs: TraderConfigs,
    trader_id: PhantomData<TraderID>,
}

impl<
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag,
    ReplayConfig: BuildReplay<ExchangeID, Symbol, Settlement>,
    TraderConfigs: IntoIterator<Item=(TraderConfig, ConnectedBrokers)>,
    TraderConfig,
    ConnectedBrokers,
    SubscriptionConfigs
>
ThreadConfig<
    TraderID, BrokerID, ExchangeID, Symbol, Settlement,
    ReplayConfig, TraderConfigs, TraderConfig,
    ConnectedBrokers, SubscriptionConfigs
>
    where TraderConfig: BuildTrader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>,
          ConnectedBrokers: IntoIterator<Item=(BrokerID, SubscriptionConfigs)>,
          SubscriptionConfigs: IntoIterator<Item=SubscriptionConfig<ExchangeID, Symbol, Settlement>>
{
    pub fn new(rng_seed: u64, replay_config: ReplayConfig, trader_configs: TraderConfigs) -> Self {
        Self {
            rng_seed,
            replay_config,
            trader_configs,
            trader_id: Default::default(),
        }
    }
}

pub struct ParallelBacktester<
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag,
    ExchangeConfig,
    ReplayConfig,
    BrokerConfig,
    TraderConfig,
    ExchangeConfigs: IntoIterator<Item=ExchangeConfig>,
    BrokerConfigs: IntoIterator<Item=(BrokerConfig, ConnectedExchanges)>,
    PerThreadConfigs: IntoIterator<
        Item=ThreadConfig<
            TraderID, BrokerID, ExchangeID, Symbol, Settlement,
            ReplayConfig, TraderConfigs, TraderConfig,
            ConnectedBrokers, SubscriptionConfigs
        >
    >,
    TraderConfigs: IntoIterator<Item=(TraderConfig, ConnectedBrokers)>,
    ConnectedExchanges: IntoIterator<Item=ExchangeID>,
    ConnectedBrokers,
    SubscriptionConfigs: IntoIterator<Item=SubscriptionConfig<ExchangeID, Symbol, Settlement>>,
    RNG: SeedableRng + Rng
>
    where
        ExchangeConfig: Sync + BuildExchange<ExchangeID, BrokerID, Symbol, Settlement>,
        BrokerConfig: Sync + BuildBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>,
        ReplayConfig: Send + BuildReplay<ExchangeID, Symbol, Settlement>,
        TraderConfig: Send + BuildTrader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>,
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
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag,
    ExchangeConfig,
    ReplayConfig,
    BrokerConfig,
    TraderConfig,
    ExchangeConfigs: IntoIterator<Item=ExchangeConfig>,
    BrokerConfigs: IntoIterator<Item=(BrokerConfig, ConnectedExchanges)>,
    PerThreadConfigs: IntoIterator<
        Item=ThreadConfig<
            TraderID, BrokerID, ExchangeID, Symbol, Settlement,
            ReplayConfig, TraderConfigs, TraderConfig,
            ConnectedBrokers, SubscriptionConfigs
        >
    >,
    TraderConfigs: IntoIterator<Item=(TraderConfig, ConnectedBrokers)>,
    ConnectedExchanges: IntoIterator<Item=ExchangeID>,
    ConnectedBrokers,
    SubscriptionConfigs: IntoIterator<Item=SubscriptionConfig<ExchangeID, Symbol, Settlement>>,
>
ParallelBacktester<
    TraderID, BrokerID, ExchangeID, Symbol, Settlement,
    ExchangeConfig, ReplayConfig, BrokerConfig, TraderConfig,
    ExchangeConfigs, BrokerConfigs, PerThreadConfigs, TraderConfigs,
    ConnectedExchanges, ConnectedBrokers, SubscriptionConfigs,
    StdRng
>
    where
        ExchangeConfig: Sync + BuildExchange<ExchangeID, BrokerID, Symbol, Settlement>,
        BrokerConfig: Sync + BuildBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>,
        ReplayConfig: Send + BuildReplay<ExchangeID, Symbol, Settlement>,
        TraderConfig: Send + BuildTrader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>,
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
        TraderID, BrokerID, ExchangeID, Symbol, Settlement,
        ExchangeConfig, ReplayConfig, BrokerConfig, TraderConfig,
        ExchangeConfigs, BrokerConfigs, PerThreadConfigs, TraderConfigs,
        ConnectedExchanges, ConnectedBrokers, SubscriptionConfigs,
        RNG
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
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag,
    ExchangeConfig,
    ReplayConfig,
    BrokerConfig,
    TraderConfig,
    ExchangeConfigs: IntoIterator<Item=ExchangeConfig>,
    BrokerConfigs: IntoIterator<Item=(BrokerConfig, ConnectedExchanges)>,
    PerThreadConfigs: IntoIterator<
        Item=ThreadConfig<
            TraderID, BrokerID, ExchangeID, Symbol, Settlement,
            ReplayConfig, TraderConfigs, TraderConfig,
            ConnectedBrokers, SubscriptionConfigs
        >
    >,
    TraderConfigs: IntoIterator<Item=(TraderConfig, ConnectedBrokers)>,
    ConnectedExchanges: IntoIterator<Item=ExchangeID>,
    ConnectedBrokers,
    SubscriptionConfigs: IntoIterator<Item=SubscriptionConfig<ExchangeID, Symbol, Settlement>>,
    RNG: SeedableRng + Rng
>
ParallelBacktester<
    TraderID, BrokerID, ExchangeID, Symbol, Settlement,
    ExchangeConfig, ReplayConfig, BrokerConfig, TraderConfig,
    ExchangeConfigs, BrokerConfigs, PerThreadConfigs, TraderConfigs,
    ConnectedExchanges, ConnectedBrokers, SubscriptionConfigs,
    RNG
>
    where
        ExchangeConfig: Sync + BuildExchange<ExchangeID, BrokerID, Symbol, Settlement>,
        BrokerConfig: Sync + BuildBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>,
        ReplayConfig: Send + BuildReplay<ExchangeID, Symbol, Settlement>,
        TraderConfig: Send + BuildTrader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>,
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
                .expect_with(
                    || panic!(
                        "Cannot build ThreadPool \
                        with the following number of threads to use: {num_threads}"
                    )
                )
                .install(job)
        }
    }
}