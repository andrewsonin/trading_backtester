use {
    crate::{
        broker::Broker,
        exchange::Exchange,
        kernel::Kernel,
        replay::Replay,
        traded_pair::TradedPair,
        trader::{subscriptions::SubscriptionList, Trader},
        types::{DateTime, Identifier},
        utils::{ExpectWith, input::config::FromConfig},
    },
    rayon::{iter::{IntoParallelIterator, ParallelIterator}, ThreadPoolBuilder},
};

pub fn parallel_backtest<
    TraderID, BrokerID, ExchangeID, Symbol,
    E, R, B, T,
    ExchangeConfig, ReplayConfig, BrokerConfig, TraderConfig, PerThreadConfig,
    ConnectedExchanges, ConnectedBrokers, SubscriptionConfigs,
>(
    max_num_threads: usize,
    exchange_configs: impl IntoIterator<Item=ExchangeConfig>,
    broker_configs: impl IntoIterator<Item=(BrokerConfig, ConnectedExchanges)>,
    per_thread_configs: impl IntoIterator<
        // (RnG seed, Replay config, Trader Configs with Connected Broker IDs)
        Item=(u64, ReplayConfig, impl IntoIterator<Item=(TraderConfig, ConnectedBrokers)>)
    >,
    date_range: (DateTime, DateTime),
)
    where
        TraderID: Identifier,
        BrokerID: Identifier,
        ExchangeID: Identifier,
        Symbol: Identifier,
        E: Exchange<ExchangeID, BrokerID, Symbol> + FromConfig<ExchangeConfig>,
        R: Replay<ExchangeID, Symbol> + FromConfig<ReplayConfig>,
        B: Broker<BrokerID, TraderID, ExchangeID, Symbol> + FromConfig<BrokerConfig>,
        T: Trader<TraderID, BrokerID, ExchangeID, Symbol> + FromConfig<TraderConfig>,
        ExchangeConfig: Sync,
        BrokerConfig: Sync,
        ReplayConfig: Send,
        TraderConfig: Send,
        ConnectedExchanges: IntoIterator<Item=ExchangeID>,
        ConnectedBrokers: Send + IntoIterator<Item=(BrokerID, SubscriptionConfigs)>,
        SubscriptionConfigs: IntoIterator<Item=(ExchangeID, TradedPair<Symbol>, SubscriptionList)>
{
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
            let exchanges = exchange_configs.iter().map(E::from_config);
            let brokers = broker_configs.iter().map(
                |(broker_cfg, connected_exchanges)|
                    (B::from_config(broker_cfg), connected_exchanges.iter().cloned())
            );
            let traders = trader_configs.into_iter().map(
                |(trader_config, connected_brokers)|
                    (T::from_config(&trader_config), connected_brokers)
            );
            let replay = R::from_config(&replay_config);
            Kernel::new(exchanges, brokers, traders, replay, date_range, rng_seed)
                .run_simulation()
        }
    );
    if max_num_threads != 0 {
        ThreadPoolBuilder::new()
            .num_threads(max_num_threads)
            .build()
            .expect_with(
                || panic!(
                    "Cannot build ThreadPool with the following number of threads to use: {}",
                    max_num_threads
                )
            )
            .install(job)
    } else {
        job()
    }
}