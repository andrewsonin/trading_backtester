use {
    crate::{
        traded_pair::{PairKind, SettleKind, Spot, TradedPair, TradedPairParser},
        types::Identifier,
        utils::ExpectWith,
    },
    std::str::FromStr,
};

pub struct DefaultTradedPairParser;

impl<Symbol: Identifier + FromStr> TradedPairParser<Symbol> for DefaultTradedPairParser
{
    fn parse(
        kind: impl AsRef<str>,
        quoted_symbol: impl AsRef<str>,
        base_symbol: impl AsRef<str>) -> TradedPair<Symbol>
    {
        let (kind, quoted_symbol, base_symbol) = (
            kind.as_ref(), quoted_symbol.as_ref(), base_symbol.as_ref()
        );
        let quoted_symbol = FromStr::from_str(quoted_symbol).expect_with(
            || panic!("Cannot parse {} to Symbol", quoted_symbol)
        );
        let base_symbol = FromStr::from_str(base_symbol).expect_with(
            || panic!("Cannot parse {} to Symbol", base_symbol)
        );
        let kind = if let "Spot" | "spot" = kind {
            PairKind::Spot(Spot { settlement: SettleKind::Immediately })
        } else {
            panic!("Cannot parse to PairKind: {}", kind)
        };
        TradedPair {
            kind,
            quoted_symbol,
            base_symbol,
        }
    }
}