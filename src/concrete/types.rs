use {
    chrono::NaiveDateTime as DateTime,
    derive_more::{Add, AddAssign, From, FromStr, Into, Sub, SubAssign, Sum},
    std::{cmp::Ordering, str::FromStr},
};

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
    #[inline]
    pub fn from_decimal_str(string: &str, price_step: PriceStep) -> Self
    {
        let parsed_f64 = f64::from_str(string).unwrap_or_else(
            |err| panic!("Cannot parse to f64: {string}. Error: {err}")
        );
        Self::from_f64(parsed_f64, price_step)
    }

    #[inline]
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

    #[inline]
    pub fn to_f64(&self, price_step: PriceStep) -> f64 {
        self.0 as f64 * price_step.0
    }
}

impl PartialEq for PriceStep {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let diff = self.0 - other.0;
        diff.abs() < ACCEPTABLE_PRECISION_ERROR
    }
}

impl Eq for PriceStep {}

impl Ord for PriceStep {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        if self < other {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}