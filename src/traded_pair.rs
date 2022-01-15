use {
    crate::{
        settlement::GetSettlementLag,
        types::{DateTime, Identifier, Price},
        utils::enum_dispatch,
    },
    std::str::FromStr,
};

pub mod concrete;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct TradedPair<Name: Identifier, Settlement: GetSettlementLag> {
    pub kind: PairKind<Settlement>,
    pub quoted_symbol: Name,
    pub base_symbol: Name,
}

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

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum PairKind<Settlement: GetSettlementLag> {
    Spot,
    Futures(Futures<Settlement>),
    Option(Option<Settlement>),
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Futures<Settlement: GetSettlementLag> {
    pub maturity: DateTime,
    pub delivery: Settlement,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Option<Settlement: GetSettlementLag> {
    pub kind: OptionKind,
    pub strike: Price,
    pub maturity: DateTime,
    pub delivery: Settlement,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum OptionKind {
    EuroPut,
    EuroCall,
}