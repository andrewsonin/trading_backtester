use {
    crate::types::{DateTime, Id},
    rand::Rng,
};

pub mod concrete;

pub trait Latent
{
    type OuterID: Id;
    type LatencyGenerator: LatencyGenerator<Self::OuterID>;

    fn latency_generator<RNG: Rng>(&self) -> Self::LatencyGenerator;
}

pub trait LatencyGenerator<OuterID: Id>
{
    fn outgoing_latency<RNG: Rng>(
        &mut self,
        outer_id: OuterID,
        event_dt: DateTime,
        rng: &mut RNG) -> u64;

    fn incoming_latency<RNG: Rng>(
        &mut self,
        outer_id: OuterID,
        event_dt: DateTime,
        rng: &mut RNG) -> u64;
}