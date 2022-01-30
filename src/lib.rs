pub mod broker;
pub mod exchange;
pub mod kernel;
pub mod latency;
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
            BrokerToExchange,
            BrokerToItself,
            BrokerToTrader,
            concrete as broker_examples,
            reply as broker_reply,
            request as broker_request,
        },
        enum_def,
        exchange::{
            concrete as exchange_example,
            Exchange,
            ExchangeAction,
            ExchangeActionKind,
            ExchangeToBroker,
            ExchangeToItself,
            ExchangeToReplay,
            reply as exchange_reply,
        },
        kernel::{Kernel, KernelBuilder, LatentActionProcessor},
        latency::{concrete as latency_examples, LatencyGenerator, Latent},
        order::{LimitOrderCancelRequest, LimitOrderPlacingRequest, MarketOrderPlacingRequest},
        order_book::{LimitOrder, OrderBook, OrderBookEvent, OrderBookEventKind},
        parallel::{ParallelBacktester, ThreadConfig},
        replay::{
            concrete as replay_examples,
            Replay,
            ReplayAction,
            ReplayToExchange,
            ReplayToItself,
            request as replay_request,
        },
        settlement::{concrete as settlement_examples, GetSettlementLag},
        traded_pair::{
            Asset,
            Base,
            Futures,
            OptionContract,
            OptionKind,
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
            TraderToBroker,
            TraderToItself,
        },
        types::*,
        utils::{
            chrono,
            constants,
            derive_macros,
            derive_more,
            input::{
                config::{from_structs::*, from_yaml::parse_yaml},
                one_tick::OneTickTradedPairReader,
            },
            parse_datetime,
            queue::{LessElementBinaryHeap, MessageReceiver},
            rand,
        },
    };
}

#[cfg(test)]
mod tests {
    use {
        broker_examples::BasicBroker,
        crate::prelude::*,
        exchange_example::BasicExchange,
        rand::{Rng, rngs::StdRng},
        replay_examples::{GetNextObSnapshotDelay, OneTickReplay},
        settlement_examples::SpotSettlement,
        std::{num::NonZeroU64, path::Path, str::FromStr},
        traded_pair_parser_examples::SpotBaseTradedPairParser,
        trader_examples::SpreadWriter,
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

    impl<ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
    GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>
    for DelayScheduler
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

    #[test]
    fn test_parse_yaml()
    {
        let usd_rub = TradedPair {
            quoted_asset: Asset::Base(Base::new(SymbolName::USD)),
            base_asset: Base::new(SymbolName::RUB),
            settlement: SpotSettlement,
        };

        let test_files = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
        let simulated_spreads_file_path = test_files
            .join("example_01")
            .join("simulated_spread.csv");

        let (exchange_names, replay_config, start_dt, end_dt) = parse_yaml(
            test_files.join("example_01.yml"),
            SpotBaseTradedPairParser,
            DelayScheduler,
        );

        let exchanges = exchange_names.iter().map(BasicExchange::from);
        let replay = OneTickReplay::from(&replay_config);
        let brokers = [
            (
                BasicBroker::new(BrokerName::Broker1),
                [ExchangeName::MOEX, ExchangeName::NYSE]
            )
        ];
        let subscription_config = SubscriptionConfig::new(
            ExchangeName::MOEX,
            usd_rub,
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
        let usd_rub = TradedPair {
            quoted_asset: Asset::Base(Base::new(SymbolName::USD)),
            base_asset: Base::new(SymbolName::RUB),
            settlement: SpotSettlement,
        };

        let test_files = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

        let (exchange_names, replay_config, start_dt, end_dt) = parse_yaml(
            test_files.join("example_01.yml"),
            SpotBaseTradedPairParser,
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
            usd_rub,
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

        type Trader = SpreadWriter<u8, BrokerName, ExchangeName, SymbolName, SpotSettlement>;
        type Broker = BasicBroker<BrokerName, u8, ExchangeName, SymbolName, SpotSettlement>;
        type Exchange = BasicExchange<ExchangeName, BrokerName, SymbolName, SpotSettlement>;
        type Replay = OneTickReplay<ExchangeName, SymbolName, DelayScheduler, SpotSettlement>;

        ParallelBacktester::new(
            exchange_names.clone(),
            broker_configs,
            per_thread_configs,
            (start_dt, end_dt),
        )
            .run_simulation::<Trader, Broker, Exchange, Replay>();

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
            .run_simulation::<Trader, Broker, Exchange, Replay>()
    }

    #[allow(dead_code)]
    mod test_enum_def {
        use {
            broker_examples::{BasicBroker, BasicVoidBroker},
            crate::prelude::*,
            derive_macros::{Broker, Exchange, GetSettlementLag, LatencyGenerator, Replay, Trader},
            exchange_example::{BasicExchange, BasicVoidExchange},
            latency_examples::ConstantLatency,
            rand::Rng,
            replay_examples::{BasicVoidReplay, GetNextObSnapshotDelay, OneTickReplay},
            settlement_examples::{SpotSettlement, VoidSettlement},
            trader_examples::{BasicVoidTrader, SpreadWriter},
        };

        enum_def! {
            #[derive(Trader)]
            TraderEnum<
                TraderID: Id, BrokerID: Id, ExchangeID: Id, Symbol: Id,
                Settlement: GetSettlementLag
            > {
                SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>,
                BasicVoidTrader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
            }
        }

        #[derive(Trader)]
        enum AnotherTraderEnum<
            TraderID: Id, BrokerID: Id, ExchangeID: Id, Symbol: Id,
            Settlement: GetSettlementLag
        > {
            Var1(SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>),
            Var2(BasicVoidTrader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>),
        }

        enum_def! {
            #[derive(Broker)]
            BrokerEnum<
                BrokerID, TraderID: Id, ExchangeID: Id, Symbol: Id,
                Settlement: GetSettlementLag
            >
            where BrokerID: Id
            {
                BasicBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>,
                BasicVoidBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>
            }
        }

        #[derive(Broker)]
        enum AnotherBrokerEnum<
            BrokerID: Id, TraderID: Id, ExchangeID: Id, Symbol: Id,
            Settlement: GetSettlementLag
        > {
            Var1(BasicBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>),
            Var2(BasicVoidBroker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>),
        }

        enum_def! {
            #[derive(Exchange)]
            ExchangeEnum<ExchangeID: Id, BrokerID: Id, Symbol: Id, Settlement: GetSettlementLag>
            {
                BasicExchange<ExchangeID, BrokerID, Symbol, Settlement>,
                BasicVoidExchange<ExchangeID, BrokerID, Symbol, Settlement>
            }
        }

        #[derive(Exchange)]
        enum AnotherExchangeEnum<
            ExchangeID: Id,
            BrokerID: Id,
            Symbol: Id,
            Settlement: GetSettlementLag
        > {
            Var1(BasicExchange<ExchangeID, BrokerID, Symbol, Settlement>),
            Var2(BasicVoidExchange<ExchangeID, BrokerID, Symbol, Settlement>),
        }

        enum_def! {
            #[derive(Replay)]
            ReplayEnum<ExchangeID: Id, Symbol: Id, ObSnapshotDelay, Settlement: GetSettlementLag>
                where ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>
            {
                OneTickReplay<ExchangeID, Symbol, ObSnapshotDelay, Settlement>,
                BasicVoidReplay<ExchangeID, Symbol, Settlement>
            }
        }

        #[derive(Replay)]
        enum AnotherReplayEnum<
            ExchangeID: Id,
            Symbol: Id,
            ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>,
            Settlement: GetSettlementLag
        > {
            Var1(OneTickReplay<ExchangeID, Symbol, ObSnapshotDelay, Settlement>),
            Var2(BasicVoidReplay<ExchangeID, Symbol, Settlement>),
        }

        type ZeroLatency<OuterID> = ConstantLatency<OuterID, 0, 0>;
        type OneNSLatency<OuterID> = ConstantLatency<OuterID, 1, 1>;

        enum_def! {
            #[derive(LatencyGenerator)]
            LatencyGenEnum<OuterID: Id> {
                ZeroLatency<OuterID>,
                OneNSLatency<OuterID>
            }
        }

        #[derive(LatencyGenerator)]
        enum AnotherLatencyGenEnum<OuterID: Id> {
            Var1(ConstantLatency<OuterID, 0, 0>),
            Var2(ConstantLatency<OuterID, 1, 0>),
        }

        enum_def! {
            #[derive(GetSettlementLag, Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy)]
            SettlementEnum {
                VoidSettlement,
                SpotSettlement
            }
        }

        #[derive(GetSettlementLag, Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy)]
        enum AnotherSettlementEnum {
            Var1(VoidSettlement),
            Var2(SpotSettlement),
        }
    }
}