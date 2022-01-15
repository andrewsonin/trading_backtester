use {
    crate::{
        kernel::KernelBuilder,
        traded_pair::TradedPair,
        trader::subscriptions::SubscriptionList,
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

pub struct ParallelBacktester<
    TraderID, BrokerID, ExchangeID, Symbol,
    ExchangeConfig, ReplayConfig, BrokerConfig, TraderConfig,
    ExchangeConfigs, BrokerConfigs, PerThreadConfigs, TraderConfigs,
    ConnectedExchanges, ConnectedBrokers, SubscriptionConfigs,
    RNG
> where
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    ExchangeConfig: Sync + BuildExchange<ExchangeID, BrokerID, Symbol>,
    BrokerConfig: Sync + BuildBroker<BrokerID, TraderID, ExchangeID, Symbol>,
    ReplayConfig: Send + BuildReplay<ExchangeID, Symbol>,
    TraderConfig: Send + BuildTrader<TraderID, BrokerID, ExchangeID, Symbol>,
    ExchangeConfigs: IntoIterator<Item=ExchangeConfig>,
    BrokerConfigs: IntoIterator<Item=(BrokerConfig, ConnectedExchanges)>,
    PerThreadConfigs: IntoIterator<Item=(u64, ReplayConfig, TraderConfigs)>,
    TraderConfigs: IntoIterator<Item=(TraderConfig, ConnectedBrokers)>,
    ConnectedExchanges: IntoIterator<Item=ExchangeID>,
    ConnectedBrokers: Send + IntoIterator<Item=(BrokerID, SubscriptionConfigs)>,
    SubscriptionConfigs: IntoIterator<Item=(ExchangeID, TradedPair<Symbol>, SubscriptionList)>,
    RNG: SeedableRng + Rng
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
    TraderID, BrokerID, ExchangeID, Symbol,
    ExchangeConfig, ReplayConfig, BrokerConfig, TraderConfig,
    ExchangeConfigs, BrokerConfigs, PerThreadConfigs, TraderConfigs,
    ConnectedExchanges, ConnectedBrokers, SubscriptionConfigs
>
ParallelBacktester<
    TraderID, BrokerID, ExchangeID, Symbol,
    ExchangeConfig, ReplayConfig, BrokerConfig, TraderConfig,
    ExchangeConfigs, BrokerConfigs, PerThreadConfigs, TraderConfigs,
    ConnectedExchanges, ConnectedBrokers, SubscriptionConfigs,
    StdRng
> where
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    ExchangeConfig: Sync + BuildExchange<ExchangeID, BrokerID, Symbol>,
    BrokerConfig: Sync + BuildBroker<BrokerID, TraderID, ExchangeID, Symbol>,
    ReplayConfig: Send + BuildReplay<ExchangeID, Symbol>,
    TraderConfig: Send + BuildTrader<TraderID, BrokerID, ExchangeID, Symbol>,
    ExchangeConfigs: IntoIterator<Item=ExchangeConfig>,
    BrokerConfigs: IntoIterator<Item=(BrokerConfig, ConnectedExchanges)>,
    PerThreadConfigs: IntoIterator<Item=(u64, ReplayConfig, TraderConfigs)>,
    TraderConfigs: IntoIterator<Item=(TraderConfig, ConnectedBrokers)>,
    ConnectedExchanges: IntoIterator<Item=ExchangeID>,
    ConnectedBrokers: Send + IntoIterator<Item=(BrokerID, SubscriptionConfigs)>,
    SubscriptionConfigs: IntoIterator<Item=(ExchangeID, TradedPair<Symbol>, SubscriptionList)>
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
        TraderID, BrokerID, ExchangeID, Symbol,
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
    TraderID, BrokerID, ExchangeID, Symbol,
    ExchangeConfig, ReplayConfig, BrokerConfig, TraderConfig,
    ExchangeConfigs, BrokerConfigs, PerThreadConfigs, TraderConfigs,
    ConnectedExchanges, ConnectedBrokers, SubscriptionConfigs,
    RNG
>
ParallelBacktester<
    TraderID, BrokerID, ExchangeID, Symbol,
    ExchangeConfig, ReplayConfig, BrokerConfig, TraderConfig,
    ExchangeConfigs, BrokerConfigs, PerThreadConfigs, TraderConfigs,
    ConnectedExchanges, ConnectedBrokers, SubscriptionConfigs,
    RNG
> where
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    ExchangeConfig: Sync + BuildExchange<ExchangeID, BrokerID, Symbol>,
    BrokerConfig: Sync + BuildBroker<BrokerID, TraderID, ExchangeID, Symbol>,
    ReplayConfig: Send + BuildReplay<ExchangeID, Symbol>,
    TraderConfig: Send + BuildTrader<TraderID, BrokerID, ExchangeID, Symbol>,
    ExchangeConfigs: IntoIterator<Item=ExchangeConfig>,
    BrokerConfigs: IntoIterator<Item=(BrokerConfig, ConnectedExchanges)>,
    PerThreadConfigs: IntoIterator<Item=(u64, ReplayConfig, TraderConfigs)>,
    TraderConfigs: IntoIterator<Item=(TraderConfig, ConnectedBrokers)>,
    ConnectedExchanges: IntoIterator<Item=ExchangeID>,
    ConnectedBrokers: Send + IntoIterator<Item=(BrokerID, SubscriptionConfigs)>,
    SubscriptionConfigs: IntoIterator<Item=(ExchangeID, TradedPair<Symbol>, SubscriptionList)>,
    RNG: SeedableRng + Rng
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
                |(rng_seed, replay_config, trader_configs)|
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