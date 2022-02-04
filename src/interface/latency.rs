use {crate::types::{DateTime, Id}, rand::Rng};

pub trait Latent
{
    type OuterID: Id;
    type LatencyGenerator: LatencyGenerator<OuterID=Self::OuterID>;

    fn get_latency_generator(&self) -> Self::LatencyGenerator;
}

pub trait LatencyGenerator
{
    type OuterID: Id;

    fn outgoing_latency(
        &mut self,
        outer_id: Self::OuterID,
        event_dt: DateTime,
        rng: &mut impl Rng) -> u64;

    fn incoming_latency(
        &mut self,
        outer_id: Self::OuterID,
        event_dt: DateTime,
        rng: &mut impl Rng) -> u64;
}