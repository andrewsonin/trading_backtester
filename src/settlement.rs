use {
    crate::types::DateTime,
    std::{fmt::Debug, hash::Hash},
};

pub mod concrete;

pub trait GetSettlementLag: Debug + Clone + Copy + PartialOrd + PartialEq + Ord + Eq + Hash
{
    fn get_settlement_lag(&self, transaction_dt: DateTime) -> u64;
}