use {
    crate::{
        broker::reply::BrokerReply,
        exchange::reply::ExchangeEventNotification,
        settlement::GetSettlementLag,
        trader::{Trader, TraderAction},
        types::{Date, DateTime, Identifier, Named, ObState, PriceStep, Size, TimeSync},
        utils::{ExpectWith, queue::MessagePusher, rand::Rng},
    },
    std::{fs::File, io::Write, path::Path},
};

pub struct VoidTrader<TraderID: Identifier> {
    name: TraderID,
    current_dt: DateTime,
}

impl<TraderID: Identifier> VoidTrader<TraderID>
{
    pub fn new(name: TraderID) -> Self {
        VoidTrader {
            name,
            current_dt: Date::from_ymd(1970, 1, 1).and_hms(0, 0, 0),
        }
    }
}

impl<TraderID: Identifier> TimeSync for VoidTrader<TraderID>
{
    fn current_datetime_mut(&mut self) -> &mut DateTime { &mut self.current_dt }
}

impl<TraderID: Identifier> Named<TraderID> for VoidTrader<TraderID>
{
    fn get_name(&self) -> TraderID { self.name }
}

impl<
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
>
Trader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
for VoidTrader<TraderID>
{
    fn process_broker_reply<KM: Ord>(
        &mut self,
        _: MessagePusher<KM>,
        _: impl FnMut(TraderAction<BrokerID, ExchangeID, Symbol, Settlement>, &Self) -> KM,
        _: BrokerReply<Symbol, Settlement>,
        _: BrokerID,
        _: ExchangeID,
        _: DateTime,
    ) {}

    fn wakeup<KM: Ord>(
        &mut self,
        _: MessagePusher<KM>,
        _: impl FnMut(TraderAction<BrokerID, ExchangeID, Symbol, Settlement>, &Self) -> KM,
    ) {}

    fn broker_to_trader_latency(&self, _: BrokerID, _: &mut impl Rng, _: DateTime) -> u64 { 0 }
    fn trader_to_broker_latency(&self, _: BrokerID, _: &mut impl Rng, _: DateTime) -> u64 { 0 }
    fn upon_register_at_broker(&mut self, _: BrokerID) {}
}

pub struct SpreadWriter<TraderID: Identifier> {
    name: TraderID,
    current_dt: DateTime,
    price_step: PriceStep,
    file: File,
}

impl<TraderID: Identifier> SpreadWriter<TraderID>
{
    pub fn new(name: TraderID, price_step: impl Into<PriceStep>, file: impl AsRef<Path>) -> Self {
        let file = file.as_ref();
        let file = File::create(file).expect_with(|| panic!("Cannot create file {file:?}"));
        writeln!(&file, "Timestamp,BID_PRICE,BID_SIZE,ASK_PRICE,ASK_SIZE")
            .expect_with(|| panic!("Cannot write to file {file:?}"));
        SpreadWriter {
            name,
            current_dt: Date::from_ymd(1970, 1, 1).and_hms(0, 0, 0),
            price_step: price_step.into(),
            file,
        }
    }
}

impl<TraderID: Identifier> TimeSync for SpreadWriter<TraderID> {
    fn current_datetime_mut(&mut self) -> &mut DateTime { &mut self.current_dt }
}

impl<TraderID: Identifier> Named<TraderID> for SpreadWriter<TraderID> {
    fn get_name(&self) -> TraderID { self.name }
}

impl<
    TraderID: Identifier,
    BrokerID: Identifier,
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
>
Trader<TraderID, BrokerID, ExchangeID, Symbol, Settlement>
for SpreadWriter<TraderID>
{
    fn process_broker_reply<KM: Ord>(
        &mut self,
        _: MessagePusher<KM>,
        _: impl FnMut(TraderAction<BrokerID, ExchangeID, Symbol, Settlement>, &Self) -> KM,
        reply: BrokerReply<Symbol, Settlement>,
        _: BrokerID,
        _: ExchangeID,
        event_dt: DateTime,
    ) {
        if let BrokerReply::ExchangeEventNotification(
            ExchangeEventNotification::ObSnapshot(snapshot)) = reply
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
                    "{event_dt},{bid_price:.4},{bid_size},{ask_price:.4},{ask_size}"
                )
                    .expect_with(|| panic!("Cannot write to file {:?}", self.file))
            }
        }
    }

    fn wakeup<KM: Ord>(
        &mut self,
        _: MessagePusher<KM>,
        _: impl FnMut(TraderAction<BrokerID, ExchangeID, Symbol, Settlement>, &Self) -> KM,
    ) {
        unreachable!("Trader {} did not schedule any wakeups", self.get_name())
    }

    fn broker_to_trader_latency(&self, _: BrokerID, _: &mut impl Rng, _: DateTime) -> u64 { 0 }
    fn trader_to_broker_latency(&self, _: BrokerID, _: &mut impl Rng, _: DateTime) -> u64 { 0 }
    fn upon_register_at_broker(&mut self, _: BrokerID) {}
}