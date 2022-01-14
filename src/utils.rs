pub use {enum_dispatch::enum_dispatch, rand};

use crate::types::DateTime;

pub mod queue;
pub mod input;

pub trait ExpectWith<T> {
    fn expect_with(self, get_err_msg: impl Fn() -> String) -> T;
}

impl<T> ExpectWith<T> for Option<T>
{
    fn expect_with(self, get_err_msg: impl Fn() -> String) -> T {
        if let Some(v) = self {
            v
        } else {
            panic!("{}", get_err_msg())
        }
    }
}

impl<T, E> ExpectWith<T> for Result<T, E>
{
    fn expect_with(self, get_err_msg: impl Fn() -> String) -> T {
        if let Ok(v) = self {
            v
        } else {
            panic!("{}", get_err_msg())
        }
    }
}

pub fn parse_datetime(string: &str, format: &str) -> DateTime {
    DateTime::parse_from_str(string, format).expect_with(
        || panic!("Cannot parse to DateTime: \"{}\". Datetime format used: \"{}\"", string, format)
    )
}