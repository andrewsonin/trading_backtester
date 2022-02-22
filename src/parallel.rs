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
/// Initializer struct that contain thread-unique information.
/// Here it is the RNG seed, the initializer config for building possibly thread-unique
/// entities.
pub struct ThreadConfig<ReplayConfig, ExchangeConfigs, BrokerConfigs, TraderConfigs> {
    rng_seed: u64,
    replay_config: ReplayConfig,
    trader_configs: TraderConfigs,
    broker_configs: BrokerConfigs,
    exchange_configs: ExchangeConfigs,
}

impl<ReplayConfig, ExchangeConfigs, BrokerConfigs, TraderConfigs>
ThreadConfig<ReplayConfig, ExchangeConfigs, BrokerConfigs, TraderConfigs>
{
    /// Creates a new instance of the [`ThreadConfig`].
    ///
    /// # Arguments
    ///
    /// * `rng_seed` — RNG seed.
    /// * `replay_config` — [`Replay`] initializer config.
    /// * `exchange_configs` — [`Exchange`] initializer configs.
    /// * `broker_configs` — [`Broker`] initializer configs.
    /// * `trader_configs` — [`Trader`] initializer configs.
    pub fn new(
        rng_seed: u64,
        replay_config: ReplayConfig,
        exchange_configs: ExchangeConfigs,
        broker_configs: BrokerConfigs,
        trader_configs: TraderConfigs) -> Self
    {
        Self {
            rng_seed,
            replay_config,
            trader_configs,
            broker_configs,
            exchange_configs,
        }
    }
}

/// Parallels simultaneous runs of multiple [`Kernels`](crate::kernel::Kernel).
pub struct ParallelBacktester<PerThreadConfs, RNG>
{
    per_thread_configs: PerThreadConfs,
    date_range: (DateTime, DateTime),

    num_threads: usize,
    phantom: PhantomData<RNG>,
}

impl<T> ParallelBacktester<T, StdRng>
    where T: IntoIterator
{
    /// Creates a new instance of the [`ParallelBacktester`].
    ///
    /// # Arguments
    ///
    /// * `per_thread_configs` — Thread-unique initializer configs.
    /// * `date_range` — Tuple of start and stop [`DateTimes`](crate::types::DateTime).
    pub fn new(
        per_thread_configs: T,
        date_range: (DateTime, DateTime)) -> Self
    {
        ParallelBacktester {
            per_thread_configs,
            date_range,
            num_threads: 0,
            phantom: Default::default(),
        }
    }

    /// Sets non-default ([`StdRng`]) random number generator.
    pub fn with_rng<RNG: Rng + SeedableRng>(self) -> ParallelBacktester<T, RNG> {
        let Self {
            per_thread_configs,
            date_range,
            num_threads,
            ..
        } = self;
        ParallelBacktester {
            per_thread_configs,
            date_range,
            num_threads,
            phantom: Default::default(),
        }
    }
}

impl<PerThreadConfigs, RNG>
ParallelBacktester<PerThreadConfigs, RNG>
    where PerThreadConfigs: IntoIterator,
          RNG: Rng + SeedableRng
{
    /// Sets the number of threads in a thread pool.
    ///
    /// # Arguments
    ///
    /// * `num_threads` — Number of threads in a thread pool.
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
ParallelBacktester<PerThreadConfigs, RNG>
    where BrokerID: Id,
          ExchangeID: Id,
          TraderConfig: Send,
          BrokerConfig: Send,
          ExchangeConfig: Send,
          ReplayConfig: Send,
          TraderConfigs: IntoIterator<Item=(TraderConfig, ConnectedBrokers)>,
          BrokerConfigs: IntoIterator<Item=(BrokerConfig, ConnectedExchanges)>,
          ExchangeConfigs: IntoIterator<Item=ExchangeConfig>,
          PerThreadConfigs: IntoIterator<Item=ThreadConfig<ReplayConfig, ExchangeConfigs, BrokerConfigs, TraderConfigs>>,
          ConnectedBrokers: Send + IntoIterator<Item=(BrokerID, SubscriptionConfigs)>,
          ConnectedExchanges: Send + IntoIterator<Item=ExchangeID>,
          SubscriptionConfigs: IntoIterator<Item=SubCfg>,
          RNG: Rng + SeedableRng
{
    /// Runs final simulation.
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
        let Self { num_threads, per_thread_configs, date_range, .. } = self;
        let per_thread_configs: Vec<(_, _, Vec<_>, Vec<_>, Vec<_>)> = per_thread_configs.into_iter()
            .map(
                |ThreadConfig {
                     rng_seed, replay_config, trader_configs,
                     broker_configs, exchange_configs
                 }|
                    (
                        rng_seed,
                        replay_config,
                        exchange_configs.into_iter().collect(),
                        broker_configs.into_iter().collect(),
                        trader_configs.into_iter().collect()
                    )
            )
            .collect();

        let job = || per_thread_configs.into_par_iter().for_each(
            |(rng_seed, replay_config, exchange_configs, broker_configs, trader_configs)| {
                let exchanges = exchange_configs.iter().map(E::from);
                let brokers = broker_configs.into_iter().map(
                    |(broker_cfg, connected_exchanges)|
                        (B::from(&broker_cfg), connected_exchanges)
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