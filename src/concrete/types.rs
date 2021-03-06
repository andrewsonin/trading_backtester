use {
    chrono::NaiveDateTime as DateTime,
    derive_more::{Add, AddAssign, From, FromStr, Into, Sub, SubAssign, Sum},
    std::{cmp::Ordering, str::FromStr},
};

#[derive(Debug, Default, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy)]
#[derive(derive_more::Display, FromStr, Add, Sub, AddAssign, SubAssign, From, Into)]
/// Order ID newtype.
pub struct OrderID(pub u64);

#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy)]
#[derive(derive_more::Display, Add, Sub, AddAssign, SubAssign, From, Into)]
/// Quotation tick newtype. Is equivalent to the [`i64`] due to the fact that
/// exchanges quote prices with a certain constant step.
pub struct Tick(pub i64);

#[derive(derive_more::Display, FromStr, Debug, PartialOrd, Clone, Copy, From, Into)]
/// Tick size newtype. Price quotation step.
pub struct TickSize(pub f64);

#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy)]
#[derive(derive_more::Display, FromStr, Add, Sub, AddAssign, SubAssign, Sum, From, Into)]
/// Order size newtype.
pub struct Lots(pub i64);

#[derive(derive_more::Display, Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
/// Order Direction.
pub enum Direction {
    /// Buy direction.
    Buy,
    /// Sell direction.
    Sell,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Order book state.
pub struct ObState {
    pub bids: Vec<(Tick, Vec<(Lots, DateTime)>)>,
    pub asks: Vec<(Tick, Vec<(Lots, DateTime)>)>,
}

/// Acceptable precision error during conversions between [`f64`] and [`Price`].
const ACCEPTABLE_PRECISION_ERROR: f64 = 1e-11;

impl Tick
{
    /// Converts string to [`Price`].
    ///
    /// # Arguments
    ///
    /// * `string` — String to convert.
    /// * `price_step` — Price quotation step.
    pub fn from_decimal_str(string: impl AsRef<str>, price_step: TickSize) -> Self
    {
        let string = string.as_ref();
        let parsed_f64 = f64::from_str(string).unwrap_or_else(
            |err| panic!("Cannot parse to f64: {string}. Error: {err}")
        );
        Self::from_f64(parsed_f64, price_step)
    }

    #[inline]
    /// Converts [`f64`] to [`Price`].
    ///
    /// # Arguments
    ///
    /// * `value` — Value to convert.
    /// * `price_step` — Price quotation step.
    pub fn from_f64(value: f64, price_step: TickSize) -> Self {
        let price_steps = value / price_step.0;
        let rounded_price_steps = price_steps.round();
        if (rounded_price_steps - price_steps).abs() < ACCEPTABLE_PRECISION_ERROR {
            Tick(rounded_price_steps as i64)
        } else {
            panic!(
                "Cannot convert f64 {value} to Price without loss of precision \
                with the following price step: {price_step}"
            )
        }
    }

    #[inline]
    /// Converts [`Price`] to [`f64`].
    ///
    /// # Arguments
    ///
    /// * `price_step` — Price quotation step.
    pub fn to_f64(&self, price_step: TickSize) -> f64 {
        self.0 as f64 * price_step.0
    }
}

impl From<Tick> for isize {
    fn from(price: Tick) -> Self {
        price.0 as isize
    }
}

impl PartialEq for TickSize {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let diff = self.0 - other.0;
        diff.abs() < ACCEPTABLE_PRECISION_ERROR
    }
}

impl Eq for TickSize {}

impl Ord for TickSize {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        if self < other {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}