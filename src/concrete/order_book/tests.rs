use crate::{
    concrete::{
        order_book::{LimitOrder, OrderBook, OrderBookEvent, OrderBookEventKind::*},
        types::{Direction::*, ObState, OrderID, Price, Size},
    },
    types::{Date, DateTime},
};

fn insert_limit_order(
    ob: &mut OrderBook,
    dt: DateTime,
    id: OrderID,
    price: Price,
    size: Size,
    bid: bool,
    dummy: bool) -> Vec<OrderBookEvent>
{
    match (bid, dummy) {
        (true, true) => { ob.insert_limit_order::<true, true>(dt, id, price, size) }
        (true, false) => { ob.insert_limit_order::<false, true>(dt, id, price, size) }
        (false, true) => { ob.insert_limit_order::<true, false>(dt, id, price, size) }
        (false, false) => { ob.insert_limit_order::<false, false>(dt, id, price, size) }
    }
}

fn default_example<const TEST: bool>() -> OrderBook
{
    let mut order_book = OrderBook::new();
    for (dt, id, price, size, bid) in [
        (Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00), OrderID(0), Price(27), Size(3), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04), OrderID(1), Price(23), Size(4), true),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05), OrderID(2), Price(26), Size(8), true),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04), OrderID(3), Price(23), Size(44), true),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09), OrderID(4), Price(29), Size(126), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11), OrderID(5), Price(28), Size(6), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11), OrderID(6), Price(29), Size(8), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14), OrderID(7), Price(28), Size(3), false),
    ] {
        let response = insert_limit_order(&mut order_book, dt, id, price, size, bid, false);
        if TEST {
            assert_eq!(response, [])
        }
    }
    order_book
}

fn default_example_bids(order_book: &mut OrderBook)
{
    for (dt, id, price, size, bid) in [
        (Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04), OrderID(1), Price(23), Size(4), true),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05), OrderID(2), Price(26), Size(8), true),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04), OrderID(3), Price(23), Size(44), true),
    ] {
        insert_limit_order(order_book, dt, id, price, size, bid, false);
    }
}

fn default_example_asks(order_book: &mut OrderBook)
{
    for (dt, id, price, size, bid) in [
        (Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00), OrderID(0), Price(27), Size(3), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09), OrderID(4), Price(29), Size(126), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11), OrderID(5), Price(28), Size(6), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11), OrderID(6), Price(29), Size(8), false),
        (Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14), OrderID(7), Price(28), Size(3), false),
    ] {
        insert_limit_order(order_book, dt, id, price, size, bid, false);
    }
}

fn default_example_dummies(order_book: &mut OrderBook)
{
    for (dt, id, price, size, bid) in [
        (Date::from_ymd(2020, 02, 04).and_hms(07, 00, 00), OrderID(8), Price(26), Size(3), true),
        (Date::from_ymd(2020, 02, 04).and_hms(08, 08, 09), OrderID(9), Price(27), Size(5535), false),
    ] {
        insert_limit_order(order_book, dt, id, price, size, bid, true);
    }
}

#[test]
fn test_default_example()
{
    let order_book = default_example::<true>();
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(27));
}

#[test]
fn test_default_example_bids()
{
    let mut order_book = OrderBook::new();
    default_example_bids(&mut order_book);
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![],
        }
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(0))
}

#[test]
fn test_default_example_asks()
{
    let mut order_book = OrderBook::new();
    default_example_asks(&mut order_book);
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![],
            asks: vec![
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Price(0));
    assert_eq!(order_book.best_ask, Price(27))
}

#[test]
fn test_default_example_equivalence()
{
    let mut order_book1 = OrderBook::new();
    default_example_asks(&mut order_book1);
    default_example_bids(&mut order_book1);
    let order_book2 = default_example::<false>();
    assert_eq!(order_book1.get_ob_state(), order_book2.get_ob_state())
}

#[test]
fn test_default_example_dummies()
{
    let mut order_book_with_dummies = default_example::<false>();
    default_example_dummies(&mut order_book_with_dummies);
    let order_book = default_example::<false>();
    assert_eq!(order_book.get_ob_state(), order_book_with_dummies.get_ob_state())
}

#[test]
fn test_clear()
{
    let mut order_book = default_example::<false>();
    order_book.clear();
    assert_eq!(order_book.get_ob_state(), ObState { bids: vec![], asks: vec![] })
}

#[test]
fn test_insert_real_sell_market_order()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_market_order::<false, false>(Size(20)),
        [
            OrderBookEvent { size: Size(8), price: Price(26), kind: OldOrderExecuted(OrderID(2)) },
            OrderBookEvent { size: Size(3), price: Price(26), kind: OldOrderExecuted(OrderID(8)) },
            OrderBookEvent { size: Size(8), price: Price(26), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(4), price: Price(23), kind: OldOrderExecuted(OrderID(1)) },
            OrderBookEvent { size: Size(8), price: Price(23), kind: OldOrderPartiallyExecuted(OrderID(3)) },
            OrderBookEvent { size: Size(12), price: Price(23), kind: NewOrderExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(23),
                    vec![
                        (Size(36), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04))
                    ]
                )
            ],
            asks: vec![
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Price(23));
    assert_eq!(order_book.best_ask, Price(27))
}

#[test]
fn test_insert_real_sell_market_order_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_market_order::<false, false>(Size(100)),
        [
            OrderBookEvent { size: Size(8), price: Price(26), kind: OldOrderExecuted(OrderID(2)) },
            OrderBookEvent { size: Size(3), price: Price(26), kind: OldOrderExecuted(OrderID(8)) },
            OrderBookEvent { size: Size(8), price: Price(26), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(4), price: Price(23), kind: OldOrderExecuted(OrderID(1)) },
            OrderBookEvent { size: Size(44), price: Price(23), kind: OldOrderExecuted(OrderID(3)) },
            OrderBookEvent { size: Size(48), price: Price(23), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![],
            asks: vec![
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_ask, Price(27))
}

#[test]
fn test_insert_real_sell_market_order_no_opposite_side()
{
    let mut order_book = OrderBook::new();
    default_example_asks(&mut order_book);
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_market_order::<false, false>(Size(100)),
        [
            OrderBookEvent { size: Size(3), price: Price(26), kind: OldOrderExecuted(OrderID(8)) }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![],
            asks: vec![
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_ask, Price(27))
}

#[test]
fn test_insert_real_buy_market_order()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_market_order::<false, true>(Size(20)),
        [
            OrderBookEvent { size: Size(3), price: Price(27), kind: OldOrderExecuted(OrderID(0)) },
            OrderBookEvent { size: Size(17), price: Price(27), kind: OldOrderPartiallyExecuted(OrderID(9)) },
            OrderBookEvent { size: Size(3), price: Price(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(6), price: Price(28), kind: OldOrderExecuted(OrderID(5)) },
            OrderBookEvent { size: Size(3), price: Price(28), kind: OldOrderExecuted(OrderID(7)) },
            OrderBookEvent { size: Size(9), price: Price(28), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(8), price: Price(29), kind: OldOrderPartiallyExecuted(OrderID(4)) },
            OrderBookEvent { size: Size(8), price: Price(29), kind: NewOrderExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Price(29),
                    vec![
                        (Size(118), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(27))  // Big dummy order remains
}

#[test]
fn test_insert_real_buy_market_order_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_market_order::<false, true>(Size(1000)),
        [
            OrderBookEvent { size: Size(3), price: Price(27), kind: OldOrderExecuted(OrderID(0)) },
            OrderBookEvent { size: Size(997), price: Price(27), kind: OldOrderPartiallyExecuted(OrderID(9)) },
            OrderBookEvent { size: Size(3), price: Price(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(6), price: Price(28), kind: OldOrderExecuted(OrderID(5)) },
            OrderBookEvent { size: Size(3), price: Price(28), kind: OldOrderExecuted(OrderID(7)) },
            OrderBookEvent { size: Size(9), price: Price(28), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(126), price: Price(29), kind: OldOrderExecuted(OrderID(4)) },
            OrderBookEvent { size: Size(8), price: Price(29), kind: OldOrderExecuted(OrderID(6)) },
            OrderBookEvent { size: Size(134), price: Price(29), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![],
        }
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(27))  // Big dummy order remains
}

#[test]
fn test_insert_real_buy_market_order_no_opposite_side()
{
    let mut order_book = OrderBook::new();
    default_example_bids(&mut order_book);
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_market_order::<false, true>(Size(1000)),
        [
            OrderBookEvent { size: Size(1000), price: Price(27), kind: OldOrderPartiallyExecuted(OrderID(9)) }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![],
        }
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(27))  // Big dummy order remains
}

#[test]
fn test_insert_dummy_sell_market_order()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_market_order::<true, false>(Size(20)),
        [
            OrderBookEvent { size: Size(8), price: Price(26), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(12), price: Price(23), kind: NewOrderExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(27))
}

#[test]
fn test_insert_dummy_sell_market_order_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(27))
}

#[test]
fn test_insert_dummy_sell_market_order_no_opposite_side()
{
    let mut order_book = OrderBook::new();
    default_example_asks(&mut order_book);
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_market_order::<true, false>(Size(100)),
        []
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![],
            asks: vec![
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_ask, Price(27))
}

#[test]
fn test_insert_dummy_buy_market_order()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_market_order::<true, true>(Size(20)),
        [
            OrderBookEvent { size: Size(3), price: Price(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(9), price: Price(28), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(8), price: Price(29), kind: NewOrderExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(27))
}

#[test]
fn test_insert_dummy_buy_market_order_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_market_order::<true, true>(Size(1000)),
        [
            OrderBookEvent { size: Size(3), price: Price(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(9), price: Price(28), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(134), price: Price(29), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(27))
}

#[test]
fn test_insert_dummy_buy_market_order_no_opposite_side()
{
    let mut order_book = OrderBook::new();
    default_example_bids(&mut order_book);
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_market_order::<true, true>(Size(1000)),
        []
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![],
        }
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(27))
}

#[test]
fn test_insert_real_sell_limit_order_bids_middle()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_limit_order::<false, false>(
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Price(24),
            Size(12),
        ),
        [
            OrderBookEvent { size: Size(8), price: Price(26), kind: OldOrderExecuted(OrderID(2)) },
            OrderBookEvent { size: Size(3), price: Price(26), kind: OldOrderExecuted(OrderID(8)) },
            OrderBookEvent { size: Size(8), price: Price(26), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                )
            ],
            asks: vec![
                (
                    Price(24),
                    vec![
                        (Size(4), Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01))
                    ]
                ),
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Price(23));
    assert_eq!(order_book.best_ask, Price(24))
}

#[test]
fn test_insert_real_sell_limit_order_bid_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_limit_order::<false, false>(
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Price(23),
            Size(78),
        ),
        [
            OrderBookEvent { size: Size(8), price: Price(26), kind: OldOrderExecuted(OrderID(2)) },
            OrderBookEvent { size: Size(3), price: Price(26), kind: OldOrderExecuted(OrderID(8)) },
            OrderBookEvent { size: Size(8), price: Price(26), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(4), price: Price(23), kind: OldOrderExecuted(OrderID(1)) },
            OrderBookEvent { size: Size(44), price: Price(23), kind: OldOrderExecuted(OrderID(3)) },
            OrderBookEvent { size: Size(48), price: Price(23), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![],
            asks: vec![
                (
                    Price(23),
                    vec![
                        (Size(22), Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01))
                    ]
                ),
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_ask, Price(23))
}

#[test]
fn test_insert_dummy_sell_limit_order_bids_middle()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_limit_order::<true, false>(
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Price(24),
            Size(12),
        ),
        [
            OrderBookEvent { size: Size(8), price: Price(26), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(24))
}

#[test]
fn test_insert_dummy_sell_limit_order_bid_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_limit_order::<true, false>(
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Price(23),
            Size(78),
        ),
        [
            OrderBookEvent { size: Size(8), price: Price(26), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(48), price: Price(23), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(23))
}

#[test]
fn test_insert_real_buy_limit_order_bids_middle()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_limit_order::<false, true>(
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Price(28),
            Size(13),
        ),
        [
            OrderBookEvent { size: Size(3), price: Price(27), kind: OldOrderExecuted(OrderID(0)) },
            OrderBookEvent { size: Size(10), price: Price(27), kind: OldOrderPartiallyExecuted(OrderID(9)) },
            OrderBookEvent { size: Size(3), price: Price(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(6), price: Price(28), kind: OldOrderExecuted(OrderID(5)) },
            OrderBookEvent { size: Size(3), price: Price(28), kind: OldOrderExecuted(OrderID(7)) },
            OrderBookEvent { size: Size(9), price: Price(28), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(28),
                    vec![
                        (Size(1), Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01))
                    ]
                ),
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Price(28));
    assert_eq!(order_book.best_ask, Price(27))
}

#[test]
fn test_insert_real_buy_limit_order_bid_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_limit_order::<false, true>(
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Price(30),
            Size(10_000),
        ),
        [
            OrderBookEvent { size: Size(3), price: Price(27), kind: OldOrderExecuted(OrderID(0)) },
            OrderBookEvent { size: Size(5535), price: Price(27), kind: OldOrderExecuted(OrderID(9)) },
            OrderBookEvent { size: Size(3), price: Price(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(6), price: Price(28), kind: OldOrderExecuted(OrderID(5)) },
            OrderBookEvent { size: Size(3), price: Price(28), kind: OldOrderExecuted(OrderID(7)) },
            OrderBookEvent { size: Size(9), price: Price(28), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(126), price: Price(29), kind: OldOrderExecuted(OrderID(4)) },
            OrderBookEvent { size: Size(8), price: Price(29), kind: OldOrderExecuted(OrderID(6)) },
            OrderBookEvent { size: Size(134), price: Price(29), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(30),
                    vec![
                        (Size(9854), Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01))
                    ]
                ),
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![],
        }
    );
    assert_eq!(order_book.best_bid, Price(30))
}

#[test]
fn test_insert_dummy_buy_limit_order_bids_middle()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_limit_order::<true, true>(
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Price(28),
            Size(13),
        ),
        [
            OrderBookEvent { size: Size(3), price: Price(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(9), price: Price(28), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Price(28));
    assert_eq!(order_book.best_ask, Price(27))
}

#[test]
fn test_insert_dummy_buy_limit_order_bid_overflow()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.insert_limit_order::<true, true>(
            Date::from_ymd(2021, 01, 01).and_hms(01, 01, 01),
            OrderID(10), Price(30),
            Size(10_000),
        ),
        [
            OrderBookEvent { size: Size(3), price: Price(27), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(9), price: Price(28), kind: NewOrderPartiallyExecuted },
            OrderBookEvent { size: Size(134), price: Price(29), kind: NewOrderPartiallyExecuted }
        ]
    );
    assert_eq!(
        order_book.get_ob_state(),
        ObState {
            bids: vec![
                (
                    Price(26),
                    vec![
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05))
                    ]
                ),
                (
                    Price(23),
                    vec![
                        (Size(4), Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04)),
                        (Size(44), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04)),
                    ]
                ),
            ],
            asks: vec![
                (
                    Price(27),
                    vec![
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00))
                    ]
                ),
                (
                    Price(28),
                    vec![
                        (Size(6), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                        (Size(3), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14)),
                    ]
                ),
                (
                    Price(29),
                    vec![
                        (Size(126), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09)),
                        (Size(8), Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11)),
                    ]
                ),
            ],
        }
    );
    assert_eq!(order_book.best_bid, Price(30));
    assert_eq!(order_book.best_ask, Price(27))
}

#[test]
fn test_cancel_limit_order()
{
    let mut order_book = default_example::<false>();
    default_example_dummies(&mut order_book);

    assert_eq!(
        order_book.cancel_limit_order(OrderID(7)),
        Some((
            LimitOrder {
                id: OrderID(7),
                size: Size(3),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(12, 08, 14),
            },
            Sell,
            Price(28)
        ))
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(27));
    assert_eq!(
        order_book.cancel_limit_order(OrderID(7)),
        None
    );

    assert_eq!(
        order_book.cancel_limit_order(OrderID(0)),
        Some((
            LimitOrder {
                id: OrderID(0),
                size: Size(3),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(07, 00, 00),
            },
            Sell,
            Price(27)
        ))
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(27));
    assert_eq!(
        order_book.cancel_limit_order(OrderID(9)),
        Some((
            LimitOrder {
                id: OrderID(9),
                size: Size(5535),
                is_dummy: true,
                dt: Date::from_ymd(2020, 02, 04).and_hms(08, 08, 09),
            },
            Sell,
            Price(27)
        ))
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(28));

    assert_eq!(
        order_book.cancel_limit_order(OrderID(2)),
        Some((
            LimitOrder {
                id: OrderID(2),
                size: Size(8),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(12, 03, 05),
            },
            Buy,
            Price(26)
        ))
    );
    assert_eq!(order_book.best_bid, Price(26));
    assert_eq!(order_book.best_ask, Price(28));

    assert_eq!(
        order_book.cancel_limit_order(OrderID(8)),
        Some((
            LimitOrder {
                id: OrderID(8),
                size: Size(3),
                is_dummy: true,
                dt: Date::from_ymd(2020, 02, 04).and_hms(07, 00, 00),
            },
            Buy,
            Price(26)
        ))
    );
    assert_eq!(order_book.best_bid, Price(23));
    assert_eq!(order_book.best_ask, Price(28));

    assert_eq!(
        order_book.cancel_limit_order(OrderID(1)),
        Some((
            LimitOrder {
                id: OrderID(1),
                size: Size(4),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(12, 03, 04),
            },
            Buy,
            Price(23)
        ))
    );
    assert_eq!(order_book.best_bid, Price(23));
    assert_eq!(order_book.best_ask, Price(28));

    assert_eq!(
        order_book.cancel_limit_order(OrderID(3)),
        Some((
            LimitOrder {
                id: OrderID(3),
                size: Size(44),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(12, 08, 04),
            },
            Buy,
            Price(23)
        ))
    );
    assert_eq!(order_book.best_ask, Price(28));

    assert_eq!(
        order_book.cancel_limit_order(OrderID(4)),
        Some((
            LimitOrder {
                id: OrderID(4),
                size: Size(126),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(12, 08, 09),
            },
            Sell,
            Price(29)
        ))
    );
    assert_eq!(order_book.best_ask, Price(28));

    assert_eq!(
        order_book.cancel_limit_order(OrderID(6)),
        Some((
            LimitOrder {
                id: OrderID(6),
                size: Size(8),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11),
            },
            Sell,
            Price(29)
        ))
    );
    assert_eq!(order_book.best_ask, Price(28));

    assert_eq!(
        order_book.cancel_limit_order(OrderID(5)),
        Some((
            LimitOrder {
                id: OrderID(5),
                size: Size(6),
                is_dummy: false,
                dt: Date::from_ymd(2020, 02, 03).and_hms(12, 08, 11),
            },
            Sell,
            Price(28)
        ))
    );

    assert_eq!(
        order_book.cancel_limit_order(OrderID(7)),
        None
    );

    assert_eq!(
        order_book.cancel_limit_order(OrderID(52557)),
        None
    );
}