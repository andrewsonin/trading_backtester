use {
    crate::{
        latency::LatencyGenerator,
        types::{DateTime, Id},
    },
    rand::Rng,
};

pub struct ConstantLatency<const OUTGOING: u64, const INCOMING: u64>;

impl<OuterID: Id, const OUTGOING: u64, const INCOMING: u64>
LatencyGenerator<OuterID>
for ConstantLatency<OUTGOING, INCOMING>
{
    fn outgoing_latency<RNG: Rng>(&mut self, _: OuterID, _: DateTime, _: &mut RNG) -> u64 {
        OUTGOING
    }
    fn incoming_latency<RNG: Rng>(&mut self, _: OuterID, _: DateTime, _: &mut RNG) -> u64 {
        INCOMING
    }
}