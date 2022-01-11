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
    fn parse(kind: &str, quoted_symbol: &str, base_symbol: &str) -> TradedPair<Symbol>
    {
        let quoted_symbol = FromStr::from_str(quoted_symbol).expect_with(
            || panic!("Cannot parse {} to Symbol", quoted_symbol)
        );
        let base_symbol = FromStr::from_str(base_symbol).expect_with(
            || panic!("Cannot parse {} to Symbol", base_symbol)
        );
        let kind = match kind {
            "Spot" | "spot" => PairKind::Spot(Spot { settlement: SettleKind::Immediately }),
            _ => panic!("Cannot parse to PairKind: {}", kind)
        };
        TradedPair {
            kind,
            quoted_symbol,
            base_symbol,
        }
    }
}