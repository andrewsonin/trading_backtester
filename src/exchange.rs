use crate::{
    broker::request::BrokerRequest,
    exchange::reply::{ExchangeToBroker, ExchangeToReplay},
    replay::request::ReplayRequest,
    settlement::GetSettlementLag,
    types::{Identifier, Named, TimeSync},
    utils::queue::MessageReceiver,
};

pub mod reply;
pub mod concrete;

pub struct ExchangeAction<
    BrokerID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    pub delay: u64,
    pub content: ExchangeActionKind<BrokerID, Symbol, Settlement>,
}

pub enum ExchangeActionKind<
    BrokerID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    ExchangeToBroker(ExchangeToBroker<BrokerID, Symbol, Settlement>),
    ExchangeToReplay(ExchangeToReplay<Symbol, Settlement>),
}

pub trait Exchange<ExchangeID, BrokerID, Symbol, Settlement>: TimeSync + Named<ExchangeID>
    where ExchangeID: Identifier,
          BrokerID: Identifier,
          Symbol: Identifier,
          Settlement: GetSettlementLag
{
    fn process_broker_request<KernelMessage: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KernelMessage>,
        process_action: impl FnMut(ExchangeAction<BrokerID, Symbol, Settlement>) -> KernelMessage,
        request: BrokerRequest<Symbol, Settlement>,
        broker_id: BrokerID,
    );

    fn process_replay_request<KernelMessage: Ord>(
        &mut self,
        message_receiver: MessageReceiver<KernelMessage>,
        process_action: impl FnMut(ExchangeAction<BrokerID, Symbol, Settlement>) -> KernelMessage,
        request: ReplayRequest<Symbol, Settlement>,
    );

    fn connect_broker(&mut self, broker: BrokerID);
}