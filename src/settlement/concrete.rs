use crate::{
    settlement::GetSettlementLag,
    types::DateTime,
    utils::{constants::*},
};

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct VoidSettlement;

impl GetSettlementLag for VoidSettlement {
    fn get_settlement_lag(&self, _: DateTime) -> u64 {
        unreachable!("VoidSettlement::get_settlement_dt method called")
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct SpotSettlement;

impl GetSettlementLag for SpotSettlement {
    fn get_settlement_lag(&self, _: DateTime) -> u64 { NOW }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct PreciseOneMinuteSettlement;

impl GetSettlementLag for PreciseOneMinuteSettlement {
    fn get_settlement_lag(&self, _: DateTime) -> u64 { ONE_MINUTE }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct PreciseOneHourSettlement;

impl GetSettlementLag for PreciseOneHourSettlement {
    fn get_settlement_lag(&self, _: DateTime) -> u64 { ONE_HOUR }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct PreciseOneDaySettlement;

impl GetSettlementLag for PreciseOneDaySettlement {
    fn get_settlement_lag(&self, _: DateTime) -> u64 { ONE_DAY }
}