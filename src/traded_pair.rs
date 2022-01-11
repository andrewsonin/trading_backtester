use {
    crate::{types::{DateTime, Identifier, Price}, utils::enum_dispatch},
    std::str::FromStr,
};

pub mod concrete;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct TradedPair<Name: Identifier> {
    pub kind: PairKind,
    pub quoted_symbol: Name,
    pub base_symbol: Name,
}

pub trait TradedPairParser<Symbol: Identifier + FromStr> {
    fn parse(kind: &str, quoted_symbol: &str, base_symbol: &str) -> TradedPair<Symbol>;
}

#[enum_dispatch]
pub trait GetSettlementDateTime {
    fn get_settlement_dt(&self, transaction_dt: DateTime) -> DateTime;
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum PairKind {
    Spot(Spot),
    // TODO:
    // Futures(Futures),
    // Option(Option),
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Spot {
    pub settlement: SettleKind,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum SettleKind {
    Immediately,
    // TODO:
    // Today,
    // TP1,
    // TP2,
    // TP3,
    // Custom(GetSettlementDateTime),
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Futures {
    pub maturity: DateTime,
    pub delivery: SettleKind,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Option {
    pub kind: OptionKind,
    pub strike: Price,
    pub maturity: DateTime,
    pub delivery: SettleKind,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum OptionKind {
    EuroPut,
    EuroCall,
}