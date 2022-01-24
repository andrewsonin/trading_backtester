pub use {derive_macros, derive_more, rand};

use crate::types::DateTime;

pub mod queue;
pub mod input;
pub mod constants;

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
        || panic!("Cannot parse to DateTime: \"{string}\". Datetime format used: \"{format}\"")
    )
}

#[macro_export]
macro_rules! enum_def {
    (
        $(#[$meta:meta])*
        $vis:vis
        $name:ident $(     < $(   $type:tt $( :   $bound:tt $(+   $other_bounds:tt )* )? ),+ >)?
                    $( where $( $w_type:tt $( : $w_bound:path )? ),+ )?
        {
            $($var_name:ident $(< $( $var_type:path ),+ >)?),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis
        enum $name $(     < $(   $type $( :   $bound $(+   $other_bounds )* )? ),+ >)?
                   $( where $( $w_type $( : $w_bound )? ),+ )?
        {
            $($var_name ($var_name $(< $( $var_type ),+ >)?) ),+
        }
    }
}