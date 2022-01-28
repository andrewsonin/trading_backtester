use {
    crate::types::{DateTime, Id},
    rand::Rng,
};

pub mod concrete;

pub trait Latent
{
    type OuterID: Id;
    type LatencyGenerator: LatencyGenerator<Self::OuterID>;

    fn get_latency_generator(&self) -> Self::LatencyGenerator;
}

pub trait LatencyGenerator<OuterID: Id>
{
    fn outgoing_latency(
        &mut self,
        outer_id: OuterID,
        event_dt: DateTime,
        rng: &mut impl Rng) -> u64;

    fn incoming_latency(
        &mut self,
        outer_id: OuterID,
        event_dt: DateTime,
        rng: &mut impl Rng) -> u64;
}