use {
    crate::types::DateTime,
    std::{fmt::Debug, hash::Hash},
};

pub mod concrete;

pub trait GetSettlementLag: Debug + Copy + Ord + Hash
{
    fn get_settlement_lag(&self, transaction_dt: DateTime) -> u64;
}