use {
    crate::{
        broker::reply::BrokerReply,
        exchange::reply::ExchangeEventNotification,
        trader::{Trader, TraderAction},
        types::{Date, DateTime, Identifier, Named, ObState, PriceStep, Size, StdRng, TimeSync},
        utils::ExpectWith,
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

impl<TraderID: Identifier, BrokerID: Identifier, ExchangeID: Identifier, Symbol: Identifier>
Trader<TraderID, BrokerID, ExchangeID, Symbol>
for VoidTrader<TraderID>
{
    fn process_broker_reply(
        &mut self,
        _: BrokerReply<Symbol>,
        _: BrokerID,
        _: ExchangeID,
        _: DateTime) -> Vec<TraderAction<BrokerID, ExchangeID, Symbol>>
    {
        vec![]
    }
    fn wakeup(&mut self) -> Vec<TraderAction<BrokerID, ExchangeID, Symbol>> {
        vec![]
    }
    fn broker_to_trader_latency(&self, _: BrokerID, _: &mut StdRng, _: DateTime) -> u64 { 0 }
    fn trader_to_broker_latency(&self, _: BrokerID, _: &mut StdRng, _: DateTime) -> u64 { 0 }
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
        let file = File::create(file).expect_with(|| panic!("Cannot create file {:?}", file));
        writeln!(&file, "Timestamp,BID_PRICE,BID_SIZE,ASK_PRICE,ASK_SIZE")
            .expect_with(|| panic!("Cannot write to file {:?}", file));
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

impl<TraderID: Identifier, BrokerID: Identifier, ExchangeID: Identifier, Symbol: Identifier>
Trader<TraderID, BrokerID, ExchangeID, Symbol>
for SpreadWriter<TraderID>
{
    fn process_broker_reply(
        &mut self,
        reply: BrokerReply<Symbol>,
        _: BrokerID,
        _: ExchangeID,
        event_dt: DateTime) -> Vec<TraderAction<BrokerID, ExchangeID, Symbol>>
    {
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
                        "Bid price should be lower than Ask price. Got: {:.4} {:.4}",
                        bid_price, ask_price
                    )
                }
                writeln!(
                    self.file,
                    "{},{:.4},{},{:.4},{}",
                    event_dt, bid_price, bid_size, ask_price, ask_size
                )
                    .expect_with(|| panic!("Cannot write to file {:?}", self.file))
            }
        };
        vec![]
    }
    fn wakeup(&mut self) -> Vec<TraderAction<BrokerID, ExchangeID, Symbol>> {
        unreachable!("Trader {} did not schedule any wakeups", self.get_name())
    }
    fn broker_to_trader_latency(&self, _: BrokerID, _: &mut StdRng, _: DateTime) -> u64 { 0 }
    fn trader_to_broker_latency(&self, _: BrokerID, _: &mut StdRng, _: DateTime) -> u64 { 0 }
    fn upon_register_at_broker(&mut self, _: BrokerID) {}
}