use {
    crate::{
        types::DateTime,
        utils::constants::*,
    },
    super::GetSettlementLag,
};

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
/// Panics upon calling `get_settlement_lag`.
pub struct VoidSettlement;

impl GetSettlementLag for VoidSettlement {
    fn get_settlement_lag(&self, _: DateTime) -> u64 {
        unreachable!("VoidSettlement::get_settlement_lag method called")
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
/// Immediate settlement.
pub struct SpotSettlement;

impl GetSettlementLag for SpotSettlement {
    fn get_settlement_lag(&self, _: DateTime) -> u64 { NOW }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
/// One minute settlement.
pub struct PreciseOneMinuteSettlement;

impl GetSettlementLag for PreciseOneMinuteSettlement {
    fn get_settlement_lag(&self, _: DateTime) -> u64 { ONE_MINUTE }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
/// One hour settlement.
pub struct PreciseOneHourSettlement;

impl GetSettlementLag for PreciseOneHourSettlement {
    fn get_settlement_lag(&self, _: DateTime) -> u64 { ONE_HOUR }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
/// 24-hour settlement.
pub struct PreciseOneDaySettlement;

impl GetSettlementLag for PreciseOneDaySettlement {
    fn get_settlement_lag(&self, _: DateTime) -> u64 { ONE_DAY }
}