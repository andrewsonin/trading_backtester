use {
    crate::{
        interface::{
            latency::Latent,
            message::{
                BrokerToExchange,
                BrokerToItself,
                BrokerToReplay,
                BrokerToTrader,
                ExchangeToBroker,
                ReplayToBroker,
                TraderToBroker,
            },
        },
        kernel::LatentActionProcessor,
        types::{Agent, Id, Named, TimeSync},
        utils::queue::MessageReceiver,
    },
    rand::Rng,
};

/// Defines [`Broker`] reaction to anything.
/// Supposed to be processed by [`LatentActionProcessor`]
/// before pushing into the [`Kernel`](crate::kernel::Kernel) queue.
#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct BrokerAction<B2R, B2E, B2T, B2B>
    where B2R: BrokerToReplay,
          B2E: BrokerToExchange,
          B2T: BrokerToTrader,
          B2B: BrokerToItself
{
    /// Constant part of the delay, in nanoseconds, between the current datetime of the [`Broker`]
    /// and the datetime of popping this action
    /// out of the [`Kernel`](crate::kernel::Kernel) queue.
    /// The final delay is the sum of this delay and the latency.
    pub delay: u64,
    /// [`Broker`] action content.
    pub content: BrokerActionKind<B2R, B2E, B2T, B2B>,
}

/// [`Broker`] action content.
#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum BrokerActionKind<B2R, B2E, B2T, B2B>
    where B2R: BrokerToReplay,
          B2E: BrokerToExchange,
          B2T: BrokerToTrader,
          B2B: BrokerToItself
{
    /// [`Broker`]-to-itself message.
    BrokerToItself(B2B),
    /// [`Broker`]-to-[`Replay`](crate::interface::replay::Replay) message.
    BrokerToReplay(B2R),
    /// [`Broker`]-to-[`Exchange`](crate::interface::exchange::Exchange) message.
    BrokerToExchange(B2E),
    /// [`Broker`]-to-[`Trader`](crate::interface::trader::Trader) message.
    BrokerToTrader(B2T),
}

/// Provides custom broker interface.
pub trait Broker
    where Self: TimeSync,
          Self: Latent<OuterID=Self::ExchangeID>,
          Self: Named<Self::BrokerID>,
          Self: Agent<Action=BrokerAction<Self::B2R, Self::B2E, Self::B2T, Self::B2B>>
{
    /// [`Broker`] identifier type.
    type BrokerID: Id;
    /// [`Trader`](crate::interface::trader::Trader) identifier type.
    type TraderID: Id;
    /// [`Exchange`](crate::interface::exchange::Exchange) identifier type.
    type ExchangeID: Id;

    /// [`Replay`](crate::interface::replay::Replay)-to-[`Broker`] query format.
    type R2B: ReplayToBroker<BrokerID=Self::BrokerID>;
    /// [`Exchange`](crate::interface::exchange::Exchange)-to-[`Broker`] query format.
    type E2B: ExchangeToBroker<BrokerID=Self::BrokerID>;
    /// [`Trader`](crate::interface::trader::Trader)-to-[`Broker`] query format.
    type T2B: TraderToBroker<BrokerID=Self::BrokerID>;
    /// [`Broker`]-to-[`Replay`](crate::interface::replay::Replay) query format.
    type B2R: BrokerToReplay;
    /// [`Broker`]-to-[`Exchange`](crate::interface::exchange::Exchange) query format.
    type B2E: BrokerToExchange<ExchangeID=Self::ExchangeID>;
    /// [`Broker`]-to-[`Trader`](crate::interface::trader::Trader) query format.
    type B2T: BrokerToTrader<TraderID=Self::TraderID>;
    /// [`Broker`]-to-itself query format.
    type B2B: BrokerToItself;
    /// [`Trader`](crate::interface::trader::Trader) subscription config format.
    type SubCfg;

    /// Defines the [`Broker`] reaction to a previously scheduled message from itself.
    /// Called whenever the [`Kernel`](crate::kernel::Kernel)
    /// pops a [`Self::B2B`] message out of its event queue.
    ///
    /// # Arguments
    ///
    /// * `message_receiver` — Proxy providing pushing access
    ///                        to the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `action_processor` — Structure needed to preprocess the [`Broker`]'s `Self::Action`
    ///                        into a format suitable for pushing
    ///                        into the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `scheduled_action` — scheduled message to be reacted to.
    /// * `rng` — thread-unique [`Kernel`](crate::kernel::Kernel) random number generator.
    fn wakeup<KerMsg: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        action_processor: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
        scheduled_action: Self::B2B,
        rng: &mut impl Rng,
    );

    /// Defines the [`Broker`] reaction to an incoming request
    /// from [`Trader`](crate::interface::trader::Trader).
    /// Called whenever the [`Kernel`](crate::kernel::Kernel)
    /// pops a [`Self::T2B`] message out of its event queue.
    ///
    /// # Arguments
    ///
    /// * `message_receiver` — Proxy providing pushing access
    ///                        to the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `action_processor` — Structure needed to preprocess the [`Broker`]'s `Self::Action`
    ///                        into a format suitable for pushing
    ///                        into the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `request` — received message to be reacted to.
    /// * `trader_id` — unique id of the [`Trader`](crate::interface::trader::Trader)
    /// who sent the message received.
    /// * `rng` — thread-unique [`Kernel`](crate::kernel::Kernel) random number generator.
    fn process_trader_request<KerMsg: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        action_processor: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
        request: Self::T2B,
        trader_id: Self::TraderID,
        rng: &mut impl Rng,
    );

    /// Defines the [`Broker`] reaction
    /// to an incoming request from [`Exchange`](crate::interface::exchange::Exchange).
    /// Called whenever the [`Kernel`](crate::kernel::Kernel)
    /// pops a [`Self::E2B`] message out of its event queue.
    ///
    /// # Arguments
    ///
    /// * `message_receiver` — Proxy providing pushing access
    /// to the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `action_processor` — Structure needed to preprocess the [`Broker`]'s `Self::Action`
    /// into a format suitable for pushing into the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `reply` — received message to be reacted to.
    /// * `exchange_id` — unique id of the [`Exchange`](crate::interface::exchange::Exchange)
    /// that sent the message received.
    /// * `rng` — thread-unique [`Kernel`](crate::kernel::Kernel) random number generator.
    fn process_exchange_reply<KerMsg: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        action_processor: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
        reply: Self::E2B,
        exchange_id: Self::ExchangeID,
        rng: &mut impl Rng,
    );

    /// Defines the [`Broker`] reaction to an incoming request
    /// from the [`Replay`](crate::interface::replay::Replay).
    /// Called whenever the [`Kernel`](crate::kernel::Kernel)
    /// pops a [`Self::R2B`] message out of its event queue.
    ///
    /// # Arguments
    ///
    /// * `message_receiver` — Proxy providing pushing access
    /// to the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `action_processor` — Structure needed to preprocess the [`Broker`]'s `Self::Action`
    /// into a format suitable for pushing into the [`Kernel`](crate::kernel::Kernel) event queue.
    /// * `request` — received message to be reacted to.
    /// * `rng` — thread-unique [`Kernel`](crate::kernel::Kernel) random number generator.
    fn process_replay_request<KerMsg: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KerMsg>,
        action_processor: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
        request: Self::R2B,
        rng: &mut impl Rng,
    );

    /// Called whenever the [`Broker`]
    /// is being connected to an [`Exchange`](crate::interface::exchange::Exchange).
    ///
    /// # Arguments
    ///
    /// * `exchange_id` — unique id
    /// of the [`Exchange`](crate::interface::exchange::Exchange) to be connected to.
    fn upon_connection_to_exchange(&mut self, exchange_id: Self::ExchangeID);

    /// Called whenever a [`Trader`](crate::interface::trader::Trader)
    /// is being connected to the [`Broker`].
    ///
    /// # Arguments
    ///
    /// * `trader_id` — unique id of the [`Trader`](crate::interface::trader::Trader) to connect.
    /// * `sub_cfgs` — [`Trader`](crate::interface::trader::Trader) subscription configs.
    fn register_trader(
        &mut self,
        trader_id: Self::TraderID,
        sub_cfgs: impl IntoIterator<Item=Self::SubCfg>);
}