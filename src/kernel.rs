use {
    crate::{
        broker::{
            Broker,
            BrokerAction,
            BrokerActionKind,
            reply::BrokerToTrader,
            request::BrokerToExchange,
        },
        exchange::{
            Exchange,
            ExchangeAction,
            ExchangeActionKind,
            reply::{ExchangeToBroker, ExchangeToReplay},
        },
        replay::{Replay, ReplayAction, request::ReplayToExchange},
        settlement::GetSettlementLag,
        trader::{
            request::TraderToBroker,
            subscriptions::SubscriptionConfig,
            Trader,
            TraderAction,
            TraderActionKind,
        },
        types::{DateTime, Duration, Identifier},
        utils::{ExpectWith, queue::LessElementBinaryHeap, rand::{Rng, rngs::StdRng, SeedableRng}},
    },
    std::{cmp::Reverse, collections::HashMap, marker::PhantomData},
};

pub struct Kernel<
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag,
    T: Trader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>,
    B: Broker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>,
    E: Exchange<ExchangeID, BrokerID, Symbol, Settlement>,
    R: Replay<ExchangeID, Symbol, Settlement>,
    RNG: SeedableRng + Rng
> {
    traders: HashMap<TraderID, T>,
    brokers: HashMap<BrokerID, B>,
    exchanges: HashMap<ExchangeID, E>,
    replay: R,

    message_queue: LessElementBinaryHeap<
        Message<ExchangeID, BrokerID, TraderID, Symbol, Settlement>
    >,

    end_dt: DateTime,
    current_dt: DateTime,

    rng: RNG,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct Message<
    ExchangeID: Identifier,
    BrokerID: Identifier,
    TraderID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    datetime: DateTime,
    body: MessageContent<ExchangeID, BrokerID, TraderID, Symbol, Settlement>,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
enum MessageContent<
    ExchangeID: Identifier,
    BrokerID: Identifier,
    TraderID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    ReplayToExchange(ReplayToExchange<ExchangeID, Symbol, Settlement>),

    ExchangeToReplay(ExchangeToReplay<Symbol, Settlement>, ExchangeID),

    BrokerToExchange(BrokerToExchange<ExchangeID, Symbol, Settlement>, BrokerID),

    ExchangeToBroker(ExchangeToBroker<BrokerID, Symbol, Settlement>, ExchangeID),

    BrokerWakeUp(BrokerID),

    BrokerToTrader(BrokerToTrader<TraderID, ExchangeID, Symbol, Settlement>, BrokerID),

    TraderWakeUp(TraderID),

    TraderToBroker(TraderToBroker<BrokerID, ExchangeID, Symbol, Settlement>, TraderID),
}

pub struct KernelBuilder<
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag,
    T: Trader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>,
    B: Broker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>,
    E: Exchange<ExchangeID, BrokerID, Symbol, Settlement>,
    R: Replay<ExchangeID, Symbol, Settlement>,
    RNG: SeedableRng + Rng
> {
    traders: HashMap<TraderID, T>,
    brokers: HashMap<BrokerID, B>,
    exchanges: HashMap<ExchangeID, E>,
    replay: R,

    start_dt: DateTime,
    end_dt: DateTime,

    seed: Option<u64>,

    rng: PhantomData<RNG>,
    symbol: PhantomData<Symbol>,
    settlement: PhantomData<Settlement>,
}

impl<
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag,
    T: Trader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>,
    B: Broker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>,
    E: Exchange<ExchangeID, BrokerID, Symbol, Settlement>,
    R: Replay<ExchangeID, Symbol, Settlement>
>
KernelBuilder<TraderID, BrokerID, ExchangeID, Symbol, Settlement, T, B, E, R, StdRng>
{
    pub fn new<CE, CB, SC>(exchanges: impl IntoIterator<Item=E>,
                           brokers: impl IntoIterator<Item=(B, CE)>,
                           traders: impl IntoIterator<Item=(T, CB)>,
                           replay: R,
                           date_range: (DateTime, DateTime)) -> Self
        where
            CE: IntoIterator<Item=ExchangeID>,      // Connected Exchanges
            CB: IntoIterator<Item=(BrokerID, SC)>,  // Connected Brokers
            SC: IntoIterator<Item=SubscriptionConfig<ExchangeID, Symbol, Settlement>>
    {
        let (start_dt, end_dt) = date_range;
        if end_dt < start_dt {
            panic!("start_dt ({start_dt}) is less than end_dt ({end_dt})")
        }
        let exchanges: Vec<_> = exchanges.into_iter().collect();
        let n_exchanges = exchanges.len();
        let mut exchanges: HashMap<ExchangeID, E> = exchanges.into_iter()
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
        let mut brokers: HashMap<BrokerID, B> = brokers.into_iter()
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
        let traders: HashMap<TraderID, T> = traders.into_iter()
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
            rng: Default::default(),
            symbol: Default::default(),
            settlement: Default::default(),
        }
    }

    pub fn with_rng<RNG: Rng + SeedableRng>(self) -> KernelBuilder<
        TraderID, BrokerID, ExchangeID, Symbol, Settlement, T, B, E, R, RNG
    > {
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
            rng: Default::default(),
            symbol: Default::default(),
            settlement: Default::default(),
        }
    }
}

impl<
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag,
    T: Trader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>,
    B: Broker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>,
    E: Exchange<ExchangeID, BrokerID, Symbol, Settlement>,
    R: Replay<ExchangeID, Symbol, Settlement>,
    RNG: Rng + SeedableRng
>
KernelBuilder<TraderID, BrokerID, ExchangeID, Symbol, Settlement, T, B, E, R, RNG>
{
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn build(self) -> Kernel<
        TraderID, BrokerID, ExchangeID, Symbol, Settlement, T, B, E, R, RNG
    > {
        let KernelBuilder {
            traders, brokers, exchanges, mut replay, end_dt, start_dt, seed, ..
        } = self;

        *replay.current_datetime_mut() = start_dt;
        let first_message = Kernel::<
            TraderID, BrokerID, ExchangeID, Symbol, Settlement, T, B, E, R, RNG
        >::process_replay_action(replay.next().expect("Replay does not contain any entries"));
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

impl<
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag,
    T: Trader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>,
    B: Broker<BrokerID, TraderID, ExchangeID, Symbol, Settlement>,
    E: Exchange<ExchangeID, BrokerID, Symbol, Settlement>,
    R: Replay<ExchangeID, Symbol, Settlement>,
    RNG: SeedableRng + Rng
>
Kernel<TraderID, BrokerID, ExchangeID, Symbol, Settlement, T, B, E, R, RNG>
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

    fn handle_message(
        &mut self,
        message: MessageContent<ExchangeID, BrokerID, TraderID, Symbol, Settlement>)
    {
        match message
        {
            MessageContent::ReplayToExchange(replay_request) => {
                if let Some(action) = self.replay.next() {
                    if action.datetime < self.current_dt {
                        panic!(
                            "Replay yielded action {action:?} which DateTime ({}) \
                            is less than the Kernel current DateTime ({})",
                            action.datetime,
                            self.current_dt
                        )
                    }
                    self.message_queue.push(Self::process_replay_action(action))
                }
                self.handle_replay_to_exchange(replay_request)
            }
            MessageContent::ExchangeToReplay(reply, exchange_id) => {
                self.handle_exchange_to_replay(reply, exchange_id)
            }
            MessageContent::BrokerToExchange(request, broker_id) => {
                self.handle_broker_to_exchange(request, broker_id)
            }
            MessageContent::ExchangeToBroker(reply, exchange_id) => {
                self.handle_exchange_to_broker(reply, exchange_id)
            }
            MessageContent::BrokerWakeUp(broker_id) => {
                self.handle_broker_wakeup(broker_id)
            }
            MessageContent::BrokerToTrader(reply, broker_id) => {
                self.handle_broker_to_trader(reply, broker_id)
            }
            MessageContent::TraderWakeUp(trader_id) => {
                self.handle_trader_wakeup(trader_id)
            }
            MessageContent::TraderToBroker(request, trader_id) => {
                self.handle_trader_to_broker(request, trader_id)
            }
        }
    }

    fn handle_replay_to_exchange(
        &mut self,
        request: ReplayToExchange<ExchangeID, Symbol, Settlement>)
    {
        let exchange = self.exchanges.get_mut(&request.exchange_id).expect_with(
            || panic!("Kernel does not know such an Exchange: {}", request.exchange_id)
        );
        *exchange.current_datetime_mut() = self.current_dt;
        let messages = exchange.process_replay_request(request.content)
            .into_iter()
            .map(
                |action| Self::process_exchange_action(
                    self.current_dt,
                    &mut self.brokers,
                    &mut self.rng,
                    action,
                    request.exchange_id,
                )
            );
        self.message_queue.extend(messages)
    }

    fn handle_exchange_to_replay(
        &mut self,
        reply: ExchangeToReplay<Symbol, Settlement>,
        exchange_id: ExchangeID)
    {
        *self.replay.current_datetime_mut() = self.current_dt;
        let messages = self.replay.handle_exchange_reply(reply, exchange_id, &mut self.rng)
            .into_iter()
            .map(Self::process_replay_action);
        self.message_queue.extend(messages)
    }

    fn handle_broker_to_exchange(
        &mut self,
        request: BrokerToExchange<ExchangeID, Symbol, Settlement>,
        broker_id: BrokerID)
    {
        let exchange = self.exchanges.get_mut(&request.exchange_id).expect_with(
            || panic!("Kernel does not know such an Exchange: {}", request.exchange_id)
        );
        *exchange.current_datetime_mut() = self.current_dt;
        let messages = exchange.process_broker_request(request.content, broker_id)
            .into_iter()
            .map(
                |action| Self::process_exchange_action(
                    self.current_dt,
                    &mut self.brokers,
                    &mut self.rng,
                    action,
                    request.exchange_id,
                )
            );
        self.message_queue.extend(messages)
    }

    fn handle_exchange_to_broker(
        &mut self,
        reply: ExchangeToBroker<BrokerID, Symbol, Settlement>,
        exchange_id: ExchangeID)
    {
        let broker = self.brokers.get_mut(&reply.broker_id).expect_with(
            || panic!("Kernel does not know such a Broker: {}", reply.broker_id)
        );
        *broker.current_datetime_mut() = self.current_dt;
        let messages = broker.process_exchange_reply(reply.content, exchange_id, reply.exchange_dt)
            .into_iter()
            .map(
                |action| Self::process_broker_action(
                    self.current_dt,
                    &mut self.traders,
                    &mut self.rng,
                    broker,
                    action,
                    reply.broker_id,
                )
            );
        self.message_queue.extend(messages)
    }

    fn handle_broker_wakeup(&mut self, broker_id: BrokerID)
    {
        let broker = self.brokers.get_mut(&broker_id).expect_with(
            || panic!("Kernel does not know such a Broker: {broker_id}")
        );
        *broker.current_datetime_mut() = self.current_dt;
        let messages = broker.wakeup()
            .into_iter()
            .map(
                |action| Self::process_broker_action(
                    self.current_dt,
                    &mut self.traders,
                    &mut self.rng,
                    broker,
                    action,
                    broker_id,
                )
            );
        self.message_queue.extend(messages)
    }

    fn handle_broker_to_trader(
        &mut self,
        reply: BrokerToTrader<TraderID, ExchangeID, Symbol, Settlement>,
        broker_id: BrokerID)
    {
        let trader = self.traders.get_mut(&reply.trader_id).expect_with(
            || panic!("Kernel does not know such a Trader: {}", reply.trader_id)
        );
        *trader.current_datetime_mut() = self.current_dt;
        let messages = trader.process_broker_reply(
            reply.content, broker_id, reply.exchange_id, reply.event_dt,
        )
            .into_iter()
            .map(
                |action| Self::process_trader_action(
                    self.current_dt,
                    &mut self.rng,
                    trader,
                    action,
                    reply.trader_id,
                )
            );
        self.message_queue.extend(messages)
    }

    fn handle_trader_wakeup(&mut self, trader_id: TraderID)
    {
        let trader = self.traders.get_mut(&trader_id).expect_with(
            || panic!("Kernel does not know such a Trader: {trader_id}")
        );
        *trader.current_datetime_mut() = self.current_dt;
        let messages = trader.wakeup()
            .into_iter()
            .map(
                |action| Self::process_trader_action(
                    self.current_dt,
                    &mut self.rng,
                    trader,
                    action,
                    trader_id,
                )
            );
        self.message_queue.extend(messages)
    }

    fn handle_trader_to_broker(
        &mut self,
        request: TraderToBroker<BrokerID, ExchangeID, Symbol, Settlement>,
        trader_id: TraderID)
    {
        let broker = self.brokers.get_mut(&request.broker_id).expect_with(
            || panic!("Kernel does not know such an Broker: {}", request.broker_id)
        );
        *broker.current_datetime_mut() = self.current_dt;
        let messages = broker.process_trader_request(request.content, trader_id)
            .into_iter()
            .map(
                |action| Self::process_broker_action(
                    self.current_dt,
                    &mut self.traders,
                    &mut self.rng,
                    broker,
                    action,
                    request.broker_id,
                )
            );
        self.message_queue.extend(messages)
    }

    fn process_exchange_action(
        current_dt: DateTime,
        brokers: &mut HashMap<BrokerID, B>,
        rng: &mut RNG,
        action: ExchangeAction<BrokerID, Symbol, Settlement>,
        exchange_id: ExchangeID) -> Message<ExchangeID, BrokerID, TraderID, Symbol, Settlement>
    {
        let delayed_dt = current_dt + Duration::nanoseconds(action.delay as i64);
        let (datetime, body) = match action.content
        {
            ExchangeActionKind::ExchangeToBroker(reply) => {
                let broker = brokers.get_mut(&reply.broker_id).expect_with(
                    || panic!("Kernel does not know such a Broker: {}", reply.broker_id)
                );
                *broker.current_datetime_mut() = current_dt;
                let latency = broker.exchange_to_broker_latency(exchange_id, rng, delayed_dt);
                (
                    delayed_dt + Duration::nanoseconds(latency as i64),
                    MessageContent::ExchangeToBroker(reply, exchange_id)
                )
            }
            ExchangeActionKind::ExchangeToReplay(reply) => {
                (
                    delayed_dt,
                    MessageContent::ExchangeToReplay(reply, exchange_id)
                )
            }
        };
        Message { datetime, body }
    }

    fn process_replay_action(
        action: ReplayAction<ExchangeID, Symbol, Settlement>) -> Message<
        ExchangeID, BrokerID, TraderID, Symbol, Settlement
    > {
        Message {
            datetime: action.datetime,
            body: MessageContent::ReplayToExchange(action.content),
        }
    }

    fn process_broker_action(
        current_dt: DateTime,
        traders: &mut HashMap<TraderID, T>,
        rng: &mut RNG,
        broker: &mut B,
        action: BrokerAction<TraderID, ExchangeID, Symbol, Settlement>,
        broker_id: BrokerID) -> Message<ExchangeID, BrokerID, TraderID, Symbol, Settlement>
    {
        let delayed_dt = current_dt + Duration::nanoseconds(action.delay as i64);
        let (datetime, body) = match action.content
        {
            BrokerActionKind::BrokerToTrader(reply) => {
                let trader = traders.get_mut(&reply.trader_id).expect_with(
                    || panic!("Kernel does not know such a Trader: {}", reply.trader_id)
                );
                *trader.current_datetime_mut() = current_dt;
                let latency = trader.broker_to_trader_latency(broker_id, rng, delayed_dt);
                (
                    delayed_dt + Duration::nanoseconds(latency as i64),
                    MessageContent::BrokerToTrader(reply, broker_id)
                )
            }
            BrokerActionKind::BrokerToExchange(request) => {
                let latency = broker.broker_to_exchange_latency(
                    request.exchange_id, rng, delayed_dt,
                );
                (
                    delayed_dt + Duration::nanoseconds(latency as i64),
                    MessageContent::BrokerToExchange(request, broker_id)
                )
            }
            BrokerActionKind::WakeUp => {
                (
                    delayed_dt,
                    MessageContent::BrokerWakeUp(broker_id)
                )
            }
        };
        Message { datetime, body }
    }

    fn process_trader_action(
        current_dt: DateTime,
        rng: &mut RNG,
        trader: &mut T,
        action: TraderAction<BrokerID, ExchangeID, Symbol, Settlement>,
        trader_id: TraderID) -> Message<ExchangeID, BrokerID, TraderID, Symbol, Settlement>
    {
        let delayed_dt = current_dt + Duration::nanoseconds(action.delay as i64);
        let (datetime, body) = match action.content
        {
            TraderActionKind::TraderToBroker(request) => {
                let latency = trader.trader_to_broker_latency(request.broker_id, rng, delayed_dt);
                (
                    delayed_dt + Duration::nanoseconds(latency as i64),
                    MessageContent::TraderToBroker(request, trader_id)
                )
            }
            TraderActionKind::WakeUp => {
                (
                    delayed_dt,
                    MessageContent::TraderWakeUp(trader_id)
                )
            }
        };
        Message { datetime, body }
    }
}