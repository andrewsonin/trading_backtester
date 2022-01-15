use {
    crate::{types::DateTime, utils::enum_dispatch},
    std::{fmt::Debug, hash::Hash},
};

pub mod concrete;

#[enum_dispatch]
pub trait GetSettlementLag: Debug + Clone + Copy + PartialOrd + PartialEq + Ord + Eq + Hash
{
    fn get_settlement_lag(&self, transaction_dt: DateTime) -> u64;
}