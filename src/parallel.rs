use {
    crate::{
        broker::{BrokerToExchange, BrokerToItself, BrokerToTrader},
        exchange::{ExchangeToBroker, ExchangeToItself, ExchangeToReplay},
        kernel::KernelBuilder,
        replay::{ReplayToExchange, ReplayToItself},
        trader::{TraderToBroker, TraderToItself},
        types::{DateTime, Id},
        utils::{
            rand::{Rng, rngs::StdRng, SeedableRng},
        },
    },
    rayon::{iter::{IntoParallelIterator, ParallelIterator}, ThreadPoolBuilder},
    std::marker::PhantomData,
};

use crate::broker::Broker;
use crate::exchange::Exchange;
use crate::replay::Replay;
use crate::trader::Trader;

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

impl<B: IntoIterator, E: IntoIterator, T: IntoIterator> ParallelBacktester<B, E, T, StdRng>
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

impl<
    BrokerConfigs: IntoIterator,
    ExchangeConfigs: IntoIterator,
    PerThreadConfigs: IntoIterator,
    RNG: Rng + SeedableRng
>
ParallelBacktester<BrokerConfigs, ExchangeConfigs, PerThreadConfigs, RNG>
{
    pub fn with_num_threads(mut self, num_threads: usize) -> Self {
        self.num_threads = num_threads;
        self
    }
}

impl<
    BrokerID: Id,
    ExchangeID: Id,
    BrokerConfig: Sync,
    ExchangeConfig: Sync,
    BrokerConfigs: IntoIterator<Item=(BrokerConfig, impl IntoIterator<Item=ExchangeID>)>,
    ExchangeConfigs: IntoIterator<Item=ExchangeConfig>,
    TraderConfig: Send,
    ReplayConfig: Send,
    TraderConfigs: IntoIterator<Item=(TraderConfig, impl Send + IntoIterator<Item=(BrokerID, impl IntoIterator<Item=SubCfg>)>)>,
    PerThreadConfigs: IntoIterator<Item=ThreadConfig<ReplayConfig, TraderConfigs>>,
    RNG: Rng + SeedableRng,
    SubCfg,
>
ParallelBacktester<BrokerConfigs, ExchangeConfigs, PerThreadConfigs, RNG>
{
    pub fn run_simulation<T, B, E, R>(self)
        where
            T: for<'a> From<&'a TraderConfig> + Trader<TraderID=B::TraderID, BrokerID=B::BrokerID, T2B=B::T2B, B2T=B::B2T>,
            B: for<'a> From<&'a BrokerConfig> + Broker<BrokerID=E::BrokerID, ExchangeID=E::ExchangeID, B2E=E::B2E, E2B=E::E2B, SubCfg=SubCfg>,
            E: for<'a> From<&'a ExchangeConfig> + Exchange<BrokerID=BrokerID, ExchangeID=R::ExchangeID, E2R=R::E2R, R2E=R::R2E>,
            R: for<'a> From<&'a ReplayConfig> + Replay<ExchangeID=ExchangeID>
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