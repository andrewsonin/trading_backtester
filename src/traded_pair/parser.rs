use {
    crate::{
        settlement::GetSettlementLag,
        traded_pair::TradedPair,
        types::Identifier,
        utils::enum_dispatch,
    },
    std::str::FromStr,
};

#[enum_dispatch]
pub trait TradedPairParser<
    Symbol: Identifier + FromStr,
    Settlement: GetSettlementLag
> {
    fn parse<ExchangeID: Identifier>(
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
            types::Identifier,
            utils::ExpectWith,
        },
        std::str::FromStr,
    };

    use crate::prelude::Base;

    pub struct SpotBaseTradedPairParser;

    impl<Symbol: Identifier + FromStr> TradedPairParser<Symbol, SpotSettlement>
    for SpotBaseTradedPairParser
    {
        fn parse<ExchangeID: Identifier>(
            _: ExchangeID,
            kind: impl AsRef<str>,
            quoted_symbol: impl AsRef<str>,
            base_symbol: impl AsRef<str>) -> TradedPair<Symbol, SpotSettlement>
        {
            let (kind, quoted_symbol, base_symbol) = (
                kind.as_ref(), quoted_symbol.as_ref(), base_symbol.as_ref()
            );

            let quoted_symbol = FromStr::from_str(quoted_symbol).expect_with(
                || panic!("Cannot parse {quoted_symbol} to Symbol")
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

            let base_symbol = FromStr::from_str(base_symbol).expect_with(
                || panic!("Cannot parse {base_symbol} to Symbol")
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