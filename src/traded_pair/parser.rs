use {
    crate::{
        settlement::GetSettlementLag,
        traded_pair::TradedPair,
        types::Id,
    },
    std::str::FromStr,
};

pub trait TradedPairParser<
    Symbol: Id + FromStr,
    Settlement: GetSettlementLag
> {
    fn parse<ExchangeID: Id>(
        exchange_id: ExchangeID,
        kind: impl AsRef<str>,
        quoted_symbol: impl AsRef<str>,
        base_symbol: impl AsRef<str>) -> TradedPair<Symbol, Settlement>;
}

pub mod concrete {
    use {
        crate::{
            settlement::concrete::SpotSettlement,
            traded_pair::{Asset, parser::TradedPairParser, TradedPair},
            traded_pair::Base,
            types::Id,
        },
        std::str::FromStr,
    };

    pub struct SpotBaseTradedPairParser;

    impl<Symbol: Id + FromStr> TradedPairParser<Symbol, SpotSettlement>
    for SpotBaseTradedPairParser
    {
        fn parse<ExchangeID: Id>(
            _: ExchangeID,
            kind: impl AsRef<str>,
            quoted_symbol: impl AsRef<str>,
            base_symbol: impl AsRef<str>) -> TradedPair<Symbol, SpotSettlement>
        {
            let (kind, quoted_symbol, base_symbol) = (
                kind.as_ref(), quoted_symbol.as_ref(), base_symbol.as_ref()
            );

            let quoted_symbol = FromStr::from_str(quoted_symbol).unwrap_or_else(
                |_| panic!("Cannot parse {quoted_symbol} to Symbol")
            );
            const PATTERN: &str = "base :: spot";
            let quoted_symbol = if let PATTERN = kind.to_lowercase().as_str() {
                Asset::Base(Base::new(quoted_symbol))
            } else {
                panic!(
                    "Cannot parse to TradedPair<Symbol, SpotSettlement>: \"{kind}\". \
                    Expected: \"{PATTERN}\""
                )
            };

            let base_symbol = FromStr::from_str(base_symbol).unwrap_or_else(
                |_| panic!("Cannot parse {base_symbol} to Symbol")
            );
            let base_symbol = Base::new(base_symbol);
            TradedPair {
                quoted_asset: quoted_symbol,
                base_asset: base_symbol,
                settlement: SpotSettlement,
            }
        }
    }
}