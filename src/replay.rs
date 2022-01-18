use crate::{
    exchange::reply::ExchangeToReplay,
    replay::request::ReplayToExchange,
    settlement::GetSettlementLag,
    types::{DateTime, Identifier, TimeSync},
    utils::{enum_dispatch, queue::MessagePusher, rand::Rng},
};

pub mod request;
pub mod concrete;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct ReplayAction<
    ExchangeID: Identifier,
    Symbol: Identifier,
    Settlement: GetSettlementLag
> {
    pub datetime: DateTime,
    pub content: ReplayToExchange<ExchangeID, Symbol, Settlement>,
}

#[enum_dispatch]
pub trait Replay<ExchangeID, Symbol, Settlement>: TimeSync + Iterator<
    Item=ReplayAction<ExchangeID, Symbol, Settlement>
>
    where ExchangeID: Identifier,
          Symbol: Identifier,
          Settlement: GetSettlementLag
{
    fn handle_exchange_reply<KernelMessage: Ord>(
        &mut self,
        message_pusher: MessagePusher<KernelMessage>,
        process_action: impl Fn(Self::Item) -> KernelMessage,
        reply: ExchangeToReplay<Symbol, Settlement>,
        exchange_id: ExchangeID,
        rng: &mut impl Rng,
    );
}

#[macro_export]
macro_rules! replay_enum {
    // replay_enum! {
    //     ReplayEnum<
    //         ExchangeID: Identifier,
    //         Symbol: Identifier,
    //         Settlement: GetSettlementLag,
    //         ObSnapshotDelay: Sized + Copy
    //     > where
    //         ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol, Settlement>,
    //         ObSnapshotDelay: Sized
    //     {
    //         OneTickReplay<ExchangeID, Symbol, ObSnapshotDelay, Settlement>
    //     }
    // }
    (
        $enum_name:ident<
            $( $enum_lt:lifetime $( : $enum_clt:lifetime $(+ $enum_dlt:lifetime )* )? ),*
                 $exchange_id:tt $( :   $exchange_clt:tt $(+   $exchange_dlt:tt )* )?  ,
                      $symbol:tt $( :     $symbol_clt:tt $(+     $symbol_dlt:tt )* )?  ,
                  $settlement:tt $( : $settlement_clt:tt $(+ $settlement_dlt:tt )* )?
            $(,   $enum_tt:ident $( :       $enum_ctt:tt $(+       $enum_dtt:tt )* )? ),* $(,)?
        >
        $( where $( $where_typename:ident : $where_ctt:path ),+ $(,)? )? {
            $($type_name:ident $(< $( $type_lt:path ),+ >)?),+ $(,)?
        }
    ) => {
        enum $enum_name<
            $(  $enum_lt $( :       $enum_clt $(+       $enum_dlt )* )? ),*
            $exchange_id $( :   $exchange_clt $(+   $exchange_dlt )* )?  ,
                 $symbol $( :     $symbol_clt $(+     $symbol_dlt )* )?  ,
             $settlement $( : $settlement_clt $(+ $settlement_dlt )* )?  ,
            $(  $enum_tt $( :       $enum_ctt $(+       $enum_dtt )* )? ),*
        >
        $( where $( $where_typename : $where_ctt ),+ )?
        {
            $($type_name ($type_name $(< $( $type_lt ),+ >)?) ),+
        }

        impl<
            $(  $enum_lt $( :       $enum_clt $(+       $enum_dlt )* )? ),*
            $exchange_id $( :   $exchange_clt $(+   $exchange_dlt )* )?  ,
                 $symbol $( :     $symbol_clt $(+     $symbol_dlt )* )?  ,
             $settlement $( : $settlement_clt $(+ $settlement_dlt )* )?  ,
            $(  $enum_tt $( :       $enum_ctt $(+       $enum_dtt )* )? ),*
        >
        TimeSync
        for $enum_name<$($enum_lt),* $exchange_id, $symbol, $settlement, $($enum_tt),*>
        $( where $( $where_typename : $where_ctt ),+ )?
        {
            fn current_datetime_mut(&mut self) -> &mut DateTime {
                match self {
                    $( $enum_name::$type_name(value) => value.current_datetime_mut() ),+
                }
            }
        }

        impl<
            $(  $enum_lt $( :       $enum_clt $(+       $enum_dlt )* )? ),*
            $exchange_id $( :   $exchange_clt $(+   $exchange_dlt )* )?  ,
                 $symbol $( :     $symbol_clt $(+     $symbol_dlt )* )?  ,
             $settlement $( : $settlement_clt $(+ $settlement_dlt )* )?  ,
            $(  $enum_tt $( :       $enum_ctt $(+       $enum_dtt )* )? ),*
        >
        Iterator
        for $enum_name<$($enum_lt),* $exchange_id, $symbol, $settlement, $($enum_tt),*>
        $( where $( $where_typename : $where_ctt ),+ )?
        {
            type Item = ReplayAction<$exchange_id, $symbol, $settlement>;

            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    $( $enum_name::$type_name(value) => value.next() ),+
                }
            }
        }

        impl<
            $(  $enum_lt $( :       $enum_clt $(+       $enum_dlt )* )? ),*
            $exchange_id $( :   $exchange_clt $(+   $exchange_dlt )* )?  ,
                 $symbol $( :     $symbol_clt $(+     $symbol_dlt )* )?  ,
             $settlement $( : $settlement_clt $(+ $settlement_dlt )* )?  ,
            $(  $enum_tt $( :       $enum_ctt $(+       $enum_dtt )* )? ),*
        >
        Replay<$exchange_id, $symbol, $settlement>
        for $enum_name<$($enum_lt),* $exchange_id, $symbol, $settlement, $($enum_tt),*>
        $( where $( $where_typename : $where_ctt ),+ )?
        {
            fn handle_exchange_reply<KernelMessage: Ord>(
                &mut self,
                message_pusher: MessagePusher<KernelMessage>,
                process_action: impl Fn(Self::Item) -> KernelMessage,
                reply: ExchangeToReplay<Symbol, Settlement>,
                exchange_id: ExchangeID,
                rng: &mut impl Rng,
            ) {
                match self {
                    $(
                        $enum_name::$type_name(value) => value.handle_exchange_reply(
                            message_pusher,
                            process_action,
                            reply,
                            exchange_id,
                            rng
                        )
                    ),+
                }
            }
        }

        replay_enum!(
            @impl_from_traits
            ($enum_name<
                $(  $enum_lt $( :       $enum_clt $(+       $enum_dlt )* )? ),*
                $exchange_id $( :   $exchange_clt $(+   $exchange_dlt )* )?  ,
                     $symbol $( :     $symbol_clt $(+     $symbol_dlt )* )?  ,
                 $settlement $( : $settlement_clt $(+ $settlement_dlt )* )?  ,
                $(  $enum_tt $( :       $enum_ctt $(+       $enum_dtt )* )? ),*
            >
            $( where $( $where_typename : $where_ctt ),+ )? )
            @
            ($($type_name $(< $( $type_lt ),+ >)?),+)
        );
    };
    (
        @impl_from_traits
        ($enum_name:ident<
            $( $enum_lt:lifetime $( : $enum_clt:lifetime $(+ $enum_dlt:lifetime )* )? ),*
                 $exchange_id:tt $( :   $exchange_clt:tt $(+   $exchange_dlt:tt )* )?  ,
                      $symbol:tt $( :     $symbol_clt:tt $(+     $symbol_dlt:tt )* )?  ,
                  $settlement:tt $( : $settlement_clt:tt $(+ $settlement_dlt:tt )* )?  ,
            $(    $enum_tt:ident $( :       $enum_ctt:tt $(+       $enum_dtt:tt )* )? ),*
        >
        $( where $( $where_typename:ident : $where_ctt:path ),+ )? )
        @
        ($type_name:ident $(< $( $type_lt:path ),+ >)?)
    ) => {
        impl<
            $(  $enum_lt $( :       $enum_clt $(+       $enum_dlt )* )? ),*
            $exchange_id $( :   $exchange_clt $(+   $exchange_dlt )* )?  ,
                 $symbol $( :     $symbol_clt $(+     $symbol_dlt )* )?  ,
             $settlement $( : $settlement_clt $(+ $settlement_dlt )* )?  ,
            $(  $enum_tt $( :       $enum_ctt $(+       $enum_dtt )* )? ),*
        >
        From< $type_name $(< $( $type_lt ),+ >)? >
        for $enum_name<$($enum_lt),* $exchange_id, $symbol, $settlement, $($enum_tt),*>
        $( where $( $where_typename : $where_ctt ),+ )?
        {
            fn from(value: $type_name $(< $( $type_lt ),+ >)?) -> Self {
                Self::$type_name(value)
            }
        }
    }
}