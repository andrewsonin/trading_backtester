use {
    crate::{
        interface::{latency::Latent, message::{BrokerToTrader, TraderToBroker, TraderToItself}},
        kernel::LatentActionProcessor,
        types::{Agent, Id, Named, TimeSync},
        utils::queue::MessageReceiver,
    },
    rand::Rng,
};

/// Defines [`Trader`] reaction to anything.
/// Supposed to be processed by [`LatentActionProcessor`]
/// before pushing into the [`Kernel`](crate::kernel::Kernel) queue.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TraderAction<T2B: TraderToBroker, T2T: TraderToItself> {
    /// Constant part of the delay, in nanoseconds, between the current datetime of the [`Trader`]
    /// and the datetime of popping this action
    /// out of the [`Kernel`](crate::kernel::Kernel) queue.
    /// The final delay is the sum of this delay and the latency.
    pub delay: u64,
    /// [`Trader`] action content.
    pub content: TraderActionKind<T2B, T2T>,
}

/// [`Trader`] action content.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TraderActionKind<T2B: TraderToBroker, T2T: TraderToItself> {
    /// [`Trader`]-to-itself message.
    TraderToItself(T2T),
    /// [`Trader`]-to-[`Broker`](crate::interface::broker::Broker) message.
    TraderToBroker(T2B),
}

/// Provides custom trader interface.
pub trait Trader
    where Self: TimeSync,
          Self: Latent<OuterID=Self::BrokerID>,
          Self: Named<Self::TraderID>,
          Self: Agent<Action=TraderAction<Self::T2B, Self::T2T>>
{
    /// [`Trader`] identifier type.
    type TraderID: Id;
    /// [`Broker`](crate::interface::broker::Broker) identifier type.
    type BrokerID: Id;

    /// [`Broker`](crate::interface::broker::Broker)-to-[`Trader`] query format.
    type B2T: BrokerToTrader<TraderID=Self::TraderID>;
    /// [`Trader`]-to-itself query format.
    type T2T: TraderToItself;
    /// [`Trader`]-to-[`Broker`](crate::interface::broker::Broker) query format.
    type T2B: TraderToBroker<BrokerID=Self::BrokerID>;

    /// Defines the [`Trader`] reaction to a previously scheduled message from itself.
    /// Called whenever the [`Kernel`](crate::kernel::Kernel)
    /// pops a [`Self::T2T`] message out of its event queue.
    ///
    /// # Arguments
    ///
    /// * `message_receiver` — Proxy providing pushing access
    ///                        to the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `action_processor` — Structure needed to preprocess the [`Trader`]'s `Self::Action`
    ///                        into a format suitable for pushing
    ///                        into the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `scheduled_action` — Scheduled message to react to.
    /// * `rng` — Thread-unique [`Kernel`](crate::kernel::Kernel) random number generator.
    fn wakeup<KerMsg: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        action_processor: impl LatentActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
        scheduled_action: Self::T2T,
        rng: &mut impl Rng,
    );

    /// Defines the [`Trader`] reaction to an incoming request
    /// from [`Broker`](crate::interface::broker::Broker).
    /// Called whenever the [`Kernel`](crate::kernel::Kernel)
    /// pops a [`Self::B2T`] message out of its event queue.
    ///
    /// # Arguments
    ///
    /// * `message_receiver` — Proxy providing pushing access
    ///                        to the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `action_processor` — Structure needed to preprocess the [`Trader`]'s `Self::Action`
    ///                        into a format suitable for pushing
    ///                        into the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `reply` — Received message to react to.
    /// * `broker_id` — Unique id of the [`Broker`](crate::interface::broker::Broker)
    ///                 who sent the message received.
    /// * `rng` — Thread-unique [`Kernel`](crate::kernel::Kernel)
    ///           random number generator.
    fn process_broker_reply<KerMsg: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        action_processor: impl LatentActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
        reply: Self::B2T,
        broker_id: Self::BrokerID,
        rng: &mut impl Rng,
    );

    /// Called whenever the [`Trader`] registers at [`Broker`](crate::interface::broker::Broker).
    ///
    /// # Arguments
    ///
    /// * `broker_id` — Unique id of the [`Broker`](crate::interface::broker::Broker)
    ///                 to register at.
    fn upon_register_at_broker(&mut self, broker_id: Self::BrokerID);
}