use {
    crate::{
        interface::message::{
            BrokerToReplay,
            ExchangeToReplay,
            ReplayToBroker,
            ReplayToExchange,
            ReplayToItself,
        },
        types::{DateTime, Id, TimeSync},
    },
    rand::Rng,
    std::fmt::Debug,
};

/// Defines [`Replay`] reaction to anything.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ReplayAction<R2R: ReplayToItself, R2E: ReplayToExchange, R2B: ReplayToBroker> {
    /// Global [`DateTime`] of the reaction.
    pub datetime: DateTime,
    /// [`Replay`] action content.
    pub content: ReplayActionKind<R2R, R2E, R2B>,
}

/// [`Replay`] action content.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ReplayActionKind<R2R: ReplayToItself, R2E: ReplayToExchange, R2B: ReplayToBroker> {
    /// [`Replay`]-to-itself message.
    ReplayToItself(R2R),
    /// [`Replay`]-to-[`Exchange`](crate::interface::exchange::Exchange) message.
    ReplayToExchange(R2E),
    /// [`Replay`]-to-[`Broker`](crate::interface::broker::Broker) message.
    ReplayToBroker(R2B),
}

/// Provides custom replay interface.
pub trait Replay
    where Self: TimeSync,
          Self: Iterator<Item=ReplayAction<Self::R2R, Self::R2E, Self::R2B>>
{
    /// [`Exchange`](crate::interface::exchange::Exchange) identifier type.
    type ExchangeID: Id;
    /// [`Broker`](crate::interface::broker::Broker) identifier type.
    type BrokerID: Id;

    /// [`Exchange`](crate::interface::exchange::Exchange)-to-[`Replay`] query format.
    type E2R: ExchangeToReplay;
    /// [`Broker`](crate::interface::broker::Broker)-to-[`Replay`] query format.
    type B2R: BrokerToReplay;
    /// [`Replay`]-to-itself query format.
    type R2R: ReplayToItself;
    /// [`Replay`]-to-[`Exchange`](crate::interface::exchange::Exchange) query format.
    type R2E: ReplayToExchange<ExchangeID=Self::ExchangeID>;
    /// [`Replay`]-to-[`Broker`](crate::interface::broker::Broker) query format.
    type R2B: ReplayToBroker<BrokerID=Self::BrokerID>;

    /// Defines the [`Replay`] reaction to a previously scheduled message from itself.
    /// Called whenever the [`Kernel`](crate::kernel::Kernel)
    /// pops a [`Self::R2R`] message out of its event queue.
    ///
    /// # Arguments
    ///
    /// * `scheduled_action` — scheduled message to react to.
    /// * `rng` — thread-unique [`Kernel`](crate::kernel::Kernel) random number generator.
    fn wakeup(&mut self, scheduled_action: Self::R2R, rng: &mut impl Rng);

    /// Defines the [`Replay`] reaction
    /// to an incoming reply from [`Exchange`](crate::interface::exchange::Exchange).
    /// Called whenever the [`Kernel`](crate::kernel::Kernel)
    /// pops a [`Self::E2R`] message out of its event queue.
    ///
    /// # Arguments
    ///
    /// * `reply` — received message to react to.
    /// * `exchange_id` — unique id of the [`Exchange`](crate::interface::exchange::Exchange)
    ///                   that sent the message received.
    /// * `rng` — thread-unique [`Kernel`](crate::kernel::Kernel) random number generator.
    fn handle_exchange_reply(
        &mut self,
        reply: Self::E2R,
        exchange_id: Self::ExchangeID,
        rng: &mut impl Rng,
    );

    /// Defines the [`Replay`] reaction
    /// to an incoming reply from [`Broker`](crate::interface::broker::Broker).
    /// Called whenever the [`Kernel`](crate::kernel::Kernel)
    /// pops a [`Self::B2R`] message out of its event queue.
    ///
    /// # Arguments
    ///
    /// * `reply` — received message to react to.
    /// * `broker_id` — unique id of the [`Broker`](crate::interface::broker::Broker)
    ///                 that sent the message received.
    /// * `rng` — thread-unique [`Kernel`](crate::kernel::Kernel) random number generator.
    fn handle_broker_reply(
        &mut self,
        reply: Self::B2R,
        broker_id: Self::BrokerID,
        rng: &mut impl Rng,
    );
}