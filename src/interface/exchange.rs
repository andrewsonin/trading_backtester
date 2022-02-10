use {
    crate::{
        interface::message::{
            BrokerToExchange,
            ExchangeToBroker,
            ExchangeToItself,
            ExchangeToReplay,
            ReplayToExchange,
        },
        types::{Agent, Id, Named, TimeSync},
        utils::queue::MessageReceiver,
    },
    rand::Rng,
};

/// Defines [`Exchange`] reaction to anything.
/// Supposed to be processed by `process_action` closures from the [`Exchange`] method signatures
/// before pushing into the [`Kernel`](crate::kernel::Kernel) queue.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExchangeAction<
    E2R: ExchangeToReplay,
    E2B: ExchangeToBroker,
    E2E: ExchangeToItself
> {
    /// Delay, in nanoseconds, between the current datetime of the [`Exchange`]
    /// and the datetime of popping this action
    /// out of the [`Kernel`](crate::kernel::Kernel) queue.
    pub delay: u64,
    /// [`Exchange`] action content.
    pub content: ExchangeActionKind<E2R, E2B, E2E>,
}

/// [`Exchange`] action content.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExchangeActionKind<
    E2R: ExchangeToReplay,
    E2B: ExchangeToBroker,
    E2E: ExchangeToItself
> {
    /// [`Exchange`]-to-itself message.
    ExchangeToItself(E2E),
    /// [`Exchange`]-to-[`Replay`](crate::interface::replay::Replay) message.
    ExchangeToReplay(E2R),
    /// [`Exchange`]-to-[`Broker`](crate::interface::broker::Broker) message.
    ExchangeToBroker(E2B),
}

/// Provides custom exchange interface.
pub trait Exchange
    where Self: TimeSync,
          Self: Named<Self::ExchangeID>,
          Self: Agent<Action=ExchangeAction<Self::E2R, Self::E2B, Self::E2E>>
{
    /// [`Exchange`] identifier type.
    type ExchangeID: Id;
    /// [`Broker`](crate::interface::broker::Broker) identifier type.
    type BrokerID: Id;

    /// [`Replay`](crate::interface::replay::Replay)-to-[`Exchange`] query format.
    type R2E: ReplayToExchange<ExchangeID=Self::ExchangeID>;
    /// [`Broker`](crate::interface::broker::Broker)-to-[`Exchange`] query format.
    type B2E: BrokerToExchange<ExchangeID=Self::ExchangeID>;
    /// [`Exchange`]-to-[`Replay`](crate::interface::replay::Replay) query format.
    type E2R: ExchangeToReplay;
    /// [`Exchange`]-to-[`Broker`](crate::interface::broker::Broker) query format.
    type E2B: ExchangeToBroker<BrokerID=Self::BrokerID>;
    /// [`Exchange`]-to-itself query format.
    type E2E: ExchangeToItself;

    /// Defines the [`Exchange`] reaction to a previously scheduled message from itself.
    /// Called whenever the [`Kernel`](crate::kernel::Kernel)
    /// pops a [`Self::E2E`] message out of its event queue.
    ///
    /// # Arguments
    ///
    /// * `message_receiver` — Proxy providing pushing access
    ///                        to the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `process_action` — closure needed to preprocess the [`Exchange`]'s `Self::Action`
    ///                      into a format suitable for pushing
    ///                      into the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `scheduled_action` — scheduled message to react to.
    /// * `rng` — thread-unique [`Kernel`](crate::kernel::Kernel) random number generator.
    fn wakeup<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
        scheduled_action: Self::E2E,
        rng: &mut RNG,
    );

    /// Defines the [`Exchange`] reaction to an incoming request
    /// from the [`Broker`](crate::interface::broker::Broker).
    /// Called whenever the [`Kernel`](crate::kernel::Kernel)
    /// pops a [`Self::B2E`] message out of its event queue.
    ///
    /// # Arguments
    ///
    /// * `message_receiver` — Proxy providing pushing access
    ///                        to the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `process_action` — closure needed to preprocess the [`Exchange`]'s `Self::Action`
    ///                      into a format suitable for pushing
    ///                      into the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `request` — received message to react to.
    /// * `rng` — thread-unique [`Kernel`](crate::kernel::Kernel) random number generator.
    fn process_broker_request<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
        request: Self::B2E,
        broker_id: Self::BrokerID,
        rng: &mut RNG,
    );

    /// Defines the [`Exchange`] reaction to an incoming request
    /// from the [`Replay`](crate::interface::replay::Replay).
    /// Called whenever the [`Kernel`](crate::kernel::Kernel)
    /// pops a [`Self::R2E`] message out of its event queue.
    ///
    /// # Arguments
    ///
    /// * `message_receiver` — Proxy providing pushing access
    ///                        to the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `process_action` — closure needed to preprocess the [`Exchange`]'s `Self::Action`
    ///                      into a format suitable for pushing
    ///                      into the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `request` — received message to react to.
    /// * `rng` — thread-unique [`Kernel`](crate::kernel::Kernel) random number generator.
    fn process_replay_request<KerMsg: Ord, RNG: Rng>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
        request: Self::R2E,
        rng: &mut RNG,
    );

    /// Called whenever a [`Broker`](crate::interface::broker::Broker)
    /// is being connected to the [`Exchange`].
    ///
    /// # Arguments
    ///
    /// * `broker_id` — unique id of the [`Broker`](crate::interface::broker::Broker) to connect.
    fn connect_broker(&mut self, broker_id: Self::BrokerID);
}