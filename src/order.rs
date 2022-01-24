use crate::{
    settlement::GetSettlementLag,
    traded_pair::TradedPair,
    types::{Direction, Id, OrderID, Price, Size},
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct LimitOrderCancelRequest<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct LimitOrderPlacingRequest<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub direction: Direction,
    pub price: Price,
    pub size: Size,
    pub dummy: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MarketOrderPlacingRequest<Symbol: Id, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub direction: Direction,
    pub size: Size,
    pub dummy: bool,
}