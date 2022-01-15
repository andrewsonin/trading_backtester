use {
    crate::{
        settlement::concrete::VoidSettlement,
        traded_pair::{PairKind, TradedPair, TradedPairParser},
        types::Identifier,
        utils::ExpectWith,
    },
    std::str::FromStr,
};

pub struct SpotTradedPairParser;

impl<Symbol: Identifier + FromStr> TradedPairParser<Symbol, VoidSettlement>
for SpotTradedPairParser
{
    fn parse<ExchangeID: Identifier>(
        _: ExchangeID,
        kind: impl AsRef<str>,
        quoted_symbol: impl AsRef<str>,
        base_symbol: impl AsRef<str>) -> TradedPair<Symbol, VoidSettlement>
    {
        let (kind, quoted_symbol, base_symbol) = (
            kind.as_ref(), quoted_symbol.as_ref(), base_symbol.as_ref()
        );
        let quoted_symbol = FromStr::from_str(quoted_symbol).expect_with(
            || panic!("Cannot parse {quoted_symbol} to Symbol")
        );
        let base_symbol = FromStr::from_str(base_symbol).expect_with(
            || panic!("Cannot parse {base_symbol} to Symbol")
        );
        let kind = if let "Spot" | "spot" = kind {
            PairKind::Spot
        } else {
            panic!("Cannot parse to PairKind: {kind}")
        };
        TradedPair {
            kind,
            quoted_symbol,
            base_symbol,
        }
    }
}