use {
    crate::{
        broker::concrete::BasicBroker,
        kernel::KernelBuilder,
        parallel::ParallelBacktester,
        replay::concrete::GetNextObSnapshotDelay,
        traded_pair::{concrete::DefaultTradedPairParser, PairKind, SettleKind, Spot, TradedPair},
        trader::{concrete::SpreadWriter, subscriptions::SubscriptionList},
        types::{DateTime, Identifier, PriceStep},
        utils::{
            input::config::{
                from_structs::{
                    BuildExchange,
                    BuildReplay,
                    InitBasicBroker,
                    InitBasicExchange,
                    SpreadWriterConfig,
                },
                from_yaml::parse_yaml,
            },
            rand::{Rng, rngs::StdRng},
        },
    },
    std::{num::NonZeroU64, path::Path, str::FromStr},
};

#[derive(derive_more::Display, Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
enum ExchangeName {
    MOEX,
    NYSE,
}

impl InitBasicExchange for ExchangeName {}

#[derive(derive_more::Display, Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
enum BrokerName {
    Broker1
}

impl InitBasicBroker for BrokerName {}

#[derive(derive_more::Display, Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
enum SymbolName {
    USD,
    RUB,
}

impl FromStr for ExchangeName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MOEX" | "moex" => Ok(ExchangeName::MOEX),
            "NYSE" | "nyse" => Ok(ExchangeName::NYSE),
            _ => Err(())
        }
    }
}

impl FromStr for SymbolName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "USD" | "usd" => Ok(SymbolName::USD),
            "RUB" | "rub" => Ok(SymbolName::RUB),
            _ => Err(())
        }
    }
}

#[derive(Copy, Clone)]
struct DelayScheduler;

impl<ExchangeID: Identifier, Symbol: Identifier>
GetNextObSnapshotDelay<ExchangeID, Symbol> for DelayScheduler
{
    fn get_ob_snapshot_delay(
        &mut self,
        _: ExchangeID,
        _: TradedPair<Symbol>,
        _: &mut impl Rng,
        _: DateTime) -> Option<NonZeroU64>
    {
        Some(NonZeroU64::new(1_000_000_000).unwrap())
    }
}

const USD_RUB: TradedPair<SymbolName> = TradedPair {
    kind: PairKind::Spot(Spot { settlement: SettleKind::Immediately }),
    quoted_symbol: SymbolName::USD,
    base_symbol: SymbolName::RUB,
};

#[test]
fn test_parse_yaml()
{
    let test_files = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let simulated_spreads_file_path = test_files.join("example_01").join("simulated_spread.csv");

    let (exchange_names, replay_config, start_dt, end_dt) = parse_yaml(
        test_files.join("example_01.yml"),
        DefaultTradedPairParser,
        DelayScheduler,
    );

    let exchanges = exchange_names.iter().map(BuildExchange::build);
    let replay = replay_config.build();
    let brokers = [
        (
            BasicBroker::new(BrokerName::Broker1),
            [ExchangeName::MOEX, ExchangeName::NYSE]
        )
    ];
    let traders = [
        (
            SpreadWriter::new(0, 0.0025, simulated_spreads_file_path),
            [
                (
                    BrokerName::Broker1,
                    [
                        (
                            ExchangeName::MOEX,
                            USD_RUB,
                            SubscriptionList::subscribe().to_ob_snapshots()
                        )
                    ]
                )
            ]
        )
    ];
    KernelBuilder::new(exchanges, brokers, traders, replay, (start_dt, end_dt))
        .with_seed(3344)
        .with_rng::<StdRng>()
        .build()
        .run_simulation()
}

#[test]
fn test_parse_yaml_in_parallel()
{
    let test_files = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

    let (exchange_names, replay_config, start_dt, end_dt) = parse_yaml(
        test_files.join("example_01.yml"),
        DefaultTradedPairParser,
        DelayScheduler,
    );
    let broker_configs = [
        (
            BrokerName::Broker1,
            [ExchangeName::MOEX]
        )
    ];

    let trader_subscriptions = [
        (
            BrokerName::Broker1,
            [
                (
                    ExchangeName::MOEX,
                    USD_RUB,
                    SubscriptionList::subscribe().to_ob_snapshots()
                )
            ]
        )
    ];
    let first_thread_configs = (
        42,
        replay_config.clone(),
        [
            (
                SpreadWriterConfig {
                    name: 0,
                    file: test_files.join("example_01").join("simulated_spread_par_01.csv"),
                    price_step: PriceStep(0.0025),
                },
                trader_subscriptions
            )
        ]
    );
    let second_thread_configs = (
        4122,
        replay_config,
        [
            (
                SpreadWriterConfig {
                    name: 1,
                    file: test_files.join("example_01").join("simulated_spread_par_02.csv"),
                    price_step: PriceStep(0.0025),
                },
                trader_subscriptions
            )
        ]
    );
    let per_thread_configs = [
        first_thread_configs.clone(),
        second_thread_configs.clone()
    ];

    ParallelBacktester::new(
        exchange_names.clone(),
        broker_configs,
        per_thread_configs,
        (start_dt, end_dt),
    )
        .run_simulation();

    let per_thread_configs = [
        first_thread_configs.clone(),
        second_thread_configs.clone(),
        second_thread_configs
    ];

    ParallelBacktester::new(
        exchange_names,
        broker_configs,
        per_thread_configs,
        (start_dt, end_dt),
    )
        .with_rng::<StdRng>()
        .with_num_threads(2)
        .run_simulation()
}