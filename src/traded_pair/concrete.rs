use crate::{
    settlement::concrete::SpotSettlement,
    traded_pair::Base,
};

pub const SPOT_BASE: Base<SpotSettlement> = Base { delivery: SpotSettlement };