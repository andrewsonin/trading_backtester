use {
    crate::{
        broker::concrete::BasicBroker,
        kernel::Kernel,
        replay::concrete::GetNextObSnapshotDelay,
        traded_pair::{concrete::DefaultTradedPairParser, PairKind, SettleKind, Spot, TradedPair},
        trader::{concrete::SpreadWriter, subscriptions::SubscriptionList},
        types::{DateTime, Identifier, StdRng},
        utils::input::config::from_yaml::parse_yaml,
    },
    std::{num::NonZeroU64, path::Path, str::FromStr},
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

struct DelayScheduler;

impl<ExchangeID: Identifier, Symbol: Identifier>
GetNextObSnapshotDelay<ExchangeID, Symbol> for DelayScheduler
{
    fn get_ob_snapshot_delay(
        &mut self,
        _: ExchangeID,
        _: TradedPair<Symbol>,
        _: &mut StdRng,
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

    let (exchanges, replay, start_dt, end_dt) = parse_yaml(
        test_files.join("example_01.yml"),
        DefaultTradedPairParser,
        DelayScheduler,
    );
    let brokers = [
        (
            BasicBroker::new(BrokerName::Broker1),
            [ExchangeName::MOEX, ExchangeName::NYSE]
        )
    ];

    let simulated_spreads_file_path = test_files.join("example_01").join("simulated_spread.csv");
    let traders = [
        (
            SpreadWriter::new(0, 0.0025, simulated_spreads_file_path),
            [
                (
                    BrokerName::Broker1,
                    [
                        (ExchangeName::MOEX, USD_RUB, SubscriptionList::all())
                    ]
                )
            ]
        )
    ];
    let mut kernel = Kernel::new(
        exchanges, brokers, traders, replay,
        (start_dt, end_dt),
        3344,
    );
    kernel.run_simulation()
}