use {
    crate::types::DateTime,
    std::{fmt::Debug, hash::Hash},
};

/// Concrete implementors of the [`GetSettlementLag`].
pub mod concrete;

/// Gets settlement lag.
pub trait GetSettlementLag: Debug + Copy + Ord + Hash
{
    /// Generates settlement lag in nanoseconds since `transaction_dt`.
    ///
    /// # Arguments
    ///
    /// * `transaction_dt` — transaction datetime.
    fn get_settlement_lag(&self, transaction_dt: DateTime) -> u64;
}