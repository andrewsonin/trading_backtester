#[cfg(feature = "derive_more")]
pub use derive_more;

pub use {chrono, rand};
#[cfg(feature = "derive")]
pub use derive;

/// Useful constants.
pub mod constants;
/// Useful queue structures.
pub mod queue;

#[cfg(feature = "enum_def")]
#[macro_export]
/// Macro that generates an `enum` that can contain
/// each of the listed types as a unique `enum` variant.
///
/// # Examples
///
/// ```
/// use trading_backtester::enum_def;
///
/// enum_def! {
///     #[derive(Clone, Ord, Eq, PartialEq, PartialOrd)]
///     pub CustomEnum<M: Ord> where M: Copy {
///         String,
///         Option<M>
///     }
/// }
///
/// // Is equivalent to the following
/// #[derive(Clone, Ord, Eq, PartialEq, PartialOrd)]
/// pub enum AnotherCustomEnum<M: Ord> where M: Copy {
///     String(String),
///     Option(Option<M>),
/// }
/// ```
macro_rules! enum_def {
    (
        $(#[$meta:meta])*
        $vis:vis
        $name:ident $(     < $(   $type:tt $( :   $bound:tt $(+   $other_bounds:tt )* )? ),+ >)?
                    $( where $( $w_type:tt $( : $w_bound:path )? ),+ )?
        {
            $( $(#[$inner:ident $($args:tt)*])* $var_name:ident $(< $( $var_type:path ),+ >)?),+
            $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis
        enum $name $(     < $(   $type $( :   $bound $(+   $other_bounds )* )? ),+ >)?
                   $( where $( $w_type $( : $w_bound )? ),+ )?
        {
            $( $(#[$inner $($args)*])* $var_name ($var_name $(< $( $var_type ),+ >)?) ),+
        }
    }
}