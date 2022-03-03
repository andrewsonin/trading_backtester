use crate::{
    concrete::{
        order_book::{LimitOrder, NoSuchID, OrderBook, OrderBookEvent, OrderBookEventKind::*},
        types::{Direction::*, Lots, ObState, OrderID, Tick},
    },
    types::{Date, DateTime},
};

fn insert_limit_order<const DUMMY: bool, const BID: bool>(
    ob: &mut OrderBook<false>,
    dt: DateTime,
    id: OrderID,
    price: Tick,
    size: Lots) -> Vec<OrderBookEvent>
{
    let mut ob_events = Vec::new();
    let callback = |event| ob_events.push(event);
    ob.insert_limit_order::<_, DUMMY, BID>(dt, id, price, size, callback);
    ob_events
}

fn insert_market_order<const DUMMY: bool, const BUY: bool>(
    ob: &mut OrderBook<false>,
    size: Lots) -> Vec<OrderBookEvent>
{
    let mut ob_events = Vec::new();
    let callback = |event| ob_events.push(event);
    ob.insert_market_order::<_, DUMMY, BUY>(size, callback);
    ob_events
}

fn default_example<const TEST: bool>() -> OrderBook<false>
{
    let mut order_book = OrderBook::new();
    for (dt, id, price, size, bid) in [
        (Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00), OrderID(0), Tick(27), Lots(3), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04), OrderID(1), Tick(23), Lots(4), true),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05), OrderID(2), Tick(26), Lots(8), true),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04), OrderID(3), Tick(23), Lots(44), true),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09), OrderID(4), Tick(29), Lots(126), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11), OrderID(5), Tick(28), Lots(6), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11), OrderID(6), Tick(29), Lots(8), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14), OrderID(7), Tick(28), Lots(3), false),
    ] {
        let response = if bid {
            insert_limit_order::<false, true>(&mut order_book, dt, id, price, size)
        } else {
            insert_limit_order::<false, false>(&mut order_book, dt, id, price, size)
        };
        if TEST {
            assert_eq!(response, [])
        }
    }
    order_book
}

fn default_example_bids(order_book: &mut OrderBook<false>)
{
    for (dt, id, price, size, bid) in [
        (Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04), OrderID(1), Tick(23), Lots(4), true),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05), OrderID(2), Tick(26), Lots(8), true),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04), OrderID(3), Tick(23), Lots(44), true),
    ] {
        if bid {
            insert_limit_order::<false, true>(order_book, dt, id, price, size)
        } else {
            insert_limit_order::<false, false>(order_book, dt, id, price, size)
        };
    }
}

fn default_example_asks(order_book: &mut OrderBook<false>)
{
    for (dt, id, price, size, bid) in [
        (Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00), OrderID(0), Tick(27), Lots(3), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09), OrderID(4), Tick(29), Lots(126), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11), OrderID(5), Tick(28), Lots(6), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11), OrderID(6), Tick(29), Lots(8), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14), OrderID(7), Tick(28), Lots(3), false),
    ] {
        if bid {
            insert_limit_order::<false, true>(order_book, dt, id, price, size)
        } else {
            insert_limit_order::<false, false>(order_book, dt, id, price, size)
        };
    }
}

fn default_example_dummies(order_book: &mut OrderBook<false>)
{
    for (dt, id, price, size, bid) in [
        (Date::from_ymd(2020, 02, 04).and_hms(07, 00, 00), OrderID(8), Tick(26), Lots(3), true),
        (Date::from_ymd(2020, 02, 04).and_hms(08, 08, 09), OrderID(9), Tick(27), Lots(5535), false),
    ] {
        if bid {
            insert_limit_order::<true, true>(order_book, dt, id, price, size)
        } else {
            insert_limit_order::<true, false>(order_book, dt, id, price, size)
        };
    }
}

#[test]
fn test_default_example()
{
    let order_book = default_example::<true>();
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(27));
}

#[test]
fn test_default_example_bids()
{
    let mut order_book = OrderBook::new();
    default_example_bids(&mut order_book);
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![],
        }
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(0))
}

#[test]
fn test_default_example_asks()
{
    let mut order_book = OrderBook::new();
    default_example_asks(&mut order_book);
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![],
            asks: vec![
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Tick(0));
    assert_eq!(order_book.best_ask, Tick(27))
}

#[test]
fn test_default_example_equivalence()
{
    let mut order_book1 = OrderBook::new();
    default_example_asks(&mut order_book1);
    default_example_bids(&mut order_book1);
    let order_book2 = default_example::<false>();
    assert_eq!(order_book1.get_ob_state(0), order_book2.get_ob_state(0))
}

#[test]
fn test_default_example_dummies()
{
    let mut order_book_with_dummies = default_example::<false>();
    default_example_dummies(&mut order_book_with_dummies);
    let order_book = default_example::<false>();
    assert_eq!(order_book.get_ob_state(0), order_book_with_dummies.get_ob_state(0))
}

#[test]
fn test_clear()
{
    let mut order_book = default_example::<false>();
    order_book.clear();
    assert_eq!(order_book.get_ob_state(0), ObState { bids: vec![], asks: vec![] })
}

#[test]
fn test_insert_real_sell_market_order()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_market_order::<false, false>(&mut order_book, Lots(20)),
        [
            OrderBookEvent { size: Lots(8), price: Tick(26), kind: OldOrderExecuted(OrderID(2)) },
            OrderBookEvent { size: Lots(3), price: Tick(26), kind: OldOrderExecuted(OrderID(8)) },
            OrderBookEvent { size: Lots(8), price: Tick(26), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(4), price: Tick(23), kind: OldOrderExecuted(OrderID(1)) },
            OrderBookEvent { size: Lots(8), price: Tick(23), kind: OldOrderPartiallyExecuted(OrderID(3)) },
            OrderBookEvent { size: Lots(12), price: Tick(23), kind: NewOrderExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(23),
                    vec![
                        (Lots(36), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04))
                    ]
                )
            ],
            asks: vec![
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Tick(23));
    assert_eq!(order_book.best_ask, Tick(27))
}

#[test]
fn test_insert_real_sell_market_order_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_market_order::<false, false>(&mut order_book, Lots(100)),
        [
            OrderBookEvent { size: Lots(8), price: Tick(26), kind: OldOrderExecuted(OrderID(2)) },
            OrderBookEvent { size: Lots(3), price: Tick(26), kind: OldOrderExecuted(OrderID(8)) },
            OrderBookEvent { size: Lots(8), price: Tick(26), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(4), price: Tick(23), kind: OldOrderExecuted(OrderID(1)) },
            OrderBookEvent { size: Lots(44), price: Tick(23), kind: OldOrderExecuted(OrderID(3)) },
            OrderBookEvent { size: Lots(48), price: Tick(23), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![],
            asks: vec![
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_ask, Tick(27))
}

#[test]
fn test_insert_real_sell_market_order_no_opposite_side()
{
    let mut order_book = OrderBook::new();
    default_example_asks(&mut order_book);
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_market_order::<false, false>(&mut order_book, Lots(100)),
        [
            OrderBookEvent { size: Lots(3), price: Tick(26), kind: OldOrderExecuted(OrderID(8)) }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![],
            asks: vec![
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_ask, Tick(27))
}

#[test]
fn test_insert_real_buy_market_order()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_market_order::<false, true>(&mut order_book, Lots(20)),
        [
            OrderBookEvent { size: Lots(3), price: Tick(27), kind: OldOrderExecuted(OrderID(0)) },
            OrderBookEvent { size: Lots(17), price: Tick(27), kind: OldOrderPartiallyExecuted(OrderID(9)) },
            OrderBookEvent { size: Lots(3), price: Tick(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(6), price: Tick(28), kind: OldOrderExecuted(OrderID(5)) },
            OrderBookEvent { size: Lots(3), price: Tick(28), kind: OldOrderExecuted(OrderID(7)) },
            OrderBookEvent { size: Lots(9), price: Tick(28), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(8), price: Tick(29), kind: OldOrderPartiallyExecuted(OrderID(4)) },
            OrderBookEvent { size: Lots(8), price: Tick(29), kind: NewOrderExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Tick(29),
                    vec![
                        (Lots(118), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(27))  // Big dummy order remains
}

#[test]
fn test_insert_real_buy_market_order_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_market_order::<false, true>(&mut order_book, Lots(1000)),
        [
            OrderBookEvent { size: Lots(3), price: Tick(27), kind: OldOrderExecuted(OrderID(0)) },
            OrderBookEvent { size: Lots(997), price: Tick(27), kind: OldOrderPartiallyExecuted(OrderID(9)) },
            OrderBookEvent { size: Lots(3), price: Tick(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(6), price: Tick(28), kind: OldOrderExecuted(OrderID(5)) },
            OrderBookEvent { size: Lots(3), price: Tick(28), kind: OldOrderExecuted(OrderID(7)) },
            OrderBookEvent { size: Lots(9), price: Tick(28), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(126), price: Tick(29), kind: OldOrderExecuted(OrderID(4)) },
            OrderBookEvent { size: Lots(8), price: Tick(29), kind: OldOrderExecuted(OrderID(6)) },
            OrderBookEvent { size: Lots(134), price: Tick(29), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![],
        }
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(27))  // Big dummy order remains
}

#[test]
fn test_insert_real_buy_market_order_no_opposite_side()
{
    let mut order_book = OrderBook::new();
    default_example_bids(&mut order_book);
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_market_order::<false, true>(&mut order_book, Lots(1000)),
        [
            OrderBookEvent { size: Lots(1000), price: Tick(27), kind: OldOrderPartiallyExecuted(OrderID(9)) }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![],
        }
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(27))  // Big dummy order remains
}

#[test]
fn test_insert_dummy_sell_market_order()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_market_order::<true, false>(&mut order_book, Lots(20)),
        [
            OrderBookEvent { size: Lots(8), price: Tick(26), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(12), price: Tick(23), kind: NewOrderExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(27))
}

#[test]
fn test_insert_dummy_sell_market_order_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(27))
}

#[test]
fn test_insert_dummy_sell_market_order_no_opposite_side()
{
    let mut order_book = OrderBook::new();
    default_example_asks(&mut order_book);
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_market_order::<true, false>(&mut order_book, Lots(100)),
        []
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![],
            asks: vec![
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_ask, Tick(27))
}

#[test]
fn test_insert_dummy_buy_market_order()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_market_order::<true, true>(&mut order_book, Lots(20)),
        [
            OrderBookEvent { size: Lots(3), price: Tick(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(9), price: Tick(28), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(8), price: Tick(29), kind: NewOrderExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(27))
}

#[test]
fn test_insert_dummy_buy_market_order_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_market_order::<true, true>(&mut order_book, Lots(1000)),
        [
            OrderBookEvent { size: Lots(3), price: Tick(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(9), price: Tick(28), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(134), price: Tick(29), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(27))
}

#[test]
fn test_insert_dummy_buy_market_order_no_opposite_side()
{
    let mut order_book = OrderBook::new();
    default_example_bids(&mut order_book);
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_market_order::<true, true>(&mut order_book, Lots(1000)),
        []
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![],
        }
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(27))
}

#[test]
fn test_insert_real_sell_limit_order_bids_middle()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_limit_order::<false, false>(
            &mut order_book,
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Tick(24),
            Lots(12),
        ),
        [
            OrderBookEvent { size: Lots(8), price: Tick(26), kind: OldOrderExecuted(OrderID(2)) },
            OrderBookEvent { size: Lots(3), price: Tick(26), kind: OldOrderExecuted(OrderID(8)) },
            OrderBookEvent { size: Lots(8), price: Tick(26), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                )
            ],
            asks: vec![
                (
                    Tick(24),
                    vec![
                        (Lots(4), Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01))
                    ]
                ),
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Tick(23));
    assert_eq!(order_book.best_ask, Tick(24))
}

#[test]
fn test_insert_real_sell_limit_order_bid_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_limit_order::<false, false>(
            &mut order_book,
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Tick(23),
            Lots(78),
        ),
        [
            OrderBookEvent { size: Lots(8), price: Tick(26), kind: OldOrderExecuted(OrderID(2)) },
            OrderBookEvent { size: Lots(3), price: Tick(26), kind: OldOrderExecuted(OrderID(8)) },
            OrderBookEvent { size: Lots(8), price: Tick(26), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(4), price: Tick(23), kind: OldOrderExecuted(OrderID(1)) },
            OrderBookEvent { size: Lots(44), price: Tick(23), kind: OldOrderExecuted(OrderID(3)) },
            OrderBookEvent { size: Lots(48), price: Tick(23), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![],
            asks: vec![
                (
                    Tick(23),
                    vec![
                        (Lots(22), Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01))
                    ]
                ),
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_ask, Tick(23))
}

#[test]
fn test_insert_dummy_sell_limit_order_bids_middle()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_limit_order::<true, false>(
            &mut order_book,
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Tick(24),
            Lots(12),
        ),
        [
            OrderBookEvent { size: Lots(8), price: Tick(26), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(24))
}

#[test]
fn test_insert_dummy_sell_limit_order_bid_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_limit_order::<true, false>(
            &mut order_book,
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Tick(23),
            Lots(78),
        ),
        [
            OrderBookEvent { size: Lots(8), price: Tick(26), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(48), price: Tick(23), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(23))
}

#[test]
fn test_insert_real_buy_limit_order_bids_middle()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_limit_order::<false, true>(
            &mut order_book,
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Tick(28),
            Lots(13),
        ),
        [
            OrderBookEvent { size: Lots(3), price: Tick(27), kind: OldOrderExecuted(OrderID(0)) },
            OrderBookEvent { size: Lots(10), price: Tick(27), kind: OldOrderPartiallyExecuted(OrderID(9)) },
            OrderBookEvent { size: Lots(3), price: Tick(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(6), price: Tick(28), kind: OldOrderExecuted(OrderID(5)) },
            OrderBookEvent { size: Lots(3), price: Tick(28), kind: OldOrderExecuted(OrderID(7)) },
            OrderBookEvent { size: Lots(9), price: Tick(28), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(28),
                    vec![
                        (Lots(1), Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01))
                    ]
                ),
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Tick(28));
    assert_eq!(order_book.best_ask, Tick(27))
}

#[test]
fn test_insert_real_buy_limit_order_bid_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_limit_order::<false, true>(
            &mut order_book,
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Tick(30),
            Lots(10_000),
        ),
        [
            OrderBookEvent { size: Lots(3), price: Tick(27), kind: OldOrderExecuted(OrderID(0)) },
            OrderBookEvent { size: Lots(5535), price: Tick(27), kind: OldOrderExecuted(OrderID(9)) },
            OrderBookEvent { size: Lots(3), price: Tick(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(6), price: Tick(28), kind: OldOrderExecuted(OrderID(5)) },
            OrderBookEvent { size: Lots(3), price: Tick(28), kind: OldOrderExecuted(OrderID(7)) },
            OrderBookEvent { size: Lots(9), price: Tick(28), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(126), price: Tick(29), kind: OldOrderExecuted(OrderID(4)) },
            OrderBookEvent { size: Lots(8), price: Tick(29), kind: OldOrderExecuted(OrderID(6)) },
            OrderBookEvent { size: Lots(134), price: Tick(29), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(30),
                    vec![
                        (Lots(9854), Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01))
                    ]
                ),
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![],
        }
    );
    assert_eq!(order_book.best_bid, Tick(30))
}

#[test]
fn test_insert_dummy_buy_limit_order_bids_middle()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_limit_order::<true, true>(
            &mut order_book,
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Tick(28),
            Lots(13),
        ),
        [
            OrderBookEvent { size: Lots(3), price: Tick(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(9), price: Tick(28), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Tick(28));
    assert_eq!(order_book.best_ask, Tick(27))
}

#[test]
fn test_insert_dummy_buy_limit_order_bid_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        insert_limit_order::<true, true>(
            &mut order_book,
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Tick(30),
            Lots(10_000),
        ),
        [
            OrderBookEvent { size: Lots(3), price: Tick(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(9), price: Tick(28), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Lots(134), price: Tick(29), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(0),
        ObState {
            bids: vec![
                (
                    Tick(26),
                    vec![
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Tick(23),
                    vec![
                        (Lots(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Lots(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Tick(27),
                    vec![
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Tick(28),
                    vec![
                        (Lots(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Lots(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Tick(29),
                    vec![
                        (Lots(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Lots(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Tick(30));
    assert_eq!(order_book.best_ask, Tick(27))
}

#[test]
fn test_cancel_limit_order()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.cancel_limit_order(OrderID(7)),
        Ok((
            LimitOrder {
                id: OrderID(7),
                size: Lots(3),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14),
            },
            Sell,
            Tick(28)
        ))
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(27));
    assert_eq!(
        order_book.cancel_limit_order(OrderID(7)),
        Err(NoSuchID)
    );

    assert_eq!(
        order_book.cancel_limit_order(OrderID(0)),
        Ok((
            LimitOrder {
                id: OrderID(0),
                size: Lots(3),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00),
            },
            Sell,
            Tick(27)
        ))
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(27));
    assert_eq!(
        order_book.cancel_limit_order(OrderID(9)),
        Ok((
            LimitOrder {
                id: OrderID(9),
                size: Lots(5535),
                is_dummy: true,
                dt: Date::from_ymd(2020, 02, 04).and_hms(08, 08, 09),
            },
            Sell,
            Tick(27)
        ))
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(28));

    assert_eq!(
        order_book.cancel_limit_order(OrderID(2)),
        Ok((
            LimitOrder {
                id: OrderID(2),
                size: Lots(8),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05),
            },
            Buy,
            Tick(26)
        ))
    );
    assert_eq!(order_book.best_bid, Tick(26));
    assert_eq!(order_book.best_ask, Tick(28));

    assert_eq!(
        order_book.cancel_limit_order(OrderID(8)),
        Ok((
            LimitOrder {
                id: OrderID(8),
                size: Lots(3),
                is_dummy: true,
                dt: Date::from_ymd(2020, 02, 04).and_hms(07, 00, 00),
            },
            Buy,
            Tick(26)
        ))
    );
    assert_eq!(order_book.best_bid, Tick(23));
    assert_eq!(order_book.best_ask, Tick(28));

    assert_eq!(
        order_book.cancel_limit_order(OrderID(1)),
        Ok((
            LimitOrder {
                id: OrderID(1),
                size: Lots(4),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04),
            },
            Buy,
            Tick(23)
        ))
    );
    assert_eq!(order_book.best_bid, Tick(23));
    assert_eq!(order_book.best_ask, Tick(28));

    assert_eq!(
        order_book.cancel_limit_order(OrderID(3)),
        Ok((
            LimitOrder {
                id: OrderID(3),
                size: Lots(44),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04),
            },
            Buy,
            Tick(23)
        ))
    );
    assert_eq!(order_book.best_ask, Tick(28));

    assert_eq!(
        order_book.cancel_limit_order(OrderID(4)),
        Ok((
            LimitOrder {
                id: OrderID(4),
                size: Lots(126),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09),
            },
            Sell,
            Tick(29)
        ))
    );
    assert_eq!(order_book.best_ask, Tick(28));

    assert_eq!(
        order_book.cancel_limit_order(OrderID(6)),
        Ok((
            LimitOrder {
                id: OrderID(6),
                size: Lots(8),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11),
            },
            Sell,
            Tick(29)
        ))
    );
    assert_eq!(order_book.best_ask, Tick(28));

    assert_eq!(
        order_book.cancel_limit_order(OrderID(5)),
        Ok((
            LimitOrder {
                id: OrderID(5),
                size: Lots(6),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11),
            },
            Sell,
            Tick(28)
        ))
    );

    assert_eq!(
        order_book.cancel_limit_order(OrderID(7)),
        Err(NoSuchID)
    );

    assert_eq!(
        order_book.cancel_limit_order(OrderID(52557)),
        Err(NoSuchID)
    );
}