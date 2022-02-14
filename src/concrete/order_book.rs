use {
    crate::{concrete::types::{Direction, ObState, OrderID, Price, Size}, types::DateTime},
    std::{
        cmp::Ordering,
        collections::{hash_map::Entry::Occupied, HashMap, VecDeque},
        iter::{once, repeat_with},
    },
};

#[cfg(test)]
mod tests;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
/// [`OrderBook`] internal limit order representation.
pub struct LimitOrder {
    /// Order unique identifier.
    pub id: OrderID,
    /// Order remaining size.
    pub size: Size,
    /// Indicates whether the order is dummy.
    /// If the order is dummy,
    /// it does not affect the size of the opposite orders with which it is executed.
    pub is_dummy: bool,
    /// Order submission datetime.
    pub dt: DateTime,
}

/// Order book that only supports simple limit and market orders.
pub struct OrderBook {
    /// Bid levels.
    bids: VecDeque<VecDeque<LimitOrder>>,
    /// Ask levels.
    asks: VecDeque<VecDeque<LimitOrder>>,
    /// Best bid price.
    best_bid: Price,
    /// Best ask price.
    best_ask: Price,
    /// Map [OrderId -> (Price, Whether it is bid)]
    id_to_price_and_side: HashMap<OrderID, (Price, bool)>,
}

#[derive(Debug, Eq, PartialEq)]
/// Order book execution event.
pub struct OrderBookEvent {
    /// Size of the diff.
    pub size: Size,
    /// Price of the diff.
    pub price: Price,
    /// Order book event kind.
    pub kind: OrderBookEventKind,
}

#[derive(Debug, Eq, PartialEq)]
/// Order book event.
pub enum OrderBookEventKind {
    /// New order executed.
    NewOrderExecuted,
    /// New order partially executed for the given price.
    NewOrderPartiallyExecuted,
    /// Old limit order fully executed.
    OldOrderExecuted(OrderID),
    /// Old limit order partially executed.
    OldOrderPartiallyExecuted(OrderID),
}

#[macro_use]
mod order_book_logic_macros {
    macro_rules! match_dummy_with_level {
        (
            $callback             : ident,
            $level                : ident,
            $size                 : ident,
            $size_before_matching : ident,
            $price                : ident
        ) => {
            for order in $level.iter_mut().filter(|order| order.size != Size(0) && !order.is_dummy)
            {
                if $size > order.size {
                    $size -= order.size;
                } else {
                    $callback(
                        OrderBookEvent {
                            size: $size_before_matching,
                            price: $price,
                            kind: OrderBookEventKind::NewOrderExecuted,
                        }
                    );
                    return;
                }
            }
        };
    }
    macro_rules! match_real_with_level {
        (
            $UPPER                :  expr,
            $ob                   : ident,
            $callback             : ident,
            $side                 : ident,
            $level                : ident,
            $size                 : ident,
            $size_before_matching : ident,
            $price                : ident
        ) => {
            for order in $level.iter_mut().filter(|order| order.size != Size(0)) {
                if !order.is_dummy {
                    match $size.cmp(&order.size) {
                        Ordering::Less => {
                            // (OrderExecuted, OrderPartiallyExecuted)
                            $callback(
                                OrderBookEvent {
                                    size: $size,
                                    price: $price,
                                    kind: OrderBookEventKind::OldOrderPartiallyExecuted(order.id),
                                }
                            );
                            $callback(
                                OrderBookEvent {
                                    size: $size_before_matching,
                                    price: $price,
                                    kind: OrderBookEventKind::NewOrderExecuted,
                                }
                            );
                            order.size -= $size;
                            shrink_level!($level);
                            shrink_side!($UPPER, $ob);
                            return;
                        }
                        Ordering::Equal => {
                            // (OrderExecuted, OrderExecuted)
                            $callback(
                                OrderBookEvent {
                                    size: $size,
                                    price: $price,
                                    kind: OrderBookEventKind::OldOrderExecuted(order.id),
                                }
                            );
                            $callback(
                                OrderBookEvent {
                                    size: $size_before_matching,
                                    price: $price,
                                    kind: OrderBookEventKind::NewOrderExecuted,
                                }
                            );
                            order.size = Size(0);
                            shrink_level!($level);
                            shrink_side!($UPPER, $ob);
                            return;
                        }
                        Ordering::Greater => {
                            // (OrderPartiallyExecuted, OrderExecuted)
                            $callback(
                                OrderBookEvent {
                                    size: order.size,
                                    price: $price,
                                    kind: OrderBookEventKind::OldOrderExecuted(order.id),
                                }
                            );
                            $size -= order.size;
                            order.size = Size(0);
                        }
                    }
                } else if order.size > $size {
                    $callback(
                        OrderBookEvent {
                            size: $size,
                            price: $price,
                            kind: OrderBookEventKind::OldOrderPartiallyExecuted(order.id),
                        }
                    );
                    order.size -= $size;
                } else {
                    $callback(
                        OrderBookEvent {
                            size: order.size,
                            price: $price,
                            kind: OrderBookEventKind::OldOrderExecuted(order.id),
                        }
                    );
                    order.size = Size(0);
                }
            }
        };
    }
    macro_rules! shrink_level {
        ($level:ident) => {
            while let Some(order) = $level.front() {
                if order.size != Size(0) {
                    break;
                }
                $level.pop_front();
            }
            while let Some(order) = $level.back() {
                if order.size != Size(0) {
                    break;
                }
                $level.pop_back();
            }
        };
    }
    macro_rules! shrink_side {
        ($UPPER:expr, $ob:ident) => {
            let side = if $UPPER {
                &mut $ob.asks
            } else {
                &mut $ob.bids
            };
            while let Some(level) = side.front() {
                if !level.is_empty() {
                    break;
                }
                side.pop_front();
                if $UPPER {
                    $ob.best_ask += Price(1)
                } else {
                    $ob.best_bid -= Price(1)
                }
            }
            while let Some(level) = side.back() {
                if !level.is_empty() {
                    break;
                }
                side.pop_back();
            }
        };
    }
}

impl OrderBook {
    #[inline]
    /// Creates a new instance of the `OrderBook`.
    pub fn new() -> Self {
        OrderBook {
            bids: Default::default(),
            asks: Default::default(),
            best_bid: Price(0),
            best_ask: Price(0),
            id_to_price_and_side: Default::default(),
        }
    }

    #[inline]
    /// Clears the `OrderBook`.
    pub fn clear(&mut self) {
        self.best_bid = Price(0);
        self.best_ask = Price(0);
        self.bids.clear();
        self.asks.clear();
        self.id_to_price_and_side.clear();
    }

    #[inline]
    /// Yields all IDs of the active limit orders.
    pub fn get_all_ids(&self) -> impl Iterator<Item=OrderID> + '_ {
        let get_order_ids = |order: &LimitOrder| {
            if order.size != Size(0) { Some(order.id) } else { None }
        };
        self.asks.iter()
            .map(move |level| level.iter().filter_map(get_order_ids))
            .flatten()
            .chain(
                self.bids.iter()
                    .map(move |level| level.iter().filter_map(get_order_ids))
                    .flatten()
            )
    }

    #[inline]
    /// Cancels limit order, returning the cancelled limit order meta-information if successful.
    ///
    /// # Arguments
    ///
    /// * `id` — Order ID to cancel.
    pub fn cancel_limit_order(&mut self, id: OrderID) -> Option<(LimitOrder, Direction, Price)>
    {
        let (price, buy) = if let Occupied(e) = self.id_to_price_and_side.entry(id) {
            e.remove()
        } else {
            return None;
        };
        let (side, offset) = if buy {
            (&mut self.bids, i64::from(self.best_bid - price))
        } else {
            (&mut self.asks, i64::from(price - self.best_ask))
        };
        if offset >= 0 {
            if let Some(level) = side.get_mut(offset as usize) {
                if let Some(order) = level.iter_mut()
                    .filter(|order| order.id == id && order.size != Size(0))
                    .next()
                {
                    let cancelled_order = *order;
                    order.size = Size(0);
                    shrink_level!(level);
                    let direction = if buy {
                        shrink_side!(false, self);
                        Direction::Buy
                    } else {
                        shrink_side!(true, self);
                        Direction::Sell
                    };
                    return Some((cancelled_order, direction, price));
                }
            }
        }
        None
    }

    /// Inserts limit order.
    ///
    /// # Parameters
    ///
    /// * `DUMMY` — Whether the order is dummy.
    /// * `BUY` — Whether the order is bid.
    ///
    /// # Arguments
    ///
    /// * `dt` — Submission datetime.
    /// * `id` — ID of the order to insert.
    /// * `price` — Order price.
    /// * `size` — Order size.
    /// * `callback` — Callback.
    pub fn insert_limit_order<CallBack: FnMut(OrderBookEvent), const DUMMY: bool, const BUY: bool>(
        &mut self,
        dt: DateTime,
        id: OrderID,
        price: Price,
        mut size: Size,
        mut callback: CallBack,
    ) {
        let opposite_side = if BUY {
            &mut self.asks
        } else {
            &mut self.bids
        };
        // Match the new limit order
        // with already submitted limit orders from the opposite side of the order book
        if !opposite_side.is_empty() {
            let intersection_depth = if BUY {
                i64::from(price - self.best_ask)
            } else {
                i64::from(self.best_bid - price)
            };
            // Nearly the same logic as in the insert_market_order method
            if intersection_depth >= 0 {
                let mut price = if BUY {
                    self.best_ask
                } else {
                    self.best_bid
                };
                for level in opposite_side.iter_mut()
                    .take((1 + intersection_depth) as usize)
                {
                    let size_before_matching = size;
                    if DUMMY {
                        match_dummy_with_level!(callback, level, size, size_before_matching, price)
                    } else {
                        match_real_with_level!(
                            BUY, self,
                            callback, opposite_side, level, size, size_before_matching, price
                        )
                    }
                    let exec_size = size_before_matching - size;
                    if exec_size != Size(0) {
                        callback(
                            OrderBookEvent {
                                size: exec_size,
                                price,
                                kind: OrderBookEventKind::NewOrderPartiallyExecuted,
                            }
                        )
                    }
                    shrink_level!(level);
                    if BUY {
                        price += Price(1)
                    } else {
                        price -= Price(1)
                    }
                }
                shrink_side!(BUY, self);
            }
        }
        // Insert the remaining size of the new limit order into the order book
        self.id_to_price_and_side.insert(id, (price, BUY));
        let side = if BUY {
            &mut self.bids
        } else {
            &mut self.asks
        };
        if side.is_empty() {
            // Case if the corresponding side of the order book does not have any orders
            side.push_back([LimitOrder { dt, id, size, is_dummy: DUMMY }].into());
            if BUY {
                self.best_bid = price
            } else {
                self.best_ask = price
            }
        } else {
            // Check whether the new limit order lies inside the spread
            let offset = if BUY {
                i64::from(self.best_bid - price)
            } else {
                i64::from(price - self.best_ask)
            };
            if offset < 0 {
                // If actually lies, modify front of the corresponding side
                for _ in 1..-offset {
                    side.push_front(Default::default())
                }
                side.push_front([LimitOrder { dt, id, size, is_dummy: DUMMY }].into());
                if BUY {
                    self.best_bid = price
                } else {
                    self.best_ask = price
                }
            } else {
                // If not, place order in the depth of the corresponding side
                let offset = offset as usize;
                if let Some(level) = side.get_mut(offset) {
                    level.push_back(LimitOrder { dt, id, size, is_dummy: DUMMY })
                } else {
                    side.extend(
                        repeat_with(Default::default)
                            .take(offset - side.len())
                            .chain(once([LimitOrder { dt, id, size, is_dummy: DUMMY }].into()))
                    )
                }
            }
        }
    }

    /// Inserts market order.
    ///
    /// # Parameters
    /// * `DUMMY` — Whether the order is dummy.
    /// * `BUY` — Whether the order is bid.
    ///
    /// # Arguments
    ///
    /// * `size` — Order size.
    /// * `callback` — Callback.
    pub fn insert_market_order<CallBack: FnMut(OrderBookEvent), const DUMMY: bool, const BUY: bool>(
        &mut self,
        mut size: Size,
        mut callback: CallBack,
    ) {
        let (side, mut price) = if BUY {
            (&mut self.asks, self.best_ask)
        } else {
            (&mut self.bids, self.best_bid)
        };
        for level in side.iter_mut() {
            let size_before_matching = size;
            if DUMMY {
                match_dummy_with_level!(callback, level, size, size_before_matching, price)
            } else {
                match_real_with_level!(
                    BUY, self,
                    callback, side, level, size, size_before_matching, price
                )
            }
            let exec_size = size_before_matching - size;
            if exec_size != Size(0) {
                callback(
                    OrderBookEvent {
                        size: exec_size,
                        price,
                        kind: OrderBookEventKind::NewOrderPartiallyExecuted,
                    }
                )
            }
            shrink_level!(level);
            if BUY {
                price += Price(1)
            } else {
                price -= Price(1)
            }
        }
        shrink_side!(BUY, self);
    }

    #[inline]
    /// Gets the current state of the order book side.
    ///
    /// # Parameters
    /// * `UPPER` — Whether the side is asks.
    ///
    /// # Arguments
    ///
    /// * `max_levels` — Maximum number of non-empty price levels to get.
    ///                  If zero, the number of levels is considered unlimited.
    pub fn get_ob_side<const UPPER: bool>(
        &self,
        max_levels: usize) -> Vec<(Price, Vec<(Size, DateTime)>)>
    {
        let (side, price) = if UPPER {
            (&self.asks, self.best_ask)
        } else {
            (&self.bids, self.best_bid)
        };
        let it = side.iter()
            .map(
                |level| -> Vec<(Size, DateTime)> {
                    level.iter()
                        .filter_map(
                            |order| if order.size != Size(0) && !order.is_dummy {
                                Some((order.size, order.dt))
                            } else {
                                None
                            }
                        ).collect()
                }
            )
            .scan(
                price,
                |price, level| {
                    let result = (*price, level);
                    if UPPER {
                        *price += Price(1)
                    } else {
                        *price -= Price(1)
                    }
                    Some(result)
                },
            )
            .filter(|(_, level)| !level.is_empty());
        if max_levels != 0 {
            it.take(max_levels).collect()
        } else {
            it.collect()
        }
    }

    #[inline]
    /// Gets the current state of the order book.
    ///
    /// # Arguments
    ///
    /// * `max_levels` — Maximum number of non-empty price levels per side to get.
    ///                  If zero, full order book state is returned.
    pub fn get_ob_state(&self, max_levels: usize) -> ObState {
        ObState {
            bids: self.get_ob_side::<false>(max_levels),
            asks: self.get_ob_side::<true>(max_levels),
        }
    }
}