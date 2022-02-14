use crate::{
    concrete::{
        traded_pair::{settlement::GetSettlementLag, TradedPair},
        types::{Direction, OrderID, Price, Size},
    },
    types::Id,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
/// Limit order cancel request.
pub struct LimitOrderCancelRequest<Symbol: Id, Settlement: GetSettlementLag> {
    /// Traded pair.
    pub traded_pair: TradedPair<Symbol, Settlement>,
    /// ID of the order to cancel.
    pub order_id: OrderID,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
/// Limit order placing request.
pub struct LimitOrderPlacingRequest<Symbol: Id, Settlement: GetSettlementLag> {
    /// Traded pair.
    pub traded_pair: TradedPair<Symbol, Settlement>,
    /// ID of the order to place.
    pub order_id: OrderID,
    /// Direction of the order to place.
    pub direction: Direction,
    /// Price of the order to place.
    pub price: Price,
    /// Size of the order to place.
    pub size: Size,
    /// Whether the order is dummy.
    pub dummy: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
/// Limit order placing request.
pub struct MarketOrderPlacingRequest<Symbol: Id, Settlement: GetSettlementLag> {
    /// Traded pair.
    pub traded_pair: TradedPair<Symbol, Settlement>,
    /// ID of the order to place.
    pub order_id: OrderID,
    /// Direction of the order to place.
    pub direction: Direction,
    /// Size of the order to place.
    pub size: Size,
    /// Whether the order is dummy.
    pub dummy: bool,
}