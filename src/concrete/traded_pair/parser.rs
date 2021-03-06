use {
    crate::{
        concrete::traded_pair::{settlement::GetSettlementLag, TradedPair},
        types::Id,
    },
    std::str::FromStr,
};

/// Parsers that can interpret [`TradedPair`] from the input arguments.
pub trait TradedPairParser<
    Symbol: Id + FromStr,
    Settlement: GetSettlementLag
> {
    /// Parses [`TradedPair`] from the input parameters.
    ///
    /// # Arguments
    /// * `exchange_id` — Exchange ID.
    /// * `kind` — kind of traded pair written in a string format.
    /// * `quoted_symbol` — kind of quoted symbol written in a string format.
    /// * `base_symbol` — kind of base symbol written in a string format.
    fn parse<ExchangeID: Id>(
        exchange_id: ExchangeID,
        kind: impl AsRef<str>,
        quoted_symbol: impl AsRef<str>,
        base_symbol: impl AsRef<str>) -> TradedPair<Symbol, Settlement>;
}

pub mod concrete {
    use {
        crate::{
            concrete::traded_pair::{
                Base,
                parser::TradedPairParser,
                settlement::concrete::SpotSettlement,
                TradedPair,
            },
            types::Id,
        },
        std::{fmt::Debug, str::FromStr},
    };

    /// Parser that can only parse spot base traded pairs written in a specific format.
    /// See the concrete implementation of the `<Self as TradedPairParser>::parse`
    /// for clarification.
    pub struct SpotBaseTradedPairParser;

    impl<Symbol: Id + FromStr<Err=Err>, Err: Debug>
    TradedPairParser<Symbol, SpotSettlement>
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
                |err| panic!("Cannot parse {quoted_symbol} to Symbol. Error: {err:?}")
            );
            const PATTERN: &str = "base :: spot";
            let quoted_asset = if let PATTERN = kind.to_lowercase().as_str() {
                Base::new(quoted_symbol).into()
            } else {
                panic!(
                    "Cannot parse to TradedPair<Symbol, SpotSettlement>: \"{kind}\". \
                    Expected: \"{PATTERN}\""
                )
            };

            let base_symbol = FromStr::from_str(base_symbol).unwrap_or_else(
                |err| panic!("Cannot parse {base_symbol} to Symbol. Error: {err:?}")
            );
            let base_asset = Base::new(base_symbol).into();
            TradedPair {
                quoted_asset,
                settlement_asset: base_asset,
                settlement_determinant: SpotSettlement,
            }
        }
    }
}