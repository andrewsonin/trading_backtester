pub use enum_dispatch::enum_dispatch;

use {crate::types::DateTime, std::panic::panic_any};

pub mod queue;
pub mod input;

pub trait ExpectWith<T, F>
    where F: Fn() -> String
{
    fn expect_with(self, f: F) -> T;
}

impl<T, F> ExpectWith<T, F> for Option<T>
    where F: Fn() -> String
{
    fn expect_with(self, f: F) -> T {
        if let Some(v) = self {
            v
        } else {
            panic_any(f())
        }
    }
}

impl<T, F, E> ExpectWith<T, F> for Result<T, E>
    where F: Fn() -> String
{
    fn expect_with(self, f: F) -> T {
        if let Ok(v) = self {
            v
        } else {
            panic_any(f())
        }
    }
}

pub fn parse_datetime(string: &str, format: &str) -> DateTime {
    DateTime::parse_from_str(string, format).expect_with(
        || panic!("Cannot parse to DateTime: \"{}\". Datetime format used: \"{}\"", string, format)
    )
}