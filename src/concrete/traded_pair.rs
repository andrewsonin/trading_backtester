use {
    crate::{
        concrete::types::Price,
        enum_def,
        types::{DateTime, Id, Named},
    },
    settlement::GetSettlementLag,
};

/// Traded pair parser examples.
pub mod parser;
/// Traded pair settlement.
pub mod settlement;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
/// Traded pair.
pub struct TradedPair<Name: Id, Settlement: GetSettlementLag> {
    /// Quoted asset.
    pub quoted_asset: Asset<Name>,
    /// Settlement asset.
    pub settlement_asset: Asset<Name>,
    /// Settlement determinant.
    pub settlement_determinant: Settlement,
}

enum_def! {
    #[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
    /// Asset.
    pub Asset<Name: Id> {
        /// Base asset.
        Base<Name>,
        /// Futures contract.
        Futures<Name>,
        /// Option contract.
        OptionContract<Name>
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
/// Base asset.
pub struct Base<Name: Id> {
    /// Unique ID of the `Base`.
    pub symbol: Name,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
/// Futures contract.
pub struct Futures<Name: Id> {
    /// Unique ID of the `Futures`.
    pub symbol: Name,
    /// Underlying symbol.
    pub underlying_symbol: Name,
    /// Settlement symbol.
    pub settlement_symbol: Name,
    /// Maturity datetime.
    pub maturity: DateTime,
    /// Strike price.
    pub strike: Price,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
/// Option contract.
pub struct OptionContract<Name: Id> {
    /// Unique ID of the `OptionContract`.
    pub symbol: Name,
    /// Underlying symbol.
    pub underlying_symbol: Name,
    /// Settlement symbol.
    pub settlement_symbol: Name,
    /// Maturity datetime.
    pub maturity: DateTime,
    /// Strike price.
    pub strike: Price,
    /// Kind of `OptionContract`.
    pub kind: OptionKind,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
/// Option kind.
pub enum OptionKind {
    /// European put option.
    EuroPut,
    /// European call option.
    EuroCall,
}

impl<Name: Id> Base<Name> {
    /// Creates a new instance of the `Base`.
    ///
    /// # Arguments
    ///
    /// * `symbol` — Unique ID of the `Base`.
    pub fn new(symbol: Name) -> Self {
        Self { symbol }
    }
}

impl<Name: Id> Futures<Name> {
    /// Creates a new instance of the `Futures`.
    ///
    /// # Arguments
    ///
    /// * `symbol` — Unique ID of the `Futures`.
    /// * `underlying_symbol` — Underlying symbol.
    /// * `settlement_symbol` — Settlement symbol.
    /// * `maturity` — Maturity datetime.
    /// * `strike` — Strike price.
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

impl<Name: Id> OptionContract<Name> {
    /// Creates a new instance of the `OptionContract`.
    ///
    /// # Arguments
    ///
    /// * `symbol` — Unique ID of the `OptionContract`.
    /// * `underlying_symbol` — Underlying symbol.
    /// * `settlement_symbol` — Settlement symbol.
    /// * `maturity` — Maturity datetime.
    /// * `strike` — Strike price.
    /// * `kind` — Kind of `OptionContract`.
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

impl<Name: Id> Named<Name> for Base<Name> {
    fn get_name(&self) -> Name {
        self.symbol
    }
}

impl<Name: Id> Named<Name> for Futures<Name> {
    fn get_name(&self) -> Name {
        self.symbol
    }
}

impl<Name: Id> Named<Name> for OptionContract<Name> {
    fn get_name(&self) -> Name {
        self.symbol
    }
}

impl<Name: Id> Into<Asset<Name>> for Base<Name> {
    fn into(self) -> Asset<Name> {
        Asset::Base(self)
    }
}

impl<Name: Id> Into<Asset<Name>> for Futures<Name> {
    fn into(self) -> Asset<Name> {
        Asset::Futures(self)
    }
}

impl<Name: Id> Into<Asset<Name>> for OptionContract<Name> {
    fn into(self) -> Asset<Name> {
        Asset::OptionContract(self)
    }
}