use {
    crate::parsers::{BrokerParameters, ExchangeParameters, ReplayParameters, TraderParameters},
    proc_macro::TokenStream,
    quote::quote,
    std::str::FromStr,
    syn::{
        {Data, DeriveInput, parse2, parse_macro_input},
        __private::TokenStream2,
        parse::Parse,
    },
};

mod parsers;

#[proc_macro_derive(Exchange, attributes(exchange))]
pub fn derive_exchange(input: TokenStream) -> TokenStream
{
    let ast = parse_macro_input!(input as DeriveInput);

    let ExchangeParameters { exchange_id, broker_id, r2e, b2e, e2r, e2b, e2e } = parse_attrs(
        &ast, "exchange", "Exchange",
    );

    let data = ast.data;
    let (
        wakeup, process_broker_request,
        process_replay_request, connect_broker, time_sync, named) = if let Data::Enum(data) = data
    {
        let mut wakeup = TokenStream2::new();
        let mut process_broker_request = TokenStream2::new();
        let mut process_replay_request = TokenStream2::new();
        let mut connect_broker = TokenStream2::new();
        let mut time_sync = TokenStream2::new();
        let mut named = TokenStream2::new();

        data.variants.iter().for_each(
            |variant| {
                let variant_name = &variant.ident;
                wakeup.extend(
                    quote! {
                        Self::#variant_name(v) => v.wakeup(
                            message_receiver,
                            process_action,
                            scheduled_action,
                            rng
                        ),
                    }
                );
                process_broker_request.extend(
                    quote! {
                        Self::#variant_name(v) => v.process_broker_request(
                            message_receiver,
                            process_action,
                            request,
                            broker_id,
                            rng
                        ),
                    }
                );
                process_replay_request.extend(
                    quote! {
                        Self::#variant_name(v) => v.process_replay_request(
                            message_receiver,
                            process_action,
                            request,
                            rng
                        ),
                    }
                );
                connect_broker.extend(quote! {Self::#variant_name(v) => v.connect_broker(broker),});
                time_sync.extend(quote! {Self::#variant_name(v) => v.current_datetime_mut(),});
                named.extend(quote! {Self::#variant_name(v) => v.get_name(),})
            }
        );
        (wakeup, process_broker_request, process_replay_request, connect_broker, time_sync, named)
    } else {
        panic!("Enum type expected. Got {data:?}")
    };
    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let name = ast.ident;
    let tokens = quote! {
        impl #impl_generics
        Exchange<#exchange_id, #broker_id, #r2e, #b2e, #e2r, #e2b, #e2e>
        for #name #ty_generics
        #where_clause
        {
            fn wakeup<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
                scheduled_action: #e2e,
                rng: &mut RNG,
            ) {
                match self { #wakeup }
            }

            fn process_broker_request<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
                request: #b2e,
                broker_id: BrokerID,
                rng: &mut RNG,
            ) {
                match self { #process_broker_request }
            }

            fn process_replay_request<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
                request: #r2e,
                rng: &mut RNG,
            ) {
                match self { #process_replay_request }
            }

            fn connect_broker(&mut self, broker: BrokerID) {
                match self { #connect_broker }
            }
        }

        impl #impl_generics TimeSync for #name #ty_generics
        #where_clause
        {
            fn current_datetime_mut(&mut self) -> &mut DateTime {
                match self { #time_sync }
            }
        }

        impl #impl_generics Named<#exchange_id> for #name #ty_generics
        #where_clause
        {
            fn get_name(&self) -> #exchange_id {
                match self { #named }
            }
        }

        impl #impl_generics Agent for #name #ty_generics
        #where_clause {
            type Action = ExchangeAction<#e2r, #e2b, #e2e>;
        }
    };

    tokens.into()
}

#[proc_macro_derive(Replay, attributes(replay))]
pub fn derive_replay(input: TokenStream) -> TokenStream
{
    let ast = parse_macro_input!(input as DeriveInput);

    let ReplayParameters { exchange_id, e2r, r2r, r2e } = parse_attrs(&ast, "replay", "Replay");

    let data = ast.data;
    let (wakeup, handle_exchange_reply, iterator, time_sync) = if let Data::Enum(data) = data
    {
        let mut wakeup = TokenStream2::new();
        let mut handle_exchange_reply = TokenStream2::new();
        let mut iterator = TokenStream2::new();
        let mut time_sync = TokenStream2::new();

        data.variants.iter().for_each(
            |variant| {
                let variant_name = &variant.ident;
                wakeup.extend(
                    quote! {
                        Self::#variant_name(v) => v.wakeup(
                            message_receiver,
                            process_action,
                            scheduled_action,
                            rng
                        ),
                    }
                );
                handle_exchange_reply.extend(
                    quote! {
                        Self::#variant_name(v) => v.handle_exchange_reply(
                            message_receiver,
                            process_action,
                            reply,
                            exchange_id,
                            rng
                        ),
                    }
                );
                iterator.extend(quote! {Self::#variant_name(v) => v.next(),});
                time_sync.extend(quote! {Self::#variant_name(v) => v.current_datetime_mut(),})
            }
        );
        (wakeup, handle_exchange_reply, iterator, time_sync)
    } else {
        panic!("Enum type expected. Got {data:?}")
    };
    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let name = ast.ident;
    let tokens = quote! {
        impl #impl_generics
        Replay<#exchange_id, #e2r, #r2r, #r2e>
        for #name #ty_generics
        #where_clause
        {
            fn wakeup<KerMsg: Ord>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                process_action: impl Fn(Self::Item) -> KerMsg,
                scheduled_action: #r2r,
                rng: &mut impl Rng,
            ) {
                match self { #wakeup }
            }

            fn handle_exchange_reply<KerMsg: Ord>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                process_action: impl Fn(Self::Item) -> KerMsg,
                reply: #e2r,
                exchange_id: ExchangeID,
                rng: &mut impl Rng,
            ) {
                match self { #handle_exchange_reply }
            }
        }

        impl #impl_generics Iterator for #name #ty_generics
        #where_clause
        {
            type Item = ReplayAction<#r2r, #r2e>;

            fn next(&mut self) -> Option<Self::Item> {
                match self { #iterator }
            }
        }

        impl #impl_generics TimeSync for #name #ty_generics
        #where_clause
        {
            fn current_datetime_mut(&mut self) -> &mut DateTime {
                match self { #time_sync }
            }
        }
    };

    tokens.into()
}

#[proc_macro_derive(Broker, attributes(broker))]
pub fn derive_broker(input: TokenStream) -> TokenStream
{
    let ast = parse_macro_input!(input as DeriveInput);

    let BrokerParameters { broker_id, trader_id, exchange_id, e2b, t2b, b2e, b2t, b2b, sub_cfg }
        = parse_attrs(&ast, "broker", "Broker");

    let data = ast.data;
    let (
        wakeup, process_trader_request,
        process_exchange_reply, upon_connection_to_exchange, register_trader,
        time_sync, named, latency_generator_types,
        outgoing_latency, incoming_latency) = if let Data::Enum(data) = data
    {
        let mut wakeup = TokenStream2::new();
        let mut process_trader_request = TokenStream2::new();
        let mut process_exchange_reply = TokenStream2::new();
        let mut upon_connection_to_exchange = TokenStream2::new();
        let mut register_trader = TokenStream2::new();
        let mut time_sync = TokenStream2::new();
        let mut named = TokenStream2::new();
        let mut latency_generator_types = TokenStream2::new();
        let mut outgoing_latency = TokenStream2::new();
        let mut incoming_latency = TokenStream2::new();

        data.variants.iter().enumerate().for_each(
            |(i, variant)| {
                let variant_name = &variant.ident;

                let first_field_type = &variant.fields.iter().next().expect("No inner fields");

                latency_generator_types.extend(
                    quote! {#variant_name(<#first_field_type as Latent>::LatencyGenerator),}
                );
                outgoing_latency.extend(
                    quote! {
                        Self::#variant_name(v) => v.outgoing_latency(exchange_id, event_dt, rng),
                    }
                );
                incoming_latency.extend(
                    quote! {
                        Self::#variant_name(v) => v.incoming_latency(exchange_id, event_dt, rng),
                    }
                );
                wakeup.extend(
                    quote! {
                        Self::#variant_name(v) => v.wakeup(
                            message_receiver,
                            process_action,
                            scheduled_action,
                            rng
                        ),
                    }
                );
                process_trader_request.extend(
                    quote! {
                        Self::#variant_name(v) => v.process_trader_request(
                            message_receiver,
                            process_action,
                            request,
                            trader_id,
                            rng
                        ),
                    }
                );
                process_exchange_reply.extend(
                    quote! {
                        Self::#variant_name(v) => v.process_exchange_reply(
                            message_receiver,
                            process_action,
                            reply,
                            exchange_id,
                            rng
                        ),
                    }
                );
                upon_connection_to_exchange.extend(
                    quote! {Self::#variant_name(v) => v.upon_connection_to_exchange(exchange_id),}
                );
                register_trader.extend(
                    quote! {Self::#variant_name(v) => v.register_trader(trader_id, sub_cfgs),}
                );
                time_sync.extend(quote! {Self::#variant_name(v) => v.current_datetime_mut(),});
                named.extend(quote! {Self::#variant_name(v) => v.get_name(),})
            }
        );
        (
            wakeup, process_trader_request,
            process_exchange_reply, upon_connection_to_exchange, register_trader,
            time_sync, named, latency_generator_types, outgoing_latency, incoming_latency
        )
    } else {
        panic!("Enum type expected. Got {data:?}")
    };
    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let name = ast.ident;
    let latency_generator_name = TokenStream2::from_str(
        format!("{}LatencyGenerator", name).as_str()
    ).unwrap();
    let tokens = quote! {
        impl #impl_generics
        Broker<#broker_id, #trader_id, #exchange_id, #e2b, #t2b, #b2e, #b2t, #b2b, #sub_cfg>
        for #name #ty_generics
        #where_clause
        {
            fn wakeup<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                process_action: impl FnMut(Self::LatencyGenerator, Self::Action, &mut RNG) -> KerMsg,
                scheduled_action: #b2b,
                rng: &mut RNG,
            ) {
                match self { #wakeup }
            }

            fn process_trader_request<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                process_action: impl FnMut(Self::LatencyGenerator, Self::Action, &mut RNG) -> KerMsg,
                request: #t2b,
                trader_id: TraderID,
                rng: &mut RNG,
            ) {
                match self { #process_trader_request }
            }

            fn process_exchange_reply<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                process_action: impl FnMut(Self::LatencyGenerator, Self::Action, &mut RNG) -> KerMsg,
                reply: #e2b,
                exchange_id: ExchangeID,
                rng: &mut RNG,
            ) {
                match self { #process_exchange_reply }
            }

            fn upon_connection_to_exchange(&mut self, exchange_id: ExchangeID) {
                match self { #upon_connection_to_exchange }
            }

            fn register_trader(
                &mut self,
                trader_id: TraderID,
                sub_cfgs: impl IntoIterator<Item=#sub_cfg>)
            {
                match self { #register_trader }
            }
        }

        enum #latency_generator_name {
            #latency_generator_types
        }

        impl trait LatencyGenerator<#exchange_id>
        {
            fn outgoing_latency<RNG: Rng>(
                &mut self,
                outer_id: #exchange_id,
                event_dt: DateTime,
                rng: &mut RNG) -> u64
            {
                match self { #outgoing_latency }
            }

            fn incoming_latency<RNG: Rng>(
                &mut self,
                outer_id: #exchange_id,
                event_dt: DateTime,
                rng: &mut RNG) -> u64
            {
                match self { #incoming_latency }
            }
        }

        impl #impl_generics TimeSync for #name #ty_generics
        #where_clause
        {
            fn current_datetime_mut(&mut self) -> &mut DateTime {
                match self { #time_sync }
            }
        }

        impl #impl_generics Named<#broker_id> for #name #ty_generics
        #where_clause
        {
            fn get_name(&self) -> #broker_id {
                match self { #named }
            }
        }

        impl #impl_generics Agent for #name #ty_generics
        #where_clause {
            type Action = BrokerAction<#b2e, #b2t, #b2b>;
        }

        impl #impl_generics Latent for #name #ty_generics
        #where_clause {
            type OuterID = #exchange_id;
            type LatencyGenerator = #latency_generator_name;

            fn latency_generator<RNG: Rng>(&self) -> Self::LatencyGenerator {
                //match self { #latency_generator }
            }
        }
    };

    tokens.into()
}

pub(crate) fn parse_attrs<P: Parse>(ast: &DeriveInput, attr_name: &str, derive_name: &str) -> P
{
    let mut attr_iter = ast.attrs.iter()
        .filter(|a| a.path.segments.len() == 1 && a.path.segments[0].ident == attr_name);
    let attr = attr_iter.next().unwrap_or_else(
        || panic!("{attr_name} attribute required for deriving {derive_name}")
    );
    if attr_iter.next().is_some() {
        panic!("{attr_name} attribute cannot appear more than once")
    }
    parse2(attr.tokens.clone()).unwrap_or_else(
        |err| panic!("Invalid {attr_name} attributes: {err}")
    )
}
