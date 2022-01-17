use crate::{
    settlement::GetSettlementLag,
    types::{DateTime, Identifier, Price},
};

pub mod concrete;
pub mod parser;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct TradedPair<Name: Identifier, Settlement: GetSettlementLag> {
    pub kind: PairKind<Settlement>,
    pub quoted_symbol: Name,
    pub base_symbol: Name,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum PairKind<Settlement: GetSettlementLag> {
    Base(Base<Settlement>),
    Futures(Futures<Settlement>),
    OptionContract(OptionContract<Settlement>),
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Base<Settlement: GetSettlementLag> {
    pub delivery: Settlement,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Futures<Settlement: GetSettlementLag> {
    pub delivery: Settlement,
    pub maturity: DateTime,
    pub strike: Price,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct OptionContract<Settlement: GetSettlementLag> {
    pub kind: OptionKind,
    pub delivery: Settlement,
    pub maturity: DateTime,
    pub strike: Price,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum OptionKind {
    EuroPut,
    EuroCall,
}

impl<Settlement: GetSettlementLag> Base<Settlement> {
    pub fn new(delivery: Settlement) -> Self {
        Self { delivery }
    }
}