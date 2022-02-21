use {
    crate::{interface::latency::LatencyGenerator, types::{DateTime, Id}},
    rand::Rng,
    std::marker::PhantomData,
};

/// Constant [`LatencyGenerator`].
#[derive(Copy, Clone, Default)]
pub struct ConstantLatency<OuterID: Id, const OUTGOING: u64, const INCOMING: u64>
(PhantomData<OuterID>);

impl<OuterID: Id, const OUTGOING: u64, const INCOMING: u64>
ConstantLatency<OuterID, OUTGOING, INCOMING>
{
    /// Creates a new instance of the `ConstantLatency`.
    pub fn new() -> Self {
        ConstantLatency(PhantomData::default())
    }
}

impl<OuterID: Id, const OUTGOING: u64, const INCOMING: u64>
LatencyGenerator
for ConstantLatency<OuterID, OUTGOING, INCOMING>
{
    type OuterID = OuterID;

    fn outgoing_latency(&mut self, _: Self::OuterID, _: DateTime, _: &mut impl Rng) -> u64 {
        OUTGOING
    }
    fn incoming_latency(&mut self, _: Self::OuterID, _: DateTime, _: &mut impl Rng) -> u64 {
        INCOMING
    }
}