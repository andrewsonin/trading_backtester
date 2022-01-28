use {
    crate::{
        broker::{
            Broker,
            BrokerToExchange,
            BrokerToItself,
            BrokerToTrader,
        },
        exchange::{
            Exchange,
            ExchangeActionKind,
            ExchangeToBroker,
            ExchangeToItself,
            ExchangeToReplay,
        },
        kernel::action_processors::{BrokerActionProcessor, TraderActionProcessor},
        latency::LatencyGenerator,
        replay::{Replay, ReplayActionKind, ReplayToExchange, ReplayToItself},
        trader::{
            Trader,
            TraderToBroker,
            TraderToItself,
        },
        types::{DateTime, Duration, Id},
        utils::queue::{LessElementBinaryHeap, MessageReceiver},
    },
    rand::{Rng, rngs::StdRng, SeedableRng},
    std::{cmp::Reverse, collections::HashMap, marker::PhantomData},
};

mod action_processors;

pub trait LatentActionProcessor<Action, OuterID: Id>
{
    type KerMsg: Ord;

    fn process_action(
        &mut self,
        action: Action,
        latency_generator: impl LatencyGenerator<OuterID>,
        rng: &mut impl Rng) -> Self::KerMsg;
}

pub struct Kernel<T, B, E, R, RNG>
    where
        T: Trader<TraderID=B::TraderID, BrokerID=B::BrokerID, T2B=B::T2B, B2T=B::B2T>,
        B: Broker<BrokerID=E::BrokerID, ExchangeID=E::ExchangeID, B2E=E::B2E, E2B=E::E2B>,
        E: Exchange<ExchangeID=R::ExchangeID, E2R=R::E2R, R2E=R::R2E>,
        R: Replay,
        RNG: SeedableRng + Rng
{
    traders: HashMap<T::TraderID, T>,
    brokers: HashMap<B::BrokerID, B>,
    exchanges: HashMap<E::ExchangeID, E>,
    replay: R,

    message_queue: LessElementBinaryHeap<Message<<Self as InnerMessage>::MessageContent>>,

    end_dt: DateTime,
    current_dt: DateTime,

    rng: RNG,
}

trait InnerMessage {
    type MessageContent: Ord;
}

impl<T, B, E, R, RNG> InnerMessage for Kernel<T, B, E, R, RNG>
    where
        T: Trader<TraderID=B::TraderID, BrokerID=B::BrokerID, T2B=B::T2B, B2T=B::B2T>,
        B: Broker<BrokerID=E::BrokerID, ExchangeID=E::ExchangeID, B2E=E::B2E, E2B=E::E2B>,
        E: Exchange<ExchangeID=R::ExchangeID, E2R=R::E2R, R2E=R::R2E>,
        R: Replay,
        RNG: SeedableRng + Rng
{
    type MessageContent = MessageContent<
        E::ExchangeID, B::BrokerID, T::TraderID,
        R::R2R, R::R2E,
        B::B2E, B::B2T, B::B2B,
        T::T2B, T::T2T,
        E::E2R, E::E2B, E::E2E
    >;
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct Message<MessageContent: Ord> {
    datetime: DateTime,
    body: MessageContent,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
enum MessageContent<
    ExchangeID: Id,
    BrokerID: Id,
    TraderID: Id,
    R2R: ReplayToItself,
    R2E: ReplayToExchange<ExchangeID=ExchangeID>,
    B2E: BrokerToExchange<ExchangeID=ExchangeID>,
    B2T: BrokerToTrader<TraderID=TraderID>,
    B2B: BrokerToItself,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    T2T: TraderToItself,
    E2R: ExchangeToReplay,
    E2B: ExchangeToBroker<BrokerID=BrokerID>,
    E2E: ExchangeToItself
> {
    ReplayWakeUp(R2R),

    ReplayToExchange(R2E),

    ExchangeWakeUp(ExchangeID, E2E),

    ExchangeToReplay(ExchangeID, E2R),

    ExchangeToBroker(ExchangeID, E2B),

    BrokerWakeUp(BrokerID, B2B),

    BrokerToExchange(BrokerID, B2E),

    BrokerToTrader(BrokerID, B2T),

    TraderWakeUp(TraderID, T2T),

    TraderToBroker(TraderID, T2B),
}

pub struct KernelBuilder<T, B, E, R, RNG>
    where
        T: Trader<TraderID=B::TraderID, BrokerID=B::BrokerID, T2B=B::T2B, B2T=B::B2T>,
        B: Broker<BrokerID=E::BrokerID, ExchangeID=E::ExchangeID, B2E=E::B2E, E2B=E::E2B>,
        E: Exchange<ExchangeID=R::ExchangeID, E2R=R::E2R, R2E=R::R2E>,
        R: Replay,
        RNG: SeedableRng + Rng
{
    traders: HashMap<T::TraderID, T>,
    brokers: HashMap<B::BrokerID, B>,
    exchanges: HashMap<E::ExchangeID, E>,
    replay: R,

    start_dt: DateTime,
    end_dt: DateTime,

    seed: Option<u64>,

    phantoms: PhantomData<RNG>,
}

impl<T, B, E, R>
KernelBuilder<T, B, E, R, StdRng>
    where
        T: Trader<TraderID=B::TraderID, BrokerID=B::BrokerID, T2B=B::T2B, B2T=B::B2T>,
        B: Broker<BrokerID=E::BrokerID, ExchangeID=E::ExchangeID, B2E=E::B2E, E2B=E::E2B>,
        E: Exchange<ExchangeID=R::ExchangeID, E2R=R::E2R, R2E=R::R2E>,
        R: Replay,
{
    pub fn new<CE, CB, SC>(exchanges: impl IntoIterator<Item=E>,
                           brokers: impl IntoIterator<Item=(B, CE)>,
                           traders: impl IntoIterator<Item=(T, CB)>,
                           replay: R,
                           date_range: (DateTime, DateTime)) -> Self
        where
            CE: IntoIterator<Item=E::ExchangeID>,      // Connected Exchanges
            CB: IntoIterator<Item=(B::BrokerID, SC)>,  // Connected Brokers
            SC: IntoIterator<Item=B::SubCfg>
    {
        let (start_dt, end_dt) = date_range;
        if end_dt < start_dt {
            panic!("start_dt ({start_dt}) is less than end_dt ({end_dt})")
        }
        let exchanges: Vec<_> = exchanges.into_iter().collect();
        let n_exchanges = exchanges.len();
        let mut exchanges: HashMap<E::ExchangeID, E> = exchanges.into_iter()
            .map(
                |mut exchange| {
                    *exchange.current_datetime_mut() = start_dt;
                    (exchange.get_name(), exchange)
                }
            )
            .collect();
        if exchanges.len() != n_exchanges {
            panic!("exchanges contain entries with duplicate names")
        }

        let brokers: Vec<_> = brokers.into_iter().collect();
        let n_brokers = brokers.len();
        let mut brokers: HashMap<B::BrokerID, B> = brokers.into_iter()
            .map(
                |(mut broker, exchanges_to_connect)| {
                    *broker.current_datetime_mut() = start_dt;
                    let broker_id = broker.get_name();
                    for exchange_id in exchanges_to_connect {
                        if let Some(exchange) = exchanges.get_mut(&exchange_id) {
                            exchange.connect_broker(broker_id);
                            broker.upon_connection_to_exchange(exchange_id)
                        } else {
                            panic!(
                                "Cannot connect Broker {broker_id} to the Exchange: {exchange_id}"
                            )
                        }
                    }
                    (broker_id, broker)
                }
            )
            .collect();
        if brokers.len() != n_brokers {
            panic!("brokers contain entries with duplicate names")
        }

        let traders: Vec<_> = traders.into_iter().collect();
        let n_traders = traders.len();
        let traders: HashMap<T::TraderID, T> = traders.into_iter()
            .map(
                |(mut trader, brokers_to_register)| {
                    *trader.current_datetime_mut() = start_dt;
                    let trader_id = trader.get_name();
                    for (broker_id, subscription_config) in brokers_to_register {
                        if let Some(broker) = brokers.get_mut(&broker_id) {
                            broker.register_trader(trader_id, subscription_config);
                            trader.upon_register_at_broker(broker_id)
                        } else {
                            panic!("Cannot register Trader {trader_id} at the Broker: {broker_id}")
                        }
                    }
                    (trader_id, trader)
                }
            )
            .collect();
        if traders.len() != n_traders {
            panic!("traders contain entries with duplicate names")
        }

        KernelBuilder {
            traders,
            brokers,
            exchanges,
            replay,
            end_dt,
            start_dt,
            seed: None,
            phantoms: Default::default(),
        }
    }

    pub fn with_rng<RNG: Rng + SeedableRng>(self) -> KernelBuilder<T, B, E, R, RNG>
    {
        let KernelBuilder {
            traders, brokers, exchanges, replay, end_dt, start_dt, seed, ..
        } = self;
        KernelBuilder {
            traders,
            brokers,
            exchanges,
            replay,
            end_dt,
            start_dt,
            seed,
            phantoms: Default::default(),
        }
    }
}

impl<T, B, E, R, RNG>
KernelBuilder<T, B, E, R, RNG>
    where
        T: Trader<TraderID=B::TraderID, BrokerID=B::BrokerID, T2B=B::T2B, B2T=B::B2T>,
        B: Broker<BrokerID=E::BrokerID, ExchangeID=E::ExchangeID, B2E=E::B2E, E2B=E::E2B>,
        E: Exchange<ExchangeID=R::ExchangeID, E2R=R::E2R, R2E=R::R2E>,
        R: Replay,
        RNG: Rng + SeedableRng
{
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn build(self) -> Kernel<T, B, E, R, RNG>
    {
        let KernelBuilder {
            traders, brokers, exchanges, mut replay, end_dt, start_dt, seed, ..
        } = self;

        *replay.current_datetime_mut() = start_dt;
        let first_message = Kernel::<T, B, E, R, RNG>::process_replay_action(
            start_dt,
            replay.next().expect("Replay does not contain any entries"),
        );
        if first_message.datetime < start_dt {
            panic!("First message datetime is less than the simulation start datetime")
        }
        Kernel {
            traders,
            brokers,
            exchanges,
            replay,
            message_queue: LessElementBinaryHeap([Reverse(first_message)].into()),
            end_dt,
            current_dt: start_dt,
            rng: if let Some(seed) = seed {
                RNG::seed_from_u64(seed)
            } else {
                RNG::from_entropy()
            },
        }
    }
}

impl<T, B, E, R, RNG> Kernel<T, B, E, R, RNG>
    where
        T: Trader<TraderID=B::TraderID, BrokerID=B::BrokerID, T2B=B::T2B, B2T=B::B2T>,
        B: Broker<BrokerID=E::BrokerID, ExchangeID=E::ExchangeID, B2E=E::B2E, E2B=E::E2B>,
        E: Exchange<ExchangeID=R::ExchangeID, E2R=R::E2R, R2E=R::R2E>,
        R: Replay,
        RNG: SeedableRng + Rng
{
    pub fn run_simulation(&mut self)
    {
        while let Some(message) = self.message_queue.pop()
        {
            self.current_dt = message.datetime;
            if self.current_dt > self.end_dt {
                break;
            }
            self.handle_message(message.body)
        }
    }

    fn handle_message(&mut self, message: <Self as InnerMessage>::MessageContent)
    {
        match message
        {
            MessageContent::ReplayWakeUp(scheduled_action) => {
                self.handle_replay_wakeup(scheduled_action)
            }
            MessageContent::ReplayToExchange(replay_request) => {
                if let Some(action) = self.replay.next() {
                    self.message_queue.push(Self::process_replay_action(self.current_dt, action))
                }
                self.handle_replay_to_exchange(replay_request)
            }
            MessageContent::ExchangeWakeUp(exchange_id, scheduled_action) => {
                self.handle_exchange_wakeup(exchange_id, scheduled_action)
            }
            MessageContent::ExchangeToReplay(exchange_id, reply) => {
                self.handle_exchange_to_replay(exchange_id, reply)
            }
            MessageContent::ExchangeToBroker(exchange_id, reply) => {
                self.handle_exchange_to_broker(exchange_id, reply)
            }
            MessageContent::BrokerWakeUp(broker_id, scheduled_action) => {
                self.handle_broker_wakeup(broker_id, scheduled_action)
            }
            MessageContent::BrokerToExchange(broker_id, request) => {
                self.handle_broker_to_exchange(broker_id, request)
            }
            MessageContent::BrokerToTrader(broker_id, reply) => {
                self.handle_broker_to_trader(broker_id, reply)
            }
            MessageContent::TraderWakeUp(trader_id, scheduled_action) => {
                self.handle_trader_wakeup(trader_id, scheduled_action)
            }
            MessageContent::TraderToBroker(trader_id, request) => {
                self.handle_trader_to_broker(trader_id, request)
            }
        }
    }

    fn handle_replay_wakeup(&mut self, scheduled_action: R::R2R)
    {
        *self.replay.current_datetime_mut() = self.current_dt;
        let process_replay_action = |action| Self::process_replay_action(self.current_dt, action);
        self.replay.wakeup(
            MessageReceiver::new(&mut self.message_queue),
            process_replay_action,
            scheduled_action,
            &mut self.rng,
        )
    }

    fn handle_replay_to_exchange(&mut self, request: R::R2E)
    {
        let exchange_id = request.get_exchange_id();
        let exchange = self.exchanges.get_mut(&exchange_id).unwrap_or_else(
            || panic!("Kernel does not know such an Exchange: {exchange_id}")
        );
        *exchange.current_datetime_mut() = self.current_dt;
        let process_exchange_action = |action, rng: &mut RNG|
            Self::process_exchange_action(
                self.current_dt,
                &mut self.brokers,
                rng,
                action,
                exchange_id,
            );
        exchange.process_replay_request(
            MessageReceiver::new(&mut self.message_queue),
            process_exchange_action,
            request,
            &mut self.rng,
        )
    }

    fn handle_exchange_wakeup(&mut self, exchange_id: E::ExchangeID, scheduled_action: E::E2E)
    {
        let exchange = self.exchanges.get_mut(&exchange_id).unwrap_or_else(
            || panic!("Kernel does not know such an Exchange: {exchange_id}")
        );
        *exchange.current_datetime_mut() = self.current_dt;
        let process_exchange_action = |action, rng: &mut RNG|
            Self::process_exchange_action(
                self.current_dt,
                &mut self.brokers,
                rng,
                action,
                exchange_id,
            );
        exchange.wakeup(
            MessageReceiver::new(&mut self.message_queue),
            process_exchange_action,
            scheduled_action,
            &mut self.rng,
        )
    }

    fn handle_exchange_to_replay(&mut self, exchange_id: E::ExchangeID, reply: R::E2R)
    {
        *self.replay.current_datetime_mut() = self.current_dt;
        let process_replay_action = |action| Self::process_replay_action(self.current_dt, action);
        self.replay.handle_exchange_reply(
            MessageReceiver::new(&mut self.message_queue),
            process_replay_action,
            reply,
            exchange_id,
            &mut self.rng,
        )
    }

    fn handle_exchange_to_broker(&mut self, exchange_id: E::ExchangeID, reply: B::E2B)
    {
        let broker_id = reply.get_broker_id();
        let broker = self.brokers.get_mut(&broker_id).unwrap_or_else(
            || panic!("Kernel does not know such a Broker: {broker_id}")
        );
        *broker.current_datetime_mut() = self.current_dt;
        let broker_action_processor = BrokerActionProcessor::<B::BrokerID, B::Action, T, E, R>::new(
            self.current_dt,
            broker_id,
            &mut self.traders,
        );
        broker.process_exchange_reply(
            MessageReceiver::new(&mut self.message_queue),
            broker_action_processor,
            reply,
            exchange_id,
            &mut self.rng,
        )
    }

    fn handle_broker_wakeup(&mut self, broker_id: B::BrokerID, scheduled_action: B::B2B)
    {
        let broker = self.brokers.get_mut(&broker_id).unwrap_or_else(
            || panic!("Kernel does not know such a Broker: {broker_id}")
        );
        *broker.current_datetime_mut() = self.current_dt;
        let broker_action_processor = BrokerActionProcessor::<B::BrokerID, B::Action, T, E, R>::new(
            self.current_dt,
            broker_id,
            &mut self.traders,
        );
        broker.wakeup(
            MessageReceiver::new(&mut self.message_queue),
            broker_action_processor,
            scheduled_action,
            &mut self.rng,
        )
    }

    fn handle_broker_to_exchange(&mut self, broker_id: B::BrokerID, request: E::B2E)
    {
        let exchange_id = request.get_exchange_id();
        let exchange = self.exchanges.get_mut(&exchange_id).unwrap_or_else(
            || panic!("Kernel does not know such an Exchange: {exchange_id}")
        );
        *exchange.current_datetime_mut() = self.current_dt;
        let process_exchange_action = |action, rng: &mut RNG|
            Self::process_exchange_action(
                self.current_dt,
                &mut self.brokers,
                rng,
                action,
                exchange_id,
            );
        exchange.process_broker_request(
            MessageReceiver::new(&mut self.message_queue),
            process_exchange_action,
            request,
            broker_id,
            &mut self.rng,
        )
    }

    fn handle_broker_to_trader(&mut self, broker_id: B::BrokerID, reply: B::B2T)
    {
        let trader_id = reply.get_trader_id();
        let trader = self.traders.get_mut(&trader_id).unwrap_or_else(
            || panic!("Kernel does not know such a Trader: {trader_id}")
        );
        *trader.current_datetime_mut() = self.current_dt;
        let trader_action_processor = TraderActionProcessor::<T::TraderID, T::Action, B, E, R>::new(
            self.current_dt,
            trader_id,
        );
        trader.process_broker_reply(
            MessageReceiver::new(&mut self.message_queue),
            trader_action_processor,
            reply,
            broker_id,
            &mut self.rng,
        )
    }

    fn handle_trader_wakeup(&mut self, trader_id: T::TraderID, scheduled_action: T::T2T)
    {
        let trader = self.traders.get_mut(&trader_id).unwrap_or_else(
            || panic!("Kernel does not know such a Trader: {trader_id}")
        );
        *trader.current_datetime_mut() = self.current_dt;
        let trader_action_processor = TraderActionProcessor::<T::TraderID, T::Action, B, E, R>::new(
            self.current_dt,
            trader_id,
        );
        trader.wakeup(
            MessageReceiver::new(&mut self.message_queue),
            trader_action_processor,
            scheduled_action,
            &mut self.rng,
        )
    }

    fn handle_trader_to_broker(&mut self, trader_id: T::TraderID, request: B::T2B)
    {
        let broker_id = request.get_broker_id();
        let broker = self.brokers.get_mut(&broker_id).unwrap_or_else(
            || panic!("Kernel does not know such an Broker: {broker_id}")
        );
        *broker.current_datetime_mut() = self.current_dt;
        let broker_action_processor = BrokerActionProcessor::<B::BrokerID, B::Action, T, E, R>::new(
            self.current_dt,
            broker_id,
            &mut self.traders,
        );
        broker.process_trader_request(
            MessageReceiver::new(&mut self.message_queue),
            broker_action_processor,
            request,
            trader_id,
            &mut self.rng,
        )
    }

    fn process_replay_action(
        current_dt: DateTime,
        action: R::Item) -> Message<<Self as InnerMessage>::MessageContent>
    {
        if action.datetime < current_dt {
            panic!(
                "Replay yielded action {action:?} which DateTime ({}) \
                is less than the Kernel current DateTime ({current_dt})",
                action.datetime
            )
        };
        Message {
            datetime: action.datetime,
            body: match action.content {
                ReplayActionKind::ReplayToExchange(action) => {
                    MessageContent::ReplayToExchange(action)
                }
                ReplayActionKind::ReplayToItself(action) => {
                    MessageContent::ReplayWakeUp(action)
                }
            },
        }
    }

    fn process_exchange_action(
        current_dt: DateTime,
        brokers: &mut HashMap<B::BrokerID, B>,
        rng: &mut RNG,
        action: E::Action,
        exchange_id: E::ExchangeID) -> Message<<Self as InnerMessage>::MessageContent>
    {
        let delayed_dt = current_dt + Duration::nanoseconds(action.delay as i64);
        let (datetime, body) = match action.content
        {
            ExchangeActionKind::ExchangeToBroker(reply) => {
                let broker_id = reply.get_broker_id();
                let broker = brokers.get_mut(&broker_id).unwrap_or_else(
                    || panic!("Kernel does not know such a Broker: {broker_id}")
                );
                *broker.current_datetime_mut() = current_dt;
                let latency = broker
                    .get_latency_generator()
                    .incoming_latency(exchange_id, delayed_dt, rng);
                (
                    delayed_dt + Duration::nanoseconds(latency as i64),
                    MessageContent::ExchangeToBroker(exchange_id, reply)
                )
            }
            ExchangeActionKind::ExchangeToReplay(reply) => {
                (
                    delayed_dt,
                    MessageContent::ExchangeToReplay(exchange_id, reply)
                )
            }
            ExchangeActionKind::ExchangeToItself(wakeup) => {
                (
                    delayed_dt,
                    MessageContent::ExchangeWakeUp(exchange_id, wakeup)
                )
            }
        };
        Message { datetime, body }
    }
}