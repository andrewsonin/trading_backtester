use crate::{
    traded_pair::TradedPair,
    types::{Direction, Identifier, OrderID, Price, Size},
};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LimitOrderCancelRequest<Symbol: Identifier> {
    pub traded_pair: TradedPair<Symbol>,
    pub order_id: OrderID,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LimitOrderPlacingRequest<Symbol: Identifier> {
    pub traded_pair: TradedPair<Symbol>,
    pub order_id: OrderID,
    pub direction: Direction,
    pub price: Price,
    pub size: Size,
    pub dummy: bool,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MarketOrderPlacingRequest<Symbol: Identifier> {
    pub traded_pair: TradedPair<Symbol>,
    pub order_id: OrderID,
    pub direction: Direction,
    pub size: Size,
    pub dummy: bool,
}