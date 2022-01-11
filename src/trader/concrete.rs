use crate::{
    broker::reply::BrokerReply,
    trader::{Trader, TraderAction},
    types::{Date, DateTime, Identifier, Named, StdRng, TimeSync},
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