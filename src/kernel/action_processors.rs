use {
    crate::{
        interface::{
            broker::{Broker, BrokerAction, BrokerActionKind},
            exchange::Exchange,
            latency::LatencyGenerator,
            message::{
                BrokerToExchange,
                BrokerToItself,
                BrokerToReplay,
                BrokerToTrader,
                TraderToBroker,
                TraderToItself,
            },
            replay::Replay,
            trader::{Trader, TraderAction, TraderActionKind},
        },
        kernel::{LatentActionProcessor, Message, MessageContent},
        types::{DateTime, Duration, Id},
    },
    rand::Rng,
    std::{collections::HashMap, marker::PhantomData},
};

pub(in crate::kernel) struct BrokerActionProcessor<
    'a,
    BrokerID: Id, BrokerAction,
    T: Trader, E: Exchange, R: Replay
> {
    current_dt: DateTime,
    traders: &'a mut HashMap<T::TraderID, T>,
    broker_id: BrokerID,
    phantom: PhantomData<(BrokerAction, E, R)>,
}

pub(in crate::kernel) struct TraderActionProcessor<
    TraderID: Id, TraderAction,
    B: Broker, E: Exchange, R: Replay
> {
    current_dt: DateTime,
    trader_id: TraderID,
    phantom: PhantomData<(TraderAction, B, E, R)>,
}

impl<
    'a,
    BrokerID: Id, BrokerAction,
    T: Trader, E: Exchange, R: Replay
>
BrokerActionProcessor<'a, BrokerID, BrokerAction, T, E, R>
{
    #[inline]
    pub fn new(
        current_dt: DateTime,
        broker_id: BrokerID,
        traders: &'a mut HashMap<T::TraderID, T>) -> Self
    {
        Self {
            current_dt,
            traders,
            broker_id,
            phantom: Default::default(),
        }
    }
}

impl<
    TraderID: Id, TraderAction,
    B: Broker, E: Exchange, R: Replay
>
TraderActionProcessor<TraderID, TraderAction, B, E, R>
{
    #[inline]
    pub fn new(current_dt: DateTime, trader_id: TraderID) -> Self {
        Self {
            current_dt,
            trader_id,
            phantom: Default::default(),
        }
    }
}

impl<
    'a,
    BrokerID: Id,
    B2R: BrokerToReplay,
    B2E: BrokerToExchange<ExchangeID=R::ExchangeID>,
    B2T: BrokerToTrader<TraderID=T::TraderID>,
    B2B: BrokerToItself,
    T: Trader<BrokerID=BrokerID, B2T=B2T>,
    E: Exchange<BrokerID=BrokerID, ExchangeID=R::ExchangeID, B2E=B2E, E2R=R::E2R, R2E=R::R2E>,
    R: Replay<BrokerID=BrokerID, B2R=B2R>,
>
LatentActionProcessor<BrokerAction<B2R, B2E, B2T, B2B>, E::ExchangeID>
for BrokerActionProcessor<'a, BrokerID, BrokerAction<B2R, B2E, B2T, B2B>, T, E, R>
{
    type KerMsg = Message<
        MessageContent<
            E::ExchangeID, BrokerID, T::TraderID,
            R::R2R, R::R2E, R::R2B,
            B2R, B2E, B2T, B2B,
            T::T2B, T::T2T,
            E::E2R, E::E2B, E::E2E
        >
    >;

    #[inline]
    fn process_action(
        &mut self,
        action: BrokerAction<B2R, B2E, B2T, B2B>,
        mut latency_generator: impl LatencyGenerator<OuterID=E::ExchangeID>,
        rng: &mut impl Rng) -> Self::KerMsg
    {
        let delayed_dt = self.current_dt + Duration::nanoseconds(action.delay as i64);
        let (datetime, body) = match action.content
        {
            BrokerActionKind::BrokerToReplay(reply) => {
                (
                    delayed_dt,
                    MessageContent::BrokerToReplay { broker_id: self.broker_id, b2r: reply }
                )
            }
            BrokerActionKind::BrokerToTrader(reply) => {
                let trader_id = reply.get_trader_id();
                let trader = self.traders.get_mut(&trader_id).unwrap_or_else(
                    || panic!("Kernel does not know such a Trader: {trader_id}")
                );
                *trader.current_datetime_mut() = self.current_dt;
                let latency = trader
                    .get_latency_generator()
                    .incoming_latency(self.broker_id, delayed_dt, rng);
                (
                    delayed_dt + Duration::nanoseconds(latency as i64),
                    MessageContent::BrokerToTrader { broker_id: self.broker_id, b2t: reply }
                )
            }
            BrokerActionKind::BrokerToExchange(request) => {
                let exchange_id = request.get_exchange_id();
                let latency = latency_generator.outgoing_latency(exchange_id, delayed_dt, rng);
                (
                    delayed_dt + Duration::nanoseconds(latency as i64),
                    MessageContent::BrokerToExchange { broker_id: self.broker_id, b2e: request }
                )
            }
            BrokerActionKind::BrokerToItself(wakeup) => {
                (
                    delayed_dt,
                    MessageContent::BrokerWakeUp { broker_id: self.broker_id, b2b: wakeup }
                )
            }
        };
        Message { datetime, body }
    }
}

impl<
    TraderID: Id,
    T2B: TraderToBroker<BrokerID=B::BrokerID>,
    T2T: TraderToItself,
    B: Broker<T2B=T2B, ExchangeID=R::ExchangeID, TraderID=TraderID, BrokerID=R::BrokerID>,
    E: Exchange<BrokerID=B::BrokerID, ExchangeID=R::ExchangeID, B2E=B::B2E, E2R=R::E2R, R2E=R::R2E>,
    R: Replay
>
LatentActionProcessor<TraderAction<T2B, T2T>, B::BrokerID>
for TraderActionProcessor<TraderID, TraderAction<T2B, T2T>, B, E, R>
{
    type KerMsg = Message<
        MessageContent<
            E::ExchangeID, B::BrokerID, TraderID,
            R::R2R, R::R2E, R::R2B,
            B::B2R, B::B2E, B::B2T, B::B2B,
            B::T2B, T2T,
            E::E2R, E::E2B, E::E2E
        >
    >;

    #[inline]
    fn process_action(
        &mut self,
        action: TraderAction<T2B, T2T>,
        mut latency_generator: impl LatencyGenerator<OuterID=B::BrokerID>,
        rng: &mut impl Rng) -> Self::KerMsg
    {
        let delayed_dt = self.current_dt + Duration::nanoseconds(action.delay as i64);
        let (datetime, body) = match action.content
        {
            TraderActionKind::TraderToBroker(request) => {
                let broker_id = request.get_broker_id();
                let latency = latency_generator.outgoing_latency(broker_id, delayed_dt, rng);
                (
                    delayed_dt + Duration::nanoseconds(latency as i64),
                    MessageContent::TraderToBroker { trader_id: self.trader_id, t2b: request }
                )
            }
            TraderActionKind::TraderToItself(wakeup) => {
                (
                    delayed_dt,
                    MessageContent::TraderWakeUp { trader_id: self.trader_id, t2t: wakeup }
                )
            }
        };
        Message { datetime, body }
    }
}