use {crate::types::{DateTime, Id}, rand::Rng};

/// Is implemented for latent agents:
/// that is, for those who may have an unintentional delay
/// in the delivery of messages due to technical or natural reasons.
pub trait Latent
{
    /// The ID type of the agent to which the `Self` agent connects.
    /// This will be `ExchangeID` for [`Broker`](crate::interface::broker::Broker)
    /// or `BrokerID` for [`Trader`](crate::interface::trader::Trader).
    type OuterID: Id;
    /// [`LatencyGenerator`] type.
    type LatencyGenerator: LatencyGenerator<OuterID=Self::OuterID>;

    /// Returns [`LatencyGenerator`] describing a probabilistic model of latency
    /// against agents to which the `Self` agent connects.
    fn get_latency_generator(&self) -> Self::LatencyGenerator;
}

/// Describes a probabilistic model of latency.
pub trait LatencyGenerator
{
    /// ID type of the anchor agent against which the latent delay is sampled.
    type OuterID: Id;

    /// Samples latent delay that is outgoing for the generating agent
    /// and is incoming for the anchor one.
    ///
    /// For example, if some [`Broker`](crate::interface::broker::Broker)
    /// send a message to some [`Exchange`](crate::interface::exchange::Exchange),
    /// `outgoing_latency` of the `Broker::LatencyGenerator` should be called.
    ///
    /// # Arguments
    ///
    /// * `outer_id` — ID of the anchor agent against which the latent delay is sampled.
    /// * `event_dt` — [`DateTime`] at which the message was sent.
    /// * `rng` — thread-unique [`Kernel`](crate::kernel::Kernel) random number generator.
    fn outgoing_latency(
        &mut self,
        outer_id: Self::OuterID,
        event_dt: DateTime,
        rng: &mut impl Rng) -> u64;

    /// Samples latent delay that is incoming for the generating agent
    /// and is outgoing for the anchor one.
    ///
    /// For example, if some [`Exchange`](crate::interface::exchange::Exchange)
    /// send a message to some [`Broker`](crate::interface::broker::Broker),
    /// `incoming_latency` of the `Broker::LatencyGenerator` should be called.
    ///
    /// # Arguments
    ///
    /// * `outer_id` — ID of the anchor agent against which the latent delay is sampled.
    /// * `event_dt` — [`DateTime`] at which the message was sent.
    /// * `rng` — thread-unique [`Kernel`](crate::kernel::Kernel) random number generator.
    fn incoming_latency(
        &mut self,
        outer_id: Self::OuterID,
        event_dt: DateTime,
        rng: &mut impl Rng) -> u64;
}