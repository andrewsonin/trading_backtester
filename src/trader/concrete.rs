use {
    crate::{
        broker::{
            BrokerToTrader,
            reply::{BasicBrokerReply, BasicBrokerToTrader},
        },
        exchange::reply::ExchangeEventNotification,
        kernel::LatentActionProcessor,
        latency::{concrete::ConstantLatency, Latent},
        settlement::GetSettlementLag,
        trader::{
            request::BasicTraderToBroker,
            Trader,
            TraderAction,
            TraderToBroker,
            TraderToItself,
        },
        types::{Agent, Date, DateTime, Id, Named, Nothing, ObState, PriceStep, Size, TimeSync},
        utils::queue::MessageReceiver,
    },
    rand::Rng,
    std::{fs::File, io::Write, marker::PhantomData, path::Path},
};

pub struct VoidTrader<
    TraderID: Id,
    BrokerID: Id,
    B2T: BrokerToTrader<TraderID=TraderID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    T2T: TraderToItself
> {
    name: TraderID,
    current_dt: DateTime,
    phantom: PhantomData<(BrokerID, B2T, T2B, T2T)>,
}

impl<
    TraderID: Id,
    BrokerID: Id,
    B2T: BrokerToTrader<TraderID=TraderID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    T2T: TraderToItself
>
VoidTrader<TraderID, BrokerID, B2T, T2B, T2T>
{
    pub fn new(name: TraderID) -> Self {
        VoidTrader {
            name,
            current_dt: Date::from_ymd(1970, 1, 1).and_hms(0, 0, 0),
            phantom: Default::default(),
        }
    }
}

impl<
    TraderID: Id,
    BrokerID: Id,
    B2T: BrokerToTrader<TraderID=TraderID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    T2T: TraderToItself
>
TimeSync for VoidTrader<TraderID, BrokerID, B2T, T2B, T2T>
{
    fn current_datetime_mut(&mut self) -> &mut DateTime { &mut self.current_dt }
}

impl<
    TraderID: Id,
    BrokerID: Id,
    B2T: BrokerToTrader<TraderID=TraderID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    T2T: TraderToItself
>
Named<TraderID> for VoidTrader<TraderID, BrokerID, B2T, T2B, T2T>
{
    fn get_name(&self) -> TraderID { self.name }
}

impl<
    TraderID: Id,
    BrokerID: Id,
    B2T: BrokerToTrader<TraderID=TraderID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    T2T: TraderToItself
>
Agent for VoidTrader<TraderID, BrokerID, B2T, T2B, T2T>
{
    type Action = TraderAction<T2B, T2T>;
}

impl<
    TraderID: Id,
    BrokerID: Id,
    B2T: BrokerToTrader<TraderID=TraderID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    T2T: TraderToItself
>
Latent for VoidTrader<TraderID, BrokerID, B2T, T2B, T2T>
{
    type OuterID = BrokerID;
    type LatencyGenerator = ConstantLatency<BrokerID, 0, 0>;

    fn get_latency_generator(&self) -> Self::LatencyGenerator {
        ConstantLatency::<BrokerID, 0, 0>::new()
    }
}

impl<
    TraderID: Id,
    BrokerID: Id,
    B2T: BrokerToTrader<TraderID=TraderID>,
    T2B: TraderToBroker<BrokerID=BrokerID>,
    T2T: TraderToItself
>
Trader for VoidTrader<TraderID, BrokerID, B2T, T2B, T2T>
{
    type TraderID = TraderID;
    type BrokerID = BrokerID;

    type B2T = B2T;
    type T2T = T2T;
    type T2B = T2B;

    fn wakeup<KerMsg: Ord, RNG: Rng>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl LatentActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
        _: T2T,
        _: &mut RNG,
    ) {}

    fn process_broker_reply<KerMsg: Ord, RNG: Rng>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl LatentActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
        _: B2T,
        _: BrokerID,
        _: &mut RNG,
    ) {}

    fn upon_register_at_broker(&mut self, _: BrokerID) {}
}

pub struct SpreadWriter<
    TraderID: Id,
    BrokerID: Id,
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
> {
    name: TraderID,
    current_dt: DateTime,
    price_step: PriceStep,
    file: File,
    phantom: PhantomData<(BrokerID, ExchangeID, Symbol, Settlement)>,
}

impl<TraderID: Id, BrokerID: Id, ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
{
    pub fn new(name: TraderID, price_step: impl Into<PriceStep>, file: impl AsRef<Path>) -> Self {
        let file = file.as_ref();
        let file = File::create(file).unwrap_or_else(
            |err| panic!("Cannot create file {file:?}. Error: {err}")
        );
        writeln!(&file, "Timestamp,BID_PRICE,BID_SIZE,ASK_PRICE,ASK_SIZE")
            .unwrap_or_else(|err| panic!("Cannot write to file {file:?}. Error: {err}"));
        SpreadWriter {
            name,
            current_dt: Date::from_ymd(1970, 1, 1).and_hms(0, 0, 0),
            price_step: price_step.into(),
            file,
            phantom: Default::default(),
        }
    }
}

impl<TraderID: Id, BrokerID: Id, ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
TimeSync for SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
{
    fn current_datetime_mut(&mut self) -> &mut DateTime { &mut self.current_dt }
}

impl<TraderID: Id, BrokerID: Id, ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
Named<TraderID> for SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
{
    fn get_name(&self) -> TraderID { self.name }
}

impl<TraderID: Id, BrokerID: Id, ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
Agent for SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
{
    type Action = TraderAction<
        BasicTraderToBroker<BrokerID, ExchangeID, Symbol, Settlement>,
        Nothing
    >;
}

impl<
    TraderID: Id,
    BrokerID: Id,
    ExchangeID: Id,
    Symbol: Id,
    Settlement: GetSettlementLag
>
Latent
for SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
{
    type OuterID = BrokerID;
    type LatencyGenerator = ConstantLatency<BrokerID, 0, 0>;

    fn get_latency_generator(&self) -> Self::LatencyGenerator {
        ConstantLatency::<BrokerID, 0, 0>::new()
    }
}

impl<TraderID: Id, BrokerID: Id, ExchangeID: Id, Symbol: Id, Settlement: GetSettlementLag>
Trader
for SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
{
    type TraderID = TraderID;
    type BrokerID = BrokerID;

    type B2T = BasicBrokerToTrader<TraderID, ExchangeID, Symbol, Settlement>;
    type T2T = Nothing;
    type T2B = BasicTraderToBroker<BrokerID, ExchangeID, Symbol, Settlement>;

    fn wakeup<KerMsg: Ord, RNG: Rng>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl LatentActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
        _: Self::T2T,
        _: &mut RNG,
    ) {
        unreachable!("Trader {} did not schedule any wakeups", self.get_name())
    }

    fn process_broker_reply<KerMsg: Ord, RNG: Rng>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl LatentActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
        reply: Self::B2T,
        _: BrokerID,
        _: &mut RNG,
    ) {
        if let BasicBrokerReply::ExchangeEventNotification(
            ExchangeEventNotification::ObSnapshot(snapshot)) = reply.content
        {
            let ObState { bids, asks } = &snapshot.state;
            if let (Some((bid, bids)), Some((ask, asks))) = (bids.first(), asks.first())
            {
                let get_size = |(size, _dt): &_| *size;
                let bid_size: Size = bids.iter().map(get_size).sum();
                let ask_size: Size = asks.iter().map(get_size).sum();
                let bid_price = bid.to_f64(self.price_step);
                let ask_price = ask.to_f64(self.price_step);
                if bid_price >= ask_price {
                    panic!(
                        "Bid price should be lower than Ask price. \
                        Got: {bid_price:.4} {ask_price:.4}"
                    )
                }
                writeln!(
                    self.file,
                    "{},{bid_price:.4},{bid_size},{ask_price:.4},{ask_size}",
                    reply.event_dt
                )
                    .unwrap_or_else(
                        |err| panic!("Cannot write to file {:?}. Error: {err}", self.file)
                    )
            }
        }
    }

    fn upon_register_at_broker(&mut self, _: BrokerID) {}
}