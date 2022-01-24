use crate::{
    settlement::GetSettlementLag,
    types::{DateTime, Identifier, Named, Price},
    enum_def,
};

pub mod parser;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct TradedPair<Name: Identifier, Settlement: GetSettlementLag> {
    pub quoted_asset: Asset<Name>,
    pub base_asset: Base<Name>,
    pub settlement: Settlement,
}

enum_def! {
    #[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
    pub Asset<Name: Identifier> {
        Base<Name>,
        Futures<Name>,
        OptionContract<Name>
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Base<Name: Identifier> {
    pub symbol: Name,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Futures<Name: Identifier> {
    pub symbol: Name,
    pub underlying_symbol: Name,
    pub settlement_symbol: Name,
    pub maturity: DateTime,
    pub strike: Price,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct OptionContract<Name: Identifier> {
    pub symbol: Name,
    pub underlying_symbol: Name,
    pub settlement_symbol: Name,
    pub maturity: DateTime,
    pub strike: Price,
    pub kind: OptionKind,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum OptionKind {
    EuroPut,
    EuroCall,
}

impl<Name: Identifier> Base<Name> {
    pub fn new(symbol: Name) -> Self {
        Self { symbol }
    }
}

impl<Name: Identifier> Futures<Name> {
    pub fn new(
        symbol: Name,
        underlying_symbol: Name,
        settlement_symbol: Name,
        maturity: DateTime,
        strike: Price) -> Self
    {
        Self { symbol, underlying_symbol, settlement_symbol, maturity, strike }
    }
}

impl<Name: Identifier> OptionContract<Name> {
    pub fn new(
        symbol: Name,
        underlying_symbol: Name,
        settlement_symbol: Name,
        maturity: DateTime,
        strike: Price,
        kind: OptionKind) -> Self
    {
        Self { symbol, underlying_symbol, settlement_symbol, maturity, strike, kind }
    }
}

impl<Name: Identifier> Named<Name> for Base<Name> {
    fn get_name(&self) -> Name {
        self.symbol
    }
}

impl<Name: Identifier> Named<Name> for Futures<Name> {
    fn get_name(&self) -> Name {
        self.symbol
    }
}

impl<Name: Identifier> Named<Name> for OptionContract<Name> {
    fn get_name(&self) -> Name {
        self.symbol
    }
}