use {
    crate::{concrete::types::{Direction, ObState, OrderID, Price, Size}, types::DateTime},
    std::{
        cmp::Ordering,
        collections::{hash_map::Entry::Occupied, HashMap, VecDeque},
        fmt::{Display, Formatter},
        iter::{once, repeat_with},
    },
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
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
///
/// # Parameters
///
/// * `MATCH_DUMMY_WITH_DUMMY` — whether to match incoming dummy orders
/// with already submitted dummy orders.
pub struct OrderBook<const MATCH_DUMMY_WITH_DUMMY: bool> {
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

/// Borrows [`OrderBook`] side and performs cleanup on drop.
struct SideWrapper<'a, const UPPER: bool, const FROM_BOTH_ENDS: bool> {
    side: &'a mut VecDeque<VecDeque<LimitOrder>>,
    best_price: &'a mut Price,
}

impl<const UPPER: bool, const SHRINK_BOTH_ENDS: bool>
SideWrapper<'_, UPPER, SHRINK_BOTH_ENDS>
{
    #[inline]
    fn get_side_and_price(&mut self) -> (&mut VecDeque<VecDeque<LimitOrder>>, Price) {
        (self.side, *self.best_price)
    }

    #[inline]
    fn shrink_side(&mut self)
    {
        while let Some(level) = self.side.front() {
            if !level.is_empty() {
                break;
            }
            self.side.pop_front();
            if UPPER {
                *self.best_price += Price(1)
            } else {
                *self.best_price -= Price(1)
            }
        }
        if SHRINK_BOTH_ENDS {
            while let Some(level) = self.side.back() {
                if !level.is_empty() {
                    break;
                }
                self.side.pop_back();
            }
        }
    }
}

impl<const UPPER: bool, const SHRINK_BOTH_ENDS: bool>
Drop for SideWrapper<'_, UPPER, SHRINK_BOTH_ENDS>
{
    #[inline]
    fn drop(&mut self) {
        self.shrink_side()
    }
}

/// Borrows [`OrderBook`] side level and performs cleanup on drop.
struct LevelWrapper<'a, const SHRINK_BOTH_ENDS: bool> (&'a mut VecDeque<LimitOrder>);

impl<const SHRINK_BOTH_ENDS: bool> LevelWrapper<'_, SHRINK_BOTH_ENDS>
{
    #[inline]
    pub fn get_level(&mut self) -> &mut VecDeque<LimitOrder> {
        self.0
    }

    #[inline]
    pub fn shrink_level(&mut self)
    {
        while let Some(order) = self.0.front() {
            if order.size != Size(0) {
                break;
            }
            self.0.pop_front();
        }
        if SHRINK_BOTH_ENDS {
            while let Some(order) = self.0.back() {
                if order.size != Size(0) {
                    break;
                }
                self.0.pop_back();
            }
        }
    }
}

impl<const SHRINK_BOTH_ENDS: bool>
Drop for LevelWrapper<'_, SHRINK_BOTH_ENDS>
{
    #[inline]
    fn drop(&mut self) {
        self.shrink_level()
    }
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
/// Error struct indicating that there is no order with such ID.
pub struct NoSuchID;

impl Display for NoSuchID {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "No such ID is currently active in the order book")
    }
}

enum MatchingStatus {
    FullyExecuted,
    PartiallyExecuted(Size),
}

impl<const MATCH_DUMMY_WITH_DUMMY: bool> OrderBook<MATCH_DUMMY_WITH_DUMMY>
{
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
    pub fn cancel_limit_order(
        &mut self,
        id: OrderID) -> Result<(LimitOrder, Direction, Price), NoSuchID>
    {
        let (price, buy) = if let Occupied(e) = self.id_to_price_and_side.entry(id) {
            e.remove()
        } else {
            return Err(NoSuchID);
        };
        let res = if buy {
            self.cancel_limit_order_::<false>(id, price)
        } else {
            self.cancel_limit_order_::<true>(id, price)
        };
        Ok(res)
    }

    #[inline]
    fn cancel_limit_order_<const UPPER: bool>(
        &mut self,
        id: OrderID,
        price: Price) -> (LimitOrder, Direction, Price)
    {
        let mut opposite_side = if UPPER {
            SideWrapper::<UPPER, true> { side: &mut self.asks, best_price: &mut self.best_ask }
        } else {
            SideWrapper::<UPPER, true> { side: &mut self.bids, best_price: &mut self.best_bid }
        };
        let (side, best_price) = opposite_side.get_side_and_price();
        let offset = if UPPER {
            isize::from(price - best_price)
        } else {
            isize::from(best_price - price)
        };
        if offset >= 0 {
            if let Some(level) = side.get_mut(offset as usize) {
                if let Some(order) = level.iter_mut()
                    .filter(|order| order.id == id && order.size != Size(0))
                    .next()
                {
                    let cancelled_order = *order;
                    order.size = Size(0);
                    let direction = if UPPER {
                        Direction::Sell
                    } else {
                        Direction::Buy
                    };
                    LevelWrapper::<true>(level);
                    (cancelled_order, direction, price)
                } else {
                    unreachable!("No active order with such ID {} was found at the level", id)
                }
            } else {
                unreachable!(
                    "No non-empty level was found for the given offset {} \
                    from the best price",
                    offset
                )
            }
        } else {
            unreachable!("Offset from the best price appeared to be negative: {}", offset)
        }
    }

    #[inline]
    /// Updates size of the limit order and places it to the end of the current price queue.
    ///
    /// # Arguments
    ///
    /// * `id` — Order ID to update.
    /// * `new_size` — New size of the limit order.
    pub fn update_limit_order_moving_to_end(
        &mut self,
        id: OrderID,
        new_size: Size) -> Result<(), NoSuchID>
    {
        if new_size == Size(0) {
            self.cancel_limit_order(id)?;
            return Ok(());
        }
        let (price, buy) = if let Some(v) = self.id_to_price_and_side.get(&id) {
            *v
        } else {
            return Err(NoSuchID);
        };
        let (side, offset) = if buy {
            (&mut self.bids, isize::from(self.best_bid - price))
        } else {
            (&mut self.asks, isize::from(price - self.best_ask))
        };
        if offset >= 0 {
            if let Some(level) = side.get_mut(offset as usize) {
                if let Some(order) = level.iter_mut()
                    .filter(|order| order.id == id && order.size != Size(0))
                    .next()
                {
                    order.size = Size(0);
                    let LimitOrder { is_dummy, dt, .. } = *order;
                    LevelWrapper::<true>(level);
                    level.push_back(LimitOrder { id, size: new_size, is_dummy, dt })
                } else {
                    unreachable!("No active order with such ID {} was found at the level", id)
                }
            } else {
                unreachable!(
                    "No non-empty level was found for the given offset {} \
                    from the best price",
                    offset
                )
            }
        } else {
            unreachable!("Offset from the best price appeared to be negative: {}", offset)
        }
        Ok(())
    }

    #[inline]
    /// Updates size of the limit order.
    ///
    /// # Arguments
    ///
    /// * `id` — Order ID to update.
    /// * `new_size` — New size of the limit order.
    pub fn update_limit_order(&mut self, id: OrderID, new_size: Size) -> Result<(), NoSuchID>
    {
        if new_size == Size(0) {
            self.cancel_limit_order(id)?;
            return Ok(());
        }
        let (price, buy) = if let Some(v) = self.id_to_price_and_side.get(&id) {
            *v
        } else {
            return Err(NoSuchID);
        };
        let (side, offset) = if buy {
            (&mut self.bids, isize::from(self.best_bid - price))
        } else {
            (&mut self.asks, isize::from(price - self.best_ask))
        };
        if offset >= 0 {
            if let Some(level) = side.get_mut(offset as usize) {
                if let Some(order) = level.iter_mut()
                    .filter(|order| order.id == id && order.size != Size(0))
                    .next()
                {
                    order.size = new_size
                } else {
                    unreachable!("No active order with such ID {} was found at the level", id)
                }
            } else {
                unreachable!(
                    "No non-empty level was found for the given offset {} \
                    from the best price",
                    offset
                )
            }
        } else {
            unreachable!("Offset from the best price appeared to be negative: {}", offset)
        }
        Ok(())
    }

    /// Inserts limit order that is cancelled immediately after insertion.
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
    pub fn insert_instant_limit_order<
        CallBack: FnMut(OrderBookEvent),
        const DUMMY: bool,
        const BUY: bool
    >(
        &mut self,
        mut price: Price,
        mut size: Size,
        mut callback: CallBack,
    ) {
        let mut opposite_side = if BUY {
            SideWrapper::<BUY, false> { side: &mut self.asks, best_price: &mut self.best_ask }
        } else {
            SideWrapper::<BUY, false> { side: &mut self.bids, best_price: &mut self.best_bid }
        };
        let (opposite_side, best_price) = opposite_side.get_side_and_price();
        // Match the new limit order
        // with already submitted limit orders from the opposite side of the order book
        if !opposite_side.is_empty() {
            let intersection_depth = if BUY {
                isize::from(price - best_price)
            } else {
                isize::from(best_price - price)
            };
            // Nearly the same logic as in the insert_market_order method
            if intersection_depth >= 0 {
                price = best_price;
                for mut level in opposite_side.iter_mut()
                    .take((1 + intersection_depth) as usize)
                    .map(LevelWrapper::<false>)
                {
                    let level = level.get_level();
                    match Self::match_with_level::<_, DUMMY>(
                        level, price, size, &mut callback, &mut self.id_to_price_and_side,
                    ) {
                        MatchingStatus::FullyExecuted => {
                            callback(
                                OrderBookEvent {
                                    size,
                                    price,
                                    kind: OrderBookEventKind::NewOrderExecuted,
                                }
                            );
                            return;
                        }
                        MatchingStatus::PartiallyExecuted(exec_size) => {
                            if exec_size != Size(0) {
                                size -= exec_size;
                                callback(
                                    OrderBookEvent {
                                        size: exec_size,
                                        price,
                                        kind: OrderBookEventKind::NewOrderPartiallyExecuted,
                                    }
                                )
                            }
                        }
                    }
                    if BUY {
                        price += Price(1)
                    } else {
                        price -= Price(1)
                    }
                }
            }
        }
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
        {
            let mut opposite_side = if BUY {
                SideWrapper::<BUY, false> { side: &mut self.asks, best_price: &mut self.best_ask }
            } else {
                SideWrapper::<BUY, false> { side: &mut self.bids, best_price: &mut self.best_bid }
            };
            // Match the new limit order
            // with already submitted limit orders from the opposite side of the order book
            let (opposite_side, best_price) = opposite_side.get_side_and_price();
            if !opposite_side.is_empty() {
                let intersection_depth = if BUY {
                    isize::from(price - best_price)
                } else {
                    isize::from(best_price - price)
                };
                let mut price = best_price;
                // Nearly the same logic as in the insert_market_order method
                if intersection_depth >= 0 {
                    for mut level in opposite_side.iter_mut()
                        .take((1 + intersection_depth) as usize)
                        .map(LevelWrapper::<false>)
                    {
                        let level = level.get_level();
                        match Self::match_with_level::<_, DUMMY>(
                            level, price, size, &mut callback, &mut self.id_to_price_and_side,
                        ) {
                            MatchingStatus::FullyExecuted => {
                                callback(
                                    OrderBookEvent {
                                        size,
                                        price,
                                        kind: OrderBookEventKind::NewOrderExecuted,
                                    }
                                );
                                return;
                            }
                            MatchingStatus::PartiallyExecuted(exec_size) => {
                                if exec_size != Size(0) {
                                    size -= exec_size;
                                    callback(
                                        OrderBookEvent {
                                            size: exec_size,
                                            price,
                                            kind: OrderBookEventKind::NewOrderPartiallyExecuted,
                                        }
                                    )
                                }
                            }
                        }
                        if BUY {
                            price += Price(1)
                        } else {
                            price -= Price(1)
                        }
                    }
                }
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
                isize::from(self.best_bid - price)
            } else {
                isize::from(price - self.best_ask)
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
        let mut opposite_side = if BUY {
            SideWrapper::<BUY, false> { side: &mut self.asks, best_price: &mut self.best_ask }
        } else {
            SideWrapper::<BUY, false> { side: &mut self.bids, best_price: &mut self.best_bid }
        };
        let (side, mut price) = opposite_side.get_side_and_price();
        for mut level in side.iter_mut().map(LevelWrapper::<false>)
        {
            let level = level.get_level();
            match Self::match_with_level::<_, DUMMY>(
                level, price, size, &mut callback, &mut self.id_to_price_and_side,
            ) {
                MatchingStatus::FullyExecuted => {
                    callback(
                        OrderBookEvent {
                            size,
                            price,
                            kind: OrderBookEventKind::NewOrderExecuted,
                        }
                    );
                    return;
                }
                MatchingStatus::PartiallyExecuted(exec_size) => {
                    if exec_size != Size(0) {
                        size -= exec_size;
                        callback(
                            OrderBookEvent {
                                size: exec_size,
                                price,
                                kind: OrderBookEventKind::NewOrderPartiallyExecuted,
                            }
                        )
                    }
                }
            }
            if BUY {
                price += Price(1)
            } else {
                price -= Price(1)
            }
        }
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

    fn match_with_level<Callback: FnMut(OrderBookEvent), const DUMMY: bool>(
        level: &mut VecDeque<LimitOrder>,
        price: Price,
        size: Size,
        callback: &mut Callback,
        id_to_price_and_side: &mut HashMap<OrderID, (Price, bool)>) -> MatchingStatus
    {
        if DUMMY {
            Self::match_dummy_with_level(level, price, size, callback, id_to_price_and_side)
        } else {
            Self::match_real_with_level(level, price, size, callback, id_to_price_and_side)
        }
    }

    fn match_dummy_with_level(
        level: &mut VecDeque<LimitOrder>,
        price: Price,
        mut size: Size,
        callback: &mut impl FnMut(OrderBookEvent),
        id_to_price_and_side: &mut HashMap<OrderID, (Price, bool)>) -> MatchingStatus
    {
        let size_before_matching = size;
        for order in level.iter_mut().filter(|order| order.size != Size(0)) {
            if order.is_dummy {
                if MATCH_DUMMY_WITH_DUMMY {
                    match size.cmp(&order.size) {
                        Ordering::Less => {
                            // (OrderExecuted, OrderPartiallyExecuted)
                            callback(
                                OrderBookEvent {
                                    size,
                                    price,
                                    kind: OrderBookEventKind::OldOrderPartiallyExecuted(order.id),
                                }
                            );
                            order.size -= size;
                            return MatchingStatus::FullyExecuted;
                        }
                        Ordering::Equal => {
                            // (OrderExecuted, OrderExecuted)
                            id_to_price_and_side.remove(&order.id).unwrap_or_else(
                                || unreachable!(
                                    "id_to_price_and_side does not contain {}",
                                    order.id
                                )
                            );
                            callback(
                                OrderBookEvent {
                                    size,
                                    price,
                                    kind: OrderBookEventKind::OldOrderExecuted(order.id),
                                }
                            );
                            order.size = Size(0);
                            return MatchingStatus::FullyExecuted;
                        }
                        Ordering::Greater => {
                            // (OrderPartiallyExecuted, OrderExecuted)
                            id_to_price_and_side.remove(&order.id).unwrap_or_else(
                                || unreachable!(
                                    "id_to_price_and_side does not contain {}",
                                    order.id
                                )
                            );
                            callback(
                                OrderBookEvent {
                                    size: order.size,
                                    price,
                                    kind: OrderBookEventKind::OldOrderExecuted(order.id),
                                }
                            );
                            size -= order.size;
                            order.size = Size(0);
                        }
                    }
                }
            } else if size > order.size {
                size -= order.size;
            } else {
                return MatchingStatus::FullyExecuted;
            }
        }
        MatchingStatus::PartiallyExecuted(size_before_matching - size)
    }

    fn match_real_with_level(
        level: &mut VecDeque<LimitOrder>,
        price: Price,
        mut size: Size,
        callback: &mut impl FnMut(OrderBookEvent),
        id_to_price_and_side: &mut HashMap<OrderID, (Price, bool)>) -> MatchingStatus
    {
        let size_before_matching = size;
        for order in level.iter_mut().filter(|order| order.size != Size(0)) {
            if !order.is_dummy {
                match size.cmp(&order.size) {
                    Ordering::Less => {
                        // (OrderExecuted, OrderPartiallyExecuted)
                        callback(
                            OrderBookEvent {
                                size,
                                price,
                                kind: OrderBookEventKind::OldOrderPartiallyExecuted(order.id),
                            }
                        );
                        order.size -= size;
                        return MatchingStatus::FullyExecuted;
                    }
                    Ordering::Equal => {
                        // (OrderExecuted, OrderExecuted)
                        id_to_price_and_side.remove(&order.id).unwrap_or_else(
                            || unreachable!(
                                "id_to_price_and_side does not contain {}",
                                order.id
                            )
                        );
                        callback(
                            OrderBookEvent {
                                size,
                                price,
                                kind: OrderBookEventKind::OldOrderExecuted(order.id),
                            }
                        );
                        order.size = Size(0);
                        return MatchingStatus::FullyExecuted;
                    }
                    Ordering::Greater => {
                        // (OrderPartiallyExecuted, OrderExecuted)
                        id_to_price_and_side.remove(&order.id).unwrap_or_else(
                            || unreachable!(
                                "id_to_price_and_side does not contain {}",
                                order.id
                            )
                        );
                        callback(
                            OrderBookEvent {
                                size: order.size,
                                price,
                                kind: OrderBookEventKind::OldOrderExecuted(order.id),
                            }
                        );
                        size -= order.size;
                        order.size = Size(0);
                    }
                }
            } else if order.size > size {
                callback(
                    OrderBookEvent {
                        size,
                        price,
                        kind: OrderBookEventKind::OldOrderPartiallyExecuted(order.id),
                    }
                );
                order.size -= size;
            } else {
                id_to_price_and_side.remove(&order.id).unwrap_or_else(
                    || unreachable!(
                        "id_to_price_and_side does not contain {}",
                        order.id
                    )
                );
                callback(
                    OrderBookEvent {
                        size: order.size,
                        price,
                        kind: OrderBookEventKind::OldOrderExecuted(order.id),
                    }
                );
                order.size = Size(0);
            }
        }
        MatchingStatus::PartiallyExecuted(size_before_matching - size)
    }
}