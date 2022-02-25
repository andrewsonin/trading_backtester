//! # Trading backtester
//! _A highly customizable framework designed for parallel tuning of trading algorithms
//! by reproducing and simulating the trading history of exchanges and the behaviour of brokers._
//!
//! It aims to provide a complete set of interfaces needed to simulate and optimize trading and
//! investing algorithms at any level of precision: from the level of intra-exchange messages
//! to the level of price chart trading.
//!
//! This project is committed to achieving:
//!
//! * __The highest possible execution speed__,
//! which is available through AOT compilation that runs directly to native code, low runtime,
//! and great `rustc` and `LLVM` optimization abilities.
//!
//! * __Low probability of making critical errors__,
//! which is achieved by Rust's strong type system and borrowing rules
//! that prevent the vast majority of erroneous programs from compiling.
//!
//! * __High speed of writing custom code.__ This goal is achieved through
//! the relatively simple syntax of the Rust language,
//! which makes it no more complicated than that of C#.
//!
//! ## Usage
//!
//! Put this in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! trading_backtester = { path = "???", features = ["???"] }
//! ```
//! where `path` should point to the location of the `trading_backtester` library,
//! and `features` should consist of the available ones (or may not be set).
//!
//! ## Features
//!
//! The following features are available for enabling. Each of them provides access to:
//!
//! * __`concrete`__
//!
//!   Concrete examples of entities that implement traits from the `interface` module.
//!
//! * __`enum_def`__
//!
//!   The macro that generates an `enum` that can contain each
//!   of the listed types as a unique `enum`
//!   variant. Simplifies the creation of statically dispatched trait objects.
//!
//! * __`enum_dispatch`__
//!
//!   Derive macros for statically dispatched trait objects from the `interface` module.
//!   Convenient to use with the `enum_def`.
//!
//! * __`multithread`__
//!
//!   Utilities for running backtesters in multiple threads.

#[cfg(feature = "concrete")]
/// Concrete examples of entities that implement traits from the [`interface`] module.
pub mod concrete;

/// Abstract interfaces.
pub mod interface;

/// Kernel of the backtester.
pub mod kernel;

#[cfg(feature = "multithread")]
/// Utilities for running backtesters in multiple threads.
pub mod parallel;

/// Auxiliary types and traits.
pub mod types;

/// Other auxiliary utilities.
pub mod utils;

/// The Rust Prelude
pub mod prelude {
    pub use crate::{
        interface::{broker::*, exchange::*, latency::*, message::*, replay::*, trader::*},
        kernel::{Kernel, KernelBuilder, LatentActionProcessor},
        types::*,
        utils::{
            chrono,
            constants,
            queue::{LessElementBinaryHeap, MessageReceiver},
            rand,
        },
    };
    #[cfg(feature = "concrete")]
    pub use crate::concrete::{
        broker as broker_examples,
        exchange as exchange_example,
        input::{
            config::{from_structs::*, from_yaml::*},
            one_tick::OneTickTradedPairReader,
        },
        latency as latency_examples,
        message_protocol::{
            broker::{reply as broker_reply, request as broker_request},
            exchange::reply as exchange_reply,
            replay::request as replay_request,
            trader::request as trader_request,
        },
        order::{
            LimitOrderCancelRequest,
            LimitOrderPlacingRequest,
            MarketOrderPlacingRequest,
        },
        order_book::{LimitOrder, OrderBook, OrderBookEvent, OrderBookEventKind},
        replay as replay_examples,
        traded_pair::{
            Asset,
            Base,
            Futures,
            OptionContract,
            OptionKind,
            parser::{concrete as traded_pair_parser_examples, TradedPairParser},
            settlement::{concrete as settlement_examples, GetSettlementLag},
            TradedPair,
        },
        trader as trader_examples,
        trader::subscriptions::{SubscriptionConfig, SubscriptionList},
        types as misc_types,
    };
    #[cfg(feature = "enum_def")]
    pub use crate::enum_def;
    #[cfg(feature = "multithread")]
    pub use crate::parallel::{ParallelBacktester, ThreadConfig};
    #[cfg(feature = "derive")]
    pub use crate::utils::derive;
    #[cfg(feature = "derive_more")]
    pub use crate::utils::derive_more;
}

#[cfg(feature = "concrete")]
#[cfg(test)]
mod tests {
    use {
        broker_examples::BasicBroker,
        crate::prelude::*,
        exchange_example::BasicExchange,
        misc_types::PriceStep,
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

    #[derive(derive_more::Display, Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
    enum BrokerName {
        Broker1
    }

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
            _: DateTime) -> Option<(NonZeroU64, usize)>
        {
            Some((NonZeroU64::new(1_000_000_000).unwrap(), 1))
        }
    }

    #[test]
    fn test_parse_yaml()
    {
        let usd_rub = TradedPair {
            quoted_asset: Base::new(SymbolName::USD).into(),
            settlement_asset: Base::new(SymbolName::RUB).into(),
            settlement_determinant: SpotSettlement,
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
    fn test_parse_yaml_2()
    {
        let usd_rub = TradedPair {
            quoted_asset: Base::new(SymbolName::USD).into(),
            settlement_asset: Base::new(SymbolName::RUB).into(),
            settlement_determinant: SpotSettlement,
        };

        let test_files = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
        let simulated_spreads_file_path = test_files
            .join("example_02")
            .join("simulated_spread.csv");

        let (exchange_names, replay_config, start_dt, end_dt) = parse_yaml(
            test_files.join("example_02.yml"),
            SpotBaseTradedPairParser,
            DelayScheduler,
        );

        let exchanges = exchange_names.iter().map(BasicExchange::from);
        let replay = OneTickReplay::from(&replay_config);
        let brokers = [
            (
                BasicBroker::new(BrokerName::Broker1),
                [ExchangeName::MOEX]
            )
        ];
        let subscription_config = SubscriptionConfig::new(
            ExchangeName::MOEX,
            usd_rub,
            SubscriptionList::subscribe(),
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

    #[cfg(feature = "multithread")]
    #[test]
    fn test_parse_yaml_in_parallel()
    {
        let usd_rub = TradedPair {
            quoted_asset: Base::new(SymbolName::USD).into(),
            settlement_asset: Base::new(SymbolName::RUB).into(),
            settlement_determinant: SpotSettlement,
        };

        let test_files = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

        let (exchange_names, replay_config, start_dt, end_dt) = parse_yaml(
            test_files.join("example_01.yml"),
            SpotBaseTradedPairParser,
            DelayScheduler,
        );

        let broker_name = BrokerName::Broker1;
        let broker_configs = [
            (&broker_name, [ExchangeName::MOEX])
        ];

        let spread_writer_config = SpreadWriterConfig::new(
            0,
            test_files.join("example_01").join("simulated_spread_par_01.csv"),
            PriceStep(0.0025),
        );
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
            &replay_config,
            &exchange_names,
            broker_configs,
            [
                (&spread_writer_config, trader_subscriptions)
            ],
        );

        let spread_writer_config = SpreadWriterConfig::new(
            1,
            test_files.join("example_01").join("simulated_spread_par_02.csv"),
            PriceStep(0.0025),
        );
        let second_thread_config = ThreadConfig::new(
            4122,
            &replay_config,
            &exchange_names,
            broker_configs,
            [
                (&spread_writer_config, trader_subscriptions)
            ],
        );
        let per_thread_configs = [
            first_thread_config,
            second_thread_config
        ];

        type Trader = SpreadWriter<u8, BrokerName, ExchangeName, SymbolName, SpotSettlement>;
        type Broker = BasicBroker<BrokerName, u8, ExchangeName, SymbolName, SpotSettlement>;
        type Exchange = BasicExchange<ExchangeName, BrokerName, SymbolName, SpotSettlement>;
        type Replay = OneTickReplay<
            BrokerName, ExchangeName, SymbolName, DelayScheduler, SpotSettlement
        >;

        ParallelBacktester::new(
            per_thread_configs,
            (start_dt, end_dt),
        )
            .run_simulation::<Trader, Broker, Exchange, Replay>();

        let per_thread_configs = [
            first_thread_config,
            second_thread_config,
            second_thread_config
        ];

        ParallelBacktester::new(
            per_thread_configs,
            (start_dt, end_dt),
        )
            .with_rng::<StdRng>()
            .with_num_threads(2)
            .run_simulation::<Trader, Broker, Exchange, Replay>()
    }

    #[cfg(feature = "derive")]
    #[allow(dead_code)]
    mod test_enum_def {
        use {
            broker_examples::{BasicBroker, BasicVoidBroker},
            crate::prelude::*,
            derive::{Broker, Exchange, GetSettlementLag, LatencyGenerator, Replay, Trader},
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
            ReplayEnum<BrokerID: Id, ExchangeID: Id, Symbol: Id, ObSnapshotDelay, Settlement>
                where ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>,
                      Settlement: GetSettlementLag
            {
                OneTickReplay<BrokerID, ExchangeID, Symbol, ObSnapshotDelay, Settlement>,
                BasicVoidReplay<BrokerID, ExchangeID, Symbol, Settlement>
            }
        }

        #[derive(Replay)]
        enum AnotherReplayEnum<
            BrokerID: Id,
            ExchangeID: Id,
            Symbol: Id,
            ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>,
            Settlement: GetSettlementLag
        > {
            Var1(OneTickReplay<BrokerID, ExchangeID, Symbol, ObSnapshotDelay, Settlement>),
            Var2(BasicVoidReplay<BrokerID, ExchangeID, Symbol, Settlement>),
        }

        type ZeroLatency<OuterID> = ConstantLatency<OuterID, 0, 0>;
        type OneNSLatency<OuterID> = ConstantLatency<OuterID, 1, 1>;

        enum_def! {
            #[derive(LatencyGenerator, Copy, Clone)]
            LatencyGenEnum<OuterID: Id> {
                ZeroLatency<OuterID>,
                OneNSLatency<OuterID>
            }
        }

        #[derive(LatencyGenerator, Copy, Clone)]
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