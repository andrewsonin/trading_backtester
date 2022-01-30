pub use {chrono, derive_macros, derive_more, rand};

use crate::types::DateTime;

pub mod constants;
pub mod input;
pub mod queue;

#[inline]
pub fn parse_datetime(string: &str, format: &str) -> DateTime {
    DateTime::parse_from_str(string, format).unwrap_or_else(
        |err| panic!(
            "Cannot parse to DateTime: \"{string}\". Datetime format used: \"{format}\". \
            Error: {err}"
        )
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