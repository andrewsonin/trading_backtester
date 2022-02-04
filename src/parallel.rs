use {
    crate::{
        interface::{broker::Broker, exchange::Exchange, replay::Replay, trader::Trader},
        kernel::KernelBuilder,
        types::{DateTime, Id},
    },
    rand::{Rng, rngs::StdRng, SeedableRng},
    rayon::{iter::{IntoParallelIterator, ParallelIterator}, ThreadPoolBuilder},
    std::marker::PhantomData,
};

#[derive(Clone)]
pub struct ThreadConfig<ReplayConfig, TraderConfigs: IntoIterator> {
    rng_seed: u64,
    replay_config: ReplayConfig,
    trader_configs: TraderConfigs,
}

impl<ReplayConfig, TraderConfigs: IntoIterator>
ThreadConfig<ReplayConfig, TraderConfigs>
{
    pub fn new(rng_seed: u64, replay_config: ReplayConfig, trader_configs: TraderConfigs) -> Self {
        Self {
            rng_seed,
            replay_config,
            trader_configs,
        }
    }
}

pub struct ParallelBacktester<BrokerConfigs, ExchangeConfigs, PerThreadConfs, RNG>
{
    exchange_configs: ExchangeConfigs,
    broker_configs: BrokerConfigs,
    per_thread_configs: PerThreadConfs,
    date_range: (DateTime, DateTime),

    num_threads: usize,
    phantom: PhantomData<RNG>,
}

impl<B, E, T> ParallelBacktester<B, E, T, StdRng>
    where B: IntoIterator,
          E: IntoIterator,
          T: IntoIterator
{
    pub fn new(
        exchange_configs: E,
        broker_configs: B,
        per_thread_configs: T,
        date_range: (DateTime, DateTime)) -> Self
    {
        ParallelBacktester {
            exchange_configs,
            broker_configs,
            per_thread_configs,
            date_range,
            num_threads: 0,
            phantom: Default::default(),
        }
    }

    pub fn with_rng<RNG: Rng + SeedableRng>(self) -> ParallelBacktester<B, E, T, RNG> {
        let Self {
            exchange_configs,
            broker_configs,
            per_thread_configs,
            date_range,
            num_threads,
            ..
        } = self;
        ParallelBacktester {
            exchange_configs,
            broker_configs,
            per_thread_configs,
            date_range,
            num_threads,
            phantom: Default::default(),
        }
    }
}

impl<BrokerConfigs, ExchangeConfigs, PerThreadConfigs, RNG>
ParallelBacktester<BrokerConfigs, ExchangeConfigs, PerThreadConfigs, RNG>
    where BrokerConfigs: IntoIterator,
          ExchangeConfigs: IntoIterator,
          PerThreadConfigs: IntoIterator,
          RNG: Rng + SeedableRng
{
    pub fn with_num_threads(mut self, num_threads: usize) -> Self {
        self.num_threads = num_threads;
        self
    }
}

impl<
    BrokerID, ExchangeID, TraderConfig, BrokerConfig, ReplayConfig, ExchangeConfig,
    TraderConfigs, BrokerConfigs, ExchangeConfigs, PerThreadConfigs, ConnectedBrokers,
    ConnectedExchanges, SubscriptionConfigs, RNG, SubCfg,
>
ParallelBacktester<BrokerConfigs, ExchangeConfigs, PerThreadConfigs, RNG>
    where BrokerID: Id,
          ExchangeID: Id,
          TraderConfig: Send,
          BrokerConfig: Sync,
          ExchangeConfig: Sync,
          ReplayConfig: Send,
          TraderConfigs: IntoIterator<Item=(TraderConfig, ConnectedBrokers)>,
          BrokerConfigs: IntoIterator<Item=(BrokerConfig, ConnectedExchanges)>,
          ExchangeConfigs: IntoIterator<Item=ExchangeConfig>,
          PerThreadConfigs: IntoIterator<Item=ThreadConfig<ReplayConfig, TraderConfigs>>,
          ConnectedBrokers: Send + IntoIterator<Item=(BrokerID, SubscriptionConfigs)>,
          ConnectedExchanges: IntoIterator<Item=ExchangeID>,
          SubscriptionConfigs: IntoIterator<Item=SubCfg>,
          RNG: Rng + SeedableRng
{
    pub fn run_simulation<T, B, E, R>(self)
        where
            T: for<'a> From<&'a TraderConfig>,
            B: for<'a> From<&'a BrokerConfig>,
            E: for<'a> From<&'a ExchangeConfig>,
            R: for<'a> From<&'a ReplayConfig>,
            T: Trader<TraderID=B::TraderID, BrokerID=BrokerID, T2B=B::T2B, B2T=B::B2T>,
            B: Broker<BrokerID=BrokerID, ExchangeID=ExchangeID, B2R=R::B2R, R2B=R::R2B, SubCfg=SubCfg>,
            E: Exchange<BrokerID=BrokerID, ExchangeID=ExchangeID, E2R=R::E2R, R2E=R::R2E, B2E=B::B2E, E2B=B::E2B>,
            R: Replay<BrokerID=BrokerID, ExchangeID=ExchangeID>
    {
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
                let exchanges = exchange_configs.iter().map(E::from);
                let brokers = broker_configs.iter().map(
                    |(broker_cfg, connected_exchanges)|
                        (B::from(broker_cfg), connected_exchanges.iter().cloned())
                );
                let traders = trader_configs.into_iter().map(
                    |(trader_config, connected_brokers)|
                        (T::from(&trader_config), connected_brokers)
                );
                let replay = R::from(&replay_config);
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