use crate::{
    settlement::GetSettlementLag,
    traded_pair::TradedPair,
    types::{Direction, Identifier, OrderID, Price, Size},
};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LimitOrderCancelRequest<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LimitOrderPlacingRequest<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub direction: Direction,
    pub price: Price,
    pub size: Size,
    pub dummy: bool,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MarketOrderPlacingRequest<Symbol: Identifier, Settlement: GetSettlementLag> {
    pub traded_pair: TradedPair<Symbol, Settlement>,
    pub order_id: OrderID,
    pub direction: Direction,
    pub size: Size,
    pub dummy: bool,
}