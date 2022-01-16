pub mod broker;
pub mod exchange;
pub mod kernel;
pub mod order;
pub mod order_book;
pub mod parallel;
pub mod replay;
pub mod settlement;
pub mod traded_pair;
pub mod trader;
pub mod types;
pub mod utils;

pub mod prelude {
    pub use crate::{
        broker::{
            Broker,
            BrokerAction,
            BrokerActionKind,
            concrete as broker_examples,
            reply as broker_reply,
            request as broker_request,
        },
        exchange::{
            concrete as exchange_example,
            Exchange,
            ExchangeAction,
            ExchangeActionKind,
            reply as exchange_reply,
        },
        kernel::{Kernel, KernelBuilder},
        order::{LimitOrderCancelRequest, LimitOrderPlacingRequest, MarketOrderPlacingRequest},
        order_book::{LimitOrder, OrderBook, OrderBookEvent, OrderBookEventKind},
        parallel::{ParallelBacktester, ThreadConfig},
        replay::{
            concrete as replay_examples,
            Replay,
            ReplayAction,
            request as replay_request,
        },
        settlement::{concrete as settlement_examples, GetSettlementLag},
        traded_pair::{
            Futures,
            Option,
            OptionKind,
            PairKind,
            parser::{concrete as traded_pair_parser_examples, TradedPairParser},
            TradedPair,
        },
        trader::{
            concrete as trader_examples,
            request as trader_request,
            subscriptions::{Subscription, SubscriptionConfig, SubscriptionList},
            Trader,
            TraderAction,
            TraderActionKind,
        },
        types::*,
        utils::{
            constants,
            enum_dispatch,
            ExpectWith,
            input::{
                config::{from_structs::*, from_yaml::parse_yaml},
                one_tick::OneTickTradedPairReader,
            },
            parse_datetime,
            queue::{LessElementBinaryHeap, MessagePusher},
            rand,
        },
    };
}

#[cfg(test)]
mod tests {
    use {
        crate::{
            broker::concrete::BasicBroker,
            kernel::KernelBuilder,
            parallel::{ParallelBacktester, ThreadConfig},
            replay::concrete::GetNextObSnapshotDelay,
            settlement::{concrete::VoidSettlement, GetSettlementLag},
            traded_pair::{PairKind, parser::concrete::SpotTradedPairParser, TradedPair},
            trader::{concrete::SpreadWriter, subscriptions::{SubscriptionConfig, SubscriptionList}},
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

    impl<ExchangeID: Identifier, Symbol: Identifier, Settlement: GetSettlementLag>
    GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement> for DelayScheduler
    {
        fn get_ob_snapshot_delay(
            &mut self,
            _: ExchangeID,
            _: TradedPair<Symbol, Settlement>,
            _: &mut impl Rng,
            _: DateTime) -> Option<NonZeroU64>
        {
            Some(NonZeroU64::new(1_000_000_000).unwrap())
        }
    }

    const USD_RUB: TradedPair<SymbolName, VoidSettlement> = TradedPair {
        kind: PairKind::Spot,
        quoted_symbol: SymbolName::USD,
        base_symbol: SymbolName::RUB,
    };

    #[test]
    fn test_parse_yaml()
    {
        let test_files = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
        let simulated_spreads_file_path = test_files
            .join("example_01")
            .join("simulated_spread.csv");

        let (exchange_names, replay_config, start_dt, end_dt) = parse_yaml(
            test_files.join("example_01.yml"),
            SpotTradedPairParser,
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
        let subscription_config = SubscriptionConfig::new(
            ExchangeName::MOEX,
            USD_RUB,
            SubscriptionList::subscribe().to_ob_snapshots(),
        );
        let traders = [
            (
                SpreadWriter::new(0, 0.0025, simulated_spreads_file_path),
                [
                    (BrokerName::Broker1, [subscription_config])
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
            SpotTradedPairParser,
            DelayScheduler,
        );
        let broker_configs = [
            (
                BrokerName::Broker1,
                [ExchangeName::MOEX]
            )
        ];

        let subscription_config = SubscriptionConfig::new(
            ExchangeName::MOEX,
            USD_RUB,
            SubscriptionList::subscribe().to_ob_snapshots(),
        );
        let trader_subscriptions = [
            (BrokerName::Broker1, [subscription_config])
        ];
        let first_thread_config = ThreadConfig::new(
            42,
            replay_config.clone(),
            [
                (
                    SpreadWriterConfig::new(
                        0,
                        test_files.join("example_01").join("simulated_spread_par_01.csv"),
                        PriceStep(0.0025),
                    ),
                    trader_subscriptions
                )
            ],
        );
        let second_thread_config = ThreadConfig::new(
            4122,
            replay_config.clone(),
            [
                (
                    SpreadWriterConfig::new(
                        1,
                        test_files.join("example_01").join("simulated_spread_par_02.csv"),
                        PriceStep(0.0025),
                    ),
                    trader_subscriptions
                )
            ],
        );
        let per_thread_configs = [
            first_thread_config.clone(),
            second_thread_config.clone()
        ];

        ParallelBacktester::new(
            exchange_names.clone(),
            broker_configs,
            per_thread_configs,
            (start_dt, end_dt),
        )
            .run_simulation();

        let per_thread_configs = [
            first_thread_config.clone(),
            second_thread_config.clone(),
            second_thread_config
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
}