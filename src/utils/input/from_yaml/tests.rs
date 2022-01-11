use {
    crate::{
        broker::concrete::BasicBroker,
        exchange::reply::ObSnapshot,
        kernel::Kernel,
        replay::concrete::GetNextObSnapshotDelay,
        traded_pair::{concrete::DefaultTradedPairParser, PairKind, SettleKind, Spot, TradedPair},
        trader::{concrete::VoidTrader, subscriptions::SubscriptionList},
        types::{DateTime, Identifier, Size, StdRng},
        utils::{ExpectWith, input::from_yaml::parse},
    },
    std::{fs::File, io::Write, num::NonZeroU64, path::Path, str::FromStr},
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
GetNextObSnapshotDelay<ExchangeID, Symbol> for DelayScheduler {
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

    let simulated_spreads_file_path = test_files.join("example_01").join("simulated_spread.csv");

    let mut simulated_spreads_file = File::create(&simulated_spreads_file_path)
        .expect_with(|| panic!("Cannot create file {:?}", simulated_spreads_file_path));

    let cannot_write_to_file = || panic!("Cannot write to file {:?}", simulated_spreads_file_path);
    writeln!(
        simulated_spreads_file,
        "Timestamp,BID_PRICE,BID_SIZE,ASK_PRICE,ASK_SIZE"
    ).expect_with(cannot_write_to_file);

    let price_step = 0.0025.into();
    let get_size = |(size, _dt): &_| *size;

    let ob_snapshot_inspector = |current_dt: DateTime, _e_id, snapshot: &ObSnapshot<SymbolName>|
        {
            if let (Some((bid, bids)), Some((ask, asks))) = (
                snapshot.state.bids.first(),
                snapshot.state.asks.first()
            ) {
                let bid_size: Size = bids.iter().map(get_size).sum();
                let ask_size: Size = asks.iter().map(get_size).sum();
                let bid_price = bid.to_f64(price_step);
                let ask_price = ask.to_f64(price_step);
                if bid_price >= ask_price {
                    panic!(
                        "Bid price should be lower than Ask price. Got: {:.4} {:.4}",
                        bid_price, ask_price
                    )
                }
                writeln!(
                    simulated_spreads_file,
                    "{},{:.4},{},{:.4},{}",
                    current_dt,
                    bid_price,
                    bid_size,
                    ask_price,
                    ask_size
                ).expect_with(cannot_write_to_file)
            }
        };

    let (exchanges, replay, start_dt, end_dt) = parse(
        DelayScheduler,
        DefaultTradedPairParser,
        test_files.join("example_01.yml"),
        ob_snapshot_inspector,
    );
    let brokers = [
        (
            BasicBroker::new(BrokerName::Broker1),
            [ExchangeName::MOEX, ExchangeName::NYSE]
        )
    ];
    let traders = [
        (
            VoidTrader::new(0),
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
    );
    kernel.run_simulation()
}