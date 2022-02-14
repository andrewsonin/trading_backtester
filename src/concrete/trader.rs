use {
    crate::{
        concrete::{
            latency::ConstantLatency,
            message_protocol::{
                broker::reply::{BasicBrokerReply, BasicBrokerToTrader},
                exchange::reply::ExchangeEventNotification,
                trader::request::BasicTraderToBroker,
            },
            traded_pair::settlement::GetSettlementLag,
            types::{ObState, PriceStep, Size},
        },
        interface::{
            latency::Latent,
            message::{BrokerToTrader, TraderToBroker, TraderToItself},
            trader::{Trader, TraderAction},
        },
        kernel::LatentActionProcessor,
        types::{Agent, Date, DateTime, Id, Named, Nothing, TimeSync},
        utils::queue::MessageReceiver,
    },
    rand::Rng,
    std::{fs::File, io::Write, marker::PhantomData, path::Path},
};

/// Defines trader subscription
/// to pairs (`ExchangeID`, [`TradedPair`](crate::concrete::traded_pair::TradedPair)).
pub mod subscriptions;

/// [`Trader`] that writes best bid-offer to a csv-file whenever it receives OB update.
pub struct SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
    where TraderID: Id,
          BrokerID: Id,
          ExchangeID: Id,
          Symbol: Id,
          Settlement: GetSettlementLag
{
    name: TraderID,
    current_dt: DateTime,
    price_step: PriceStep,
    file: File,
    phantom: PhantomData<(BrokerID, ExchangeID, Symbol, Settlement)>,
}

impl<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
    where TraderID: Id,
          BrokerID: Id,
          ExchangeID: Id,
          Symbol: Id,
          Settlement: GetSettlementLag
{
    /// Creates a new instance of the `SpreadWriter`.
    ///
    /// # Arguments
    ///
    /// * `name` — ID of the `SpreadWriter`.
    /// * `price_step` — Price quotation step.
    /// * `file` — Path to the csv-file to create.
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

impl<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
TimeSync for SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
    where TraderID: Id,
          BrokerID: Id,
          ExchangeID: Id,
          Symbol: Id,
          Settlement: GetSettlementLag
{
    fn current_datetime_mut(&mut self) -> &mut DateTime { &mut self.current_dt }
}

impl<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
Named<TraderID> for SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
    where TraderID: Id,
          BrokerID: Id,
          ExchangeID: Id,
          Symbol: Id,
          Settlement: GetSettlementLag
{
    fn get_name(&self) -> TraderID { self.name }
}

impl<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
Agent for SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
    where TraderID: Id,
          BrokerID: Id,
          ExchangeID: Id,
          Symbol: Id,
          Settlement: GetSettlementLag
{
    type Action = TraderAction<
        BasicTraderToBroker<BrokerID, ExchangeID, Symbol, Settlement>,
        Nothing
    >;
}

impl<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
Latent
for SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
    where TraderID: Id,
          BrokerID: Id,
          ExchangeID: Id,
          Symbol: Id,
          Settlement: GetSettlementLag
{
    type OuterID = BrokerID;
    type LatencyGenerator = ConstantLatency<BrokerID, 0, 0>;

    fn get_latency_generator(&self) -> Self::LatencyGenerator {
        ConstantLatency::<BrokerID, 0, 0>::new()
    }
}

impl<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
Trader
for SpreadWriter<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
    where TraderID: Id,
          BrokerID: Id,
          ExchangeID: Id,
          Symbol: Id,
          Settlement: GetSettlementLag
{
    type TraderID = TraderID;
    type BrokerID = BrokerID;

    type B2T = BasicBrokerToTrader<TraderID, ExchangeID, Symbol, Settlement>;
    type T2T = Nothing;
    type T2B = BasicTraderToBroker<BrokerID, ExchangeID, Symbol, Settlement>;

    fn wakeup<KerMsg: Ord>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl LatentActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
        _: Self::T2T,
        _: &mut impl Rng,
    ) {
        unreachable!("Trader {} did not schedule any wakeups", self.get_name())
    }

    fn process_broker_reply<KerMsg: Ord>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl LatentActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
        reply: Self::B2T,
        _: BrokerID,
        _: &mut impl Rng,
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

/// [`VoidTrader`] that communicates using the default
/// [`message_protocol`](crate::concrete::message_protocol).
pub type BasicVoidTrader<TraderID, BrokerID, ExchangeID, Symbol, Settlement> = VoidTrader<
    TraderID, BrokerID,
    BasicBrokerToTrader<TraderID, ExchangeID, Symbol, Settlement>,
    BasicTraderToBroker<BrokerID, ExchangeID, Symbol, Settlement>,
    Nothing
>;

/// [`Trader`] that is doing nothing.
pub struct VoidTrader<TraderID, BrokerID, B2T, T2B, T2T>
    where
        TraderID: Id,
        BrokerID: Id,
        B2T: BrokerToTrader<TraderID=TraderID>,
        T2B: TraderToBroker<BrokerID=BrokerID>,
        T2T: TraderToItself
{
    name: TraderID,
    current_dt: DateTime,
    phantom: PhantomData<(BrokerID, B2T, T2B, T2T)>,
}

impl<TraderID, BrokerID, B2T, T2B, T2T>
VoidTrader<TraderID, BrokerID, B2T, T2B, T2T>
    where
        TraderID: Id,
        BrokerID: Id,
        B2T: BrokerToTrader<TraderID=TraderID>,
        T2B: TraderToBroker<BrokerID=BrokerID>,
        T2T: TraderToItself
{
    /// Creates a new instance of the `VoidTrader`.
    ///
    /// # Arguments
    ///
    /// * `name` — ID of the `VoidTrader`.
    pub fn new(name: TraderID) -> Self {
        VoidTrader {
            name,
            current_dt: Date::from_ymd(1970, 1, 1).and_hms(0, 0, 0),
            phantom: Default::default(),
        }
    }
}

impl<TraderID, BrokerID, B2T, T2B, T2T>
TimeSync for VoidTrader<TraderID, BrokerID, B2T, T2B, T2T>
    where
        TraderID: Id,
        BrokerID: Id,
        B2T: BrokerToTrader<TraderID=TraderID>,
        T2B: TraderToBroker<BrokerID=BrokerID>,
        T2T: TraderToItself
{
    fn current_datetime_mut(&mut self) -> &mut DateTime { &mut self.current_dt }
}

impl<TraderID, BrokerID, B2T, T2B, T2T>
Named<TraderID> for VoidTrader<TraderID, BrokerID, B2T, T2B, T2T>
    where
        TraderID: Id,
        BrokerID: Id,
        B2T: BrokerToTrader<TraderID=TraderID>,
        T2B: TraderToBroker<BrokerID=BrokerID>,
        T2T: TraderToItself
{
    fn get_name(&self) -> TraderID { self.name }
}

impl<TraderID, BrokerID, B2T, T2B, T2T>
Agent for VoidTrader<TraderID, BrokerID, B2T, T2B, T2T>
    where
        TraderID: Id,
        BrokerID: Id,
        B2T: BrokerToTrader<TraderID=TraderID>,
        T2B: TraderToBroker<BrokerID=BrokerID>,
        T2T: TraderToItself
{
    type Action = TraderAction<T2B, T2T>;
}

impl<TraderID, BrokerID, B2T, T2B, T2T>
Latent for VoidTrader<TraderID, BrokerID, B2T, T2B, T2T>
    where
        TraderID: Id,
        BrokerID: Id,
        B2T: BrokerToTrader<TraderID=TraderID>,
        T2B: TraderToBroker<BrokerID=BrokerID>,
        T2T: TraderToItself
{
    type OuterID = BrokerID;
    type LatencyGenerator = ConstantLatency<BrokerID, 0, 0>;

    fn get_latency_generator(&self) -> Self::LatencyGenerator {
        ConstantLatency::<BrokerID, 0, 0>::new()
    }
}

impl<TraderID, BrokerID, B2T, T2B, T2T>
Trader for VoidTrader<TraderID, BrokerID, B2T, T2B, T2T>
    where
        TraderID: Id,
        BrokerID: Id,
        B2T: BrokerToTrader<TraderID=TraderID>,
        T2B: TraderToBroker<BrokerID=BrokerID>,
        T2T: TraderToItself
{
    type TraderID = TraderID;
    type BrokerID = BrokerID;

    type B2T = B2T;
    type T2T = T2T;
    type T2B = T2B;

    fn wakeup<KerMsg: Ord>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl LatentActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
        _: T2T,
        _: &mut impl Rng,
    ) {}

    fn process_broker_reply<KerMsg: Ord>(
        &mut self,
        _: MessageReceiver<KerMsg>,
        _: impl LatentActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
        _: B2T,
        _: BrokerID,
        _: &mut impl Rng,
    ) {}

    fn upon_register_at_broker(&mut self, _: BrokerID) {}
}