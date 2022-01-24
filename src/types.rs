pub use chrono::{
    Duration,
    NaiveDate as Date,
    NaiveDateTime as DateTime,
    NaiveTime as Time,
    Timelike,
};

use {
    crate::{
        broker::{BrokerToExchange, BrokerToItself, BrokerToTrader},
        exchange::{ExchangeToBroker, ExchangeToItself, ExchangeToReplay},
        replay::{ReplayToExchange, ReplayToItself},
        trader::{TraderToBroker, TraderToItself},
        utils::{
            derive_more,
            derive_more::{Add, AddAssign, From, FromStr, Into, Sub, SubAssign, Sum},
        },
    },
    std::{
        cmp::Ordering,
        convert::Infallible,
        fmt::{Debug, Display},
        hash::Hash,
        marker::PhantomData,
        str::FromStr,
    },
};

pub trait Id: Hash + Ord + Copy + Send + Sync + Display + Debug {}

impl<T: Hash + Ord + Copy + Send + Sync + Display + Debug> Id for T {}

pub trait Named<Name: Id> {
    fn get_name(&self) -> Name;
}

pub trait TimeSync {
    fn current_datetime_mut(&mut self) -> &mut DateTime;
}

#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy)]
#[derive(derive_more::Display, FromStr, Add, Sub, AddAssign, SubAssign, From, Into)]
pub struct OrderID(pub u64);

#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy)]
#[derive(derive_more::Display, Add, Sub, AddAssign, SubAssign, From, Into)]
pub struct Price(pub i64);

#[derive(derive_more::Display, FromStr, Debug, PartialOrd, Clone, Copy, From, Into)]
pub struct PriceStep(pub f64);

#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy)]
#[derive(derive_more::Display, FromStr, Add, Sub, AddAssign, SubAssign, Sum, From, Into)]
pub struct Size(pub i64);

#[derive(derive_more::Display, Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum Direction {
    Buy,
    Sell,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ObState {
    pub bids: Vec<(Price, Vec<(Size, DateTime)>)>,
    pub asks: Vec<(Price, Vec<(Size, DateTime)>)>,
}

const ACCEPTABLE_PRECISION_ERROR: f64 = 1e-11;

impl Price
{
    pub fn from_decimal_str(string: &str, price_step: PriceStep) -> Self
    {
        let parsed_f64 = f64::from_str(string).unwrap_or_else(
            |err| panic!("Cannot parse to f64: {string}. Error: {err}")
        );
        Self::from_f64(parsed_f64, price_step)
    }

    pub fn from_f64(value: f64, price_step: PriceStep) -> Self {
        let price_steps = value / price_step.0;
        let rounded_price_steps = price_steps.round();
        if (rounded_price_steps - price_steps).abs() < ACCEPTABLE_PRECISION_ERROR {
            Price(rounded_price_steps as i64)
        } else {
            panic!(
                "Cannot convert f64 {value} to Price without loss of precision \
                with the following price step: {price_step}"
            )
        }
    }

    pub fn to_f64(&self, price_step: PriceStep) -> f64 {
        self.0 as f64 * price_step.0
    }
}

impl PartialEq for PriceStep {
    fn eq(&self, other: &Self) -> bool {
        let diff = self.0 - other.0;
        diff.abs() < ACCEPTABLE_PRECISION_ERROR
    }
}

impl Eq for PriceStep {}

impl Ord for PriceStep {
    fn cmp(&self, other: &Self) -> Ordering {
        if self < other {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

pub type Nothing = Infallible;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NeverType<K>((Infallible, PhantomData<K>));

impl BrokerToItself for Nothing {}

impl<ExchangeID: Id> BrokerToExchange for NeverType<ExchangeID> {
    type ExchangeID = ExchangeID;

    fn get_exchange_id(&self) -> Self::ExchangeID {
        unreachable!("Does not contain ExchangeID")
    }
}

impl<TraderID: Id> BrokerToTrader for NeverType<TraderID> {
    type TraderID = TraderID;

    fn get_trader_id(&self) -> Self::TraderID {
        unreachable!("Does not contain TraderID")
    }
}

impl ExchangeToItself for Nothing {}

impl ExchangeToReplay for Nothing {}

impl<BrokerID: Id> ExchangeToBroker for NeverType<BrokerID> {
    type BrokerID = BrokerID;

    fn get_broker_id(&self) -> Self::BrokerID {
        unreachable!("Does not contain BrokerID")
    }
}

impl ReplayToItself for Nothing {}

impl<ExchangeID: Id> ReplayToExchange for NeverType<ExchangeID> {
    type ExchangeID = ExchangeID;

    fn get_exchange_id(&self) -> ExchangeID {
        unreachable!("Does not contain ExchangeID")
    }
}

impl TraderToItself for Nothing {}

impl<BrokerID: Id> TraderToBroker for NeverType<BrokerID> {
    type BrokerID = BrokerID;

    fn get_broker_id(&self) -> BrokerID {
        unreachable!("Does not contain BrokerID")
    }
}