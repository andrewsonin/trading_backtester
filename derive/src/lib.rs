use {
    proc_macro::TokenStream,
    quote::quote,
    std::str::FromStr,
    syn::{
        {Data, DeriveInput, Field, Ident, parse_macro_input},
        __private::TokenStream2,
    },
};

#[proc_macro_derive(Trader)]
pub fn derive_trader(input: TokenStream) -> TokenStream
{
    let ast = parse_macro_input!(input as DeriveInput);
    let data = ast.data;
    let data = if let Data::Enum(data) = data {
        data
    } else {
        panic!("Enum type expected. Got {data:?}")
    };

    let get_associated_types = |variant_field: &Field| {
        let as_trait = quote! {<#variant_field as Latent>};
        let outer_id = quote! {#as_trait::OuterID};

        let as_trait = quote! {<#variant_field as Agent>};
        let action = quote! {#as_trait::Action};

        let as_trait = quote! {<#variant_field as Trader>};
        let trader_id = quote! {#as_trait::TraderID};
        let broker_id = quote! {#as_trait::BrokerID};
        let b2t = quote! {#as_trait::B2T};
        let t2t = quote! {#as_trait::T2T};
        let t2b = quote! {#as_trait::T2B};

        (outer_id, action, trader_id, broker_id, b2t, t2t, t2b)
    };

    let name = ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut into_impls = TokenStream2::new();
    let (idents, field_types): (Vec<_>, Vec<_>) = data.variants
        .iter()
        .zip(1..)
        .map(
            |(v, i)| (
                &v.ident,
                v.fields.iter().next().unwrap_or_else(|| panic!("No inner fields for {i} variant"))
            )
        )
        .inspect(
            |(ident, field_type)| into_impls.extend(
                quote! {
                    impl #impl_generics From<#field_type>
                    for #name #ty_generics
                    #where_clause {
                        fn from(value: #field_type) -> Self {
                            Self::#ident(value)
                        }
                    }
                }
            )
        )
        .unzip();

    let first_field_type = field_types.first().expect("No inner fields");
    let (outer_id, action, trader_id, broker_id, b2t, t2t, t2b)
        = get_associated_types(&first_field_type);

    let (mut time_sync,
        mut get_latency, mut latency_generator, mut get_latency_generator,
        mut outgoing_latency, mut incoming_latency,
        mut named, mut wakeup, mut process_broker_reply, mut upon_register_at_broker) = (
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new()
    );

    let process_variant = |(variant_name, variant_field): (&Ident, &Field)| {
        let as_trait = quote! {<#variant_field as Latent>};
        latency_generator.extend(quote! {#variant_name(#as_trait::LatencyGenerator),});
        get_latency_generator.extend(
            quote! {Self::#variant_name(v) =>
                Self::LatencyGenerator::#variant_name(v.get_latency_generator()),
            }
        );

        let match_arm = quote! {Self::#variant_name(v) => v};
        outgoing_latency.extend(quote! {#match_arm.outgoing_latency(outer_id, event_dt, rng),});
        incoming_latency.extend(quote! {#match_arm.incoming_latency(outer_id, event_dt, rng),});

        time_sync.extend(quote! {#match_arm.current_datetime_mut(),});
        get_latency.extend(quote! {#match_arm.get_latency_generator(),});
        named.extend(quote! {#match_arm.get_name(),});

        wakeup.extend(
            quote! {#match_arm.wakeup(message_receiver, action_processor, scheduled_action, rng),}
        );
        process_broker_reply.extend(
            quote! {
                #match_arm.process_broker_reply(
                    message_receiver, action_processor, reply, broker_id, rng
                ),
            }
        );
        upon_register_at_broker.extend(
            quote! {#match_arm.upon_register_at_broker(broker_id),}
        )
    };

    idents.into_iter().zip(field_types.into_iter()).for_each(process_variant);

    let vis = ast.vis;
    let latency_generator_name = TokenStream2::from_str(&format!("{name}LatencyGenerator"))
        .unwrap();

    let tokens = quote! {
        #[derive(Copy, Clone)]
        #vis enum #latency_generator_name #impl_generics
        #where_clause
        {
            #latency_generator
        }

        impl #impl_generics
        LatencyGenerator
        for #latency_generator_name #ty_generics
        #where_clause
        {
            type OuterID = #outer_id;

            #[inline]
            fn outgoing_latency(
                &mut self,
                outer_id: Self::OuterID,
                event_dt: DateTime,
                rng: &mut impl Rng) -> u64
            {
                match self { #outgoing_latency }
            }

            #[inline]
            fn incoming_latency(
                &mut self,
                outer_id: Self::OuterID,
                event_dt: DateTime,
                rng: &mut impl Rng) -> u64
            {
                match self { #incoming_latency }
            }
        }

        impl #impl_generics Trader
        for #name #ty_generics
        #where_clause
        {
            type TraderID = #trader_id;
            type BrokerID = #broker_id;

            type B2T = #b2t;
            type T2T = #t2t;
            type T2B = #t2b;

            #[inline]
            fn wakeup<KerMsg: Ord>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                action_processor: impl LatentActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
                scheduled_action: Self::T2T,
                rng: &mut impl Rng,
            ) {
                match self { #wakeup }
            }

            #[inline]
            fn process_broker_reply<KerMsg: Ord>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                action_processor: impl LatentActionProcessor<Self::Action, Self::BrokerID, KerMsg=KerMsg>,
                reply: Self::B2T,
                broker_id: Self::BrokerID,
                rng: &mut impl Rng,
            ) {
                match self { #process_broker_reply }
            }

            #[inline]
            fn upon_register_at_broker(&mut self, broker_id: Self::BrokerID) {
                match self { #upon_register_at_broker }
            }
        }

        impl #impl_generics TimeSync
        for #name #ty_generics
        #where_clause {
            #[inline]
            fn current_datetime_mut(&mut self) -> &mut DateTime {
                match self { #time_sync }
            }
        }

        impl #impl_generics Latent
        for #name #ty_generics
        #where_clause {
            type OuterID = #outer_id;
            type LatencyGenerator = #latency_generator_name #ty_generics;

            #[inline]
            fn get_latency_generator(&self) -> Self::LatencyGenerator {
                match self { #get_latency_generator }
            }
        }

        impl #impl_generics Named<#trader_id>
        for #name #ty_generics
        #where_clause {
            #[inline]
            fn get_name(&self) -> #trader_id {
                match self { #named }
            }
        }

        impl #impl_generics Agent
        for #name #ty_generics
        #where_clause {
            type Action = #action;
        }

        #into_impls
    };
    tokens.into()
}

#[proc_macro_derive(Broker)]
pub fn derive_broker(input: TokenStream) -> TokenStream
{
    let ast = parse_macro_input!(input as DeriveInput);
    let data = ast.data;
    let data = if let Data::Enum(data) = data {
        data
    } else {
        panic!("Enum type expected. Got {data:?}")
    };

    let get_associated_types = |variant_field: &Field| {
        let as_trait = quote! {<#variant_field as Latent>};
        let outer_id = quote! {#as_trait::OuterID};

        let as_trait = quote! {<#variant_field as Agent>};
        let action = quote! {#as_trait::Action};

        let as_trait = quote! {<#variant_field as Broker>};
        let broker_id = quote! {#as_trait::BrokerID};
        let trader_id = quote! {#as_trait::TraderID};
        let exchange_id = quote! {#as_trait::ExchangeID};
        let r2b = quote! {#as_trait::R2B};
        let e2b = quote! {#as_trait::E2B};
        let t2b = quote! {#as_trait::T2B};
        let b2r = quote! {#as_trait::B2R};
        let b2e = quote! {#as_trait::B2E};
        let b2t = quote! {#as_trait::B2T};
        let b2b = quote! {#as_trait::B2B};
        let sub_cfg = quote! {#as_trait::SubCfg};

        (outer_id, action, broker_id, trader_id, exchange_id,
         r2b, e2b, t2b, b2r, b2e, b2t, b2b, sub_cfg)
    };

    let name = ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut into_impls = TokenStream2::new();
    let (idents, field_types): (Vec<_>, Vec<_>) = data.variants
        .iter()
        .zip(1..)
        .map(
            |(v, i)| (
                &v.ident,
                v.fields.iter().next().unwrap_or_else(|| panic!("No inner fields for {i} variant"))
            )
        )
        .inspect(
            |(ident, field_type)| into_impls.extend(
                quote! {
                    impl #impl_generics From<#field_type>
                    for #name #ty_generics
                    #where_clause {
                        fn from(value: #field_type) -> Self {
                            Self::#ident(value)
                        }
                    }
                }
            )
        )
        .unzip();

    let first_field_type = field_types.first().expect("No inner fields");
    let (outer_id, action, broker_id, trader_id, exchange_id,
        r2b, e2b, t2b, b2r, b2e, b2t, b2b, sub_cfg) = get_associated_types(&first_field_type);

    let (mut time_sync,
        mut get_latency, mut latency_generator, mut get_latency_generator,
        mut outgoing_latency, mut incoming_latency,
        mut named, mut wakeup, mut process_trader_request, mut process_exchange_reply,
        mut process_replay_request, mut upon_connection_to_exchange, mut register_trader) = (
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new()
    );

    let process_variant = |(variant_name, variant_field): (&Ident, &Field)| {
        let as_trait = quote! {<#variant_field as Latent>};
        latency_generator.extend(quote! {#variant_name(#as_trait::LatencyGenerator),});
        get_latency_generator.extend(
            quote! {Self::#variant_name(v) =>
                Self::LatencyGenerator::#variant_name(v.get_latency_generator()),
            }
        );

        let match_arm = quote! {Self::#variant_name(v) => v};
        outgoing_latency.extend(quote! {#match_arm.outgoing_latency(outer_id, event_dt, rng),});
        incoming_latency.extend(quote! {#match_arm.incoming_latency(outer_id, event_dt, rng),});

        time_sync.extend(quote! {#match_arm.current_datetime_mut(),});
        get_latency.extend(quote! {#match_arm.get_latency_generator(),});
        named.extend(quote! {#match_arm.get_name(),});

        wakeup.extend(
            quote! {#match_arm.wakeup(message_receiver, action_processor, scheduled_action, rng),}
        );
        process_trader_request.extend(
            quote! {
                #match_arm.process_trader_request(
                    message_receiver, action_processor, request, trader_id, rng
                ),
            }
        );
        process_exchange_reply.extend(
            quote! {
                #match_arm.process_exchange_reply(
                    message_receiver, action_processor, reply, exchange_id, rng
                ),
            }
        );
        process_replay_request.extend(
            quote! {
                #match_arm.process_replay_request(
                    message_receiver, action_processor, request, rng
                ),
            }
        );
        upon_connection_to_exchange.extend(
            quote! {#match_arm.upon_connection_to_exchange(exchange_id),}
        );
        register_trader.extend(quote! {#match_arm.register_trader(trader_id, sub_cfgs),})
    };

    idents.into_iter().zip(field_types.into_iter()).for_each(process_variant);

    let vis = ast.vis;
    let latency_generator_name = TokenStream2::from_str(&format!("{name}LatencyGenerator"))
        .unwrap();

    let tokens = quote! {
        #[derive(Copy, Clone)]
        #vis enum #latency_generator_name #impl_generics
        #where_clause
        {
            #latency_generator
        }

        impl #impl_generics
        LatencyGenerator
        for #latency_generator_name #ty_generics
        #where_clause
        {
            type OuterID = #outer_id;

            #[inline]
            fn outgoing_latency(
                &mut self,
                outer_id: Self::OuterID,
                event_dt: DateTime,
                rng: &mut impl Rng) -> u64
            {
                match self { #outgoing_latency }
            }

            #[inline]
            fn incoming_latency(
                &mut self,
                outer_id: Self::OuterID,
                event_dt: DateTime,
                rng: &mut impl Rng) -> u64
            {
                match self { #incoming_latency }
            }
        }

        impl #impl_generics Broker
        for #name #ty_generics
        #where_clause
        {
            type BrokerID = #broker_id;
            type TraderID = #trader_id;
            type ExchangeID = #exchange_id;

            type R2B = #r2b;
            type E2B = #e2b;
            type T2B = #t2b;
            type B2R = #b2r;
            type B2E = #b2e;
            type B2T = #b2t;
            type B2B = #b2b;
            type SubCfg = #sub_cfg;

            #[inline]
            fn wakeup<KerMsg: Ord>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                action_processor: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
                scheduled_action: Self::B2B,
                rng: &mut impl Rng,
            ) {
                match self { #wakeup }
            }

            #[inline]
            fn process_trader_request<KerMsg: Ord>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                action_processor: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
                request: Self::T2B,
                trader_id: Self::TraderID,
                rng: &mut impl Rng,
            ) {
                match self { #process_trader_request }
            }

            #[inline]
            fn process_exchange_reply<KerMsg: Ord>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                action_processor: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
                reply: Self::E2B,
                exchange_id: Self::ExchangeID,
                rng: &mut impl Rng,
            ) {
                match self { #process_exchange_reply }
            }

            #[inline]
            fn process_replay_request<KerMsg: Ord>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                action_processor: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
                request: Self::R2B,
                rng: &mut impl Rng,
            ) {
                match self { #process_replay_request }
            }

            #[inline]
            fn upon_connection_to_exchange(&mut self, exchange_id: Self::ExchangeID) {
                match self { #upon_connection_to_exchange }
            }

            #[inline]
            fn register_trader(
                &mut self,
                trader_id: Self::TraderID,
                sub_cfgs: impl IntoIterator<Item=Self::SubCfg>)
            {
                match self { #register_trader }
            }
        }

        impl #impl_generics TimeSync
        for #name #ty_generics
        #where_clause {
            #[inline]
            fn current_datetime_mut(&mut self) -> &mut DateTime {
                match self { #time_sync }
            }
        }

        impl #impl_generics Latent
        for #name #ty_generics
        #where_clause {
            type OuterID = #outer_id;
            type LatencyGenerator = #latency_generator_name #ty_generics;

            #[inline]
            fn get_latency_generator(&self) -> Self::LatencyGenerator {
                match self { #get_latency_generator }
            }
        }

        impl #impl_generics Named<#broker_id>
        for #name #ty_generics
        #where_clause {
            #[inline]
            fn get_name(&self) -> #broker_id {
                match self { #named }
            }
        }

        impl #impl_generics Agent
        for #name #ty_generics
        #where_clause {
            type Action = #action;
        }

        #into_impls
    };
    tokens.into()
}

#[proc_macro_derive(Exchange)]
pub fn derive_exchange(input: TokenStream) -> TokenStream
{
    let ast = parse_macro_input!(input as DeriveInput);
    let data = ast.data;
    let data = if let Data::Enum(data) = data {
        data
    } else {
        panic!("Enum type expected. Got {data:?}")
    };

    let get_associated_types = |variant_field: &Field| {
        let as_trait = quote! {<#variant_field as Agent>};
        let action = quote! {#as_trait::Action};

        let as_trait = quote! {<#variant_field as Exchange>};
        let exchange_id = quote! {#as_trait::ExchangeID};
        let broker_id = quote! {#as_trait::BrokerID};
        let r2e = quote! {#as_trait::R2E};
        let b2e = quote! {#as_trait::B2E};
        let e2r = quote! {#as_trait::E2R};
        let e2b = quote! {#as_trait::E2B};
        let e2e = quote! {#as_trait::E2E};

        (action, exchange_id, broker_id, r2e, b2e, e2r, e2b, e2e)
    };

    let name = ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut into_impls = TokenStream2::new();
    let (idents, field_types): (Vec<_>, Vec<_>) = data.variants
        .iter()
        .zip(1..)
        .map(
            |(v, i)| (
                &v.ident,
                v.fields.iter().next().unwrap_or_else(|| panic!("No inner fields for {i} variant"))
            )
        )
        .inspect(
            |(ident, field_type)| into_impls.extend(
                quote! {
                    impl #impl_generics From<#field_type>
                    for #name #ty_generics
                    #where_clause {
                        fn from(value: #field_type) -> Self {
                            Self::#ident(value)
                        }
                    }
                }
            )
        )
        .unzip();

    let first_field_type = field_types.first().expect("No inner fields");
    let (action, exchange_id, broker_id, r2e, b2e, e2r, e2b, e2e)
        = get_associated_types(&first_field_type);


    let (mut time_sync, mut named, mut wakeup, mut process_broker_request,
        mut process_replay_request, mut connect_broker) = (
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new()
    );

    let process_variant = |variant_name: &Ident| {
        let match_arm = quote! {Self::#variant_name(v) => v};

        time_sync.extend(quote! {#match_arm.current_datetime_mut(),});
        named.extend(quote! {#match_arm.get_name(),});

        wakeup.extend(
            quote! {#match_arm.wakeup(message_receiver, process_action, scheduled_action, rng),}
        );
        process_broker_request.extend(
            quote! {
                #match_arm.process_broker_request(
                    message_receiver, process_action, request, broker_id, rng
                ),
            }
        );
        process_replay_request.extend(
            quote! {
                #match_arm.process_replay_request(message_receiver, process_action, request, rng),
            }
        );
        connect_broker.extend(quote! {#match_arm.connect_broker(broker),})
    };

    idents.into_iter().for_each(process_variant);

    let tokens = quote! {
        impl #impl_generics Exchange
        for #name #ty_generics
        #where_clause
        {
            type ExchangeID = #exchange_id;
            type BrokerID = #broker_id;

            type R2E = #r2e;
            type B2E = #b2e;
            type E2R = #e2r;
            type E2B = #e2b;
            type E2E = #e2e;

            #[inline]
            fn wakeup<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
                scheduled_action: Self::E2E,
                rng: &mut RNG,
            ) {
                match self { #wakeup }
            }

            #[inline]
            fn process_broker_request<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
                request: Self::B2E,
                broker_id: Self::BrokerID,
                rng: &mut RNG,
            ) {
                match self { #process_broker_request }
            }

            #[inline]
            fn process_replay_request<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
                request: Self::R2E,
                rng: &mut RNG,
            ) {
                match self { #process_replay_request }
            }

            #[inline]
            fn connect_broker(&mut self, broker: Self::BrokerID) {
                match self { #connect_broker }
            }
        }

        impl #impl_generics TimeSync
        for #name #ty_generics
        #where_clause {
            #[inline]
            fn current_datetime_mut(&mut self) -> &mut DateTime {
                match self { #time_sync }
            }
        }

        impl #impl_generics Named<#exchange_id>
        for #name #ty_generics
        #where_clause {
            #[inline]
            fn get_name(&self) -> #exchange_id {
                match self { #named }
            }
        }

        impl #impl_generics Agent
        for #name #ty_generics
        #where_clause {
            type Action = #action;
        }

        #into_impls
    };
    tokens.into()
}

#[proc_macro_derive(Replay)]
pub fn derive_replay(input: TokenStream) -> TokenStream
{
    let ast = parse_macro_input!(input as DeriveInput);
    let data = ast.data;
    let data = if let Data::Enum(data) = data {
        data
    } else {
        panic!("Enum type expected. Got {data:?}")
    };

    let get_associated_types = |variant_field: &Field| {
        let as_trait = quote! {<#variant_field as Iterator>};
        let item = quote! {#as_trait::Item};

        let as_trait = quote! {<#variant_field as Replay>};
        let broker_id = quote! {#as_trait::BrokerID};
        let exchange_id = quote! {#as_trait::ExchangeID};
        let e2r = quote! {#as_trait::E2R};
        let b2r = quote! {#as_trait::B2R};
        let r2r = quote! {#as_trait::R2R};
        let r2e = quote! {#as_trait::R2E};
        let r2b = quote! {#as_trait::R2B};

        (item, broker_id, exchange_id, e2r, b2r, r2r, r2e, r2b)
    };

    let name = ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut into_impls = TokenStream2::new();
    let (idents, field_types): (Vec<_>, Vec<_>) = data.variants
        .iter()
        .zip(1..)
        .map(
            |(v, i)| (
                &v.ident,
                v.fields.iter().next().unwrap_or_else(|| panic!("No inner fields for {i} variant"))
            )
        )
        .inspect(
            |(ident, field_type)| into_impls.extend(
                quote! {
                    impl #impl_generics From<#field_type>
                    for #name #ty_generics
                    #where_clause {
                        fn from(value: #field_type) -> Self {
                            Self::#ident(value)
                        }
                    }
                }
            )
        )
        .unzip();

    let first_field_type = field_types.first().expect("No inner fields");
    let (item, broker_id, exchange_id, e2r, b2r, r2r, r2e, r2b)
        = get_associated_types(&first_field_type);


    let (mut time_sync, mut wakeup,
        mut handle_exchange_reply, mut handle_broker_reply, mut next) = (
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new(),
        TokenStream2::new()
    );

    let process_variant = |variant_name: &Ident| {
        let match_arm = quote! {Self::#variant_name(v) => v};

        time_sync.extend(quote! {#match_arm.current_datetime_mut(),});
        wakeup.extend(
            quote! {#match_arm.wakeup(scheduled_action, rng),}
        );
        handle_exchange_reply.extend(
            quote! {#match_arm.handle_exchange_reply(reply, exchange_id, rng),}
        );
        handle_broker_reply.extend(
            quote! {#match_arm.handle_broker_reply(reply, broker_id, rng),}
        );
        next.extend(quote! {#match_arm.next(),})
    };

    idents.into_iter().for_each(process_variant);

    let tokens = quote! {
        #into_impls

        impl #impl_generics Replay
        for #name #ty_generics
        #where_clause
        {
            type BrokerID = #broker_id;
            type ExchangeID = #exchange_id;

            type E2R = #e2r;
            type B2R = #b2r;
            type R2R = #r2r;
            type R2E = #r2e;
            type R2B = #r2b;

            #[inline]
            fn wakeup(
                &mut self,
                scheduled_action: Self::R2R,
                rng: &mut impl Rng,
            ) {
                match self { #wakeup }
            }

            #[inline]
            fn handle_exchange_reply(
                &mut self,
                reply: Self::E2R,
                exchange_id: Self::ExchangeID,
                rng: &mut impl Rng,
            ) {
                match self { #handle_exchange_reply }
            }

            #[inline]
            fn handle_broker_reply(
                &mut self,
                reply: Self::B2R,
                broker_id: Self::BrokerID,
                rng: &mut impl Rng,
            ) {
                match self { #handle_broker_reply }
            }
        }

        impl #impl_generics TimeSync
        for #name #ty_generics
        #where_clause {
            #[inline]
            fn current_datetime_mut(&mut self) -> &mut DateTime {
                match self { #time_sync }
            }
        }

        impl #impl_generics Iterator
        for #name #ty_generics
        #where_clause {
            type Item = #item;
            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                match self { #next }
            }
        }
    };
    tokens.into()
}

#[proc_macro_derive(LatencyGenerator)]
pub fn derive_latency_generator(input: TokenStream) -> TokenStream
{
    let ast = parse_macro_input!(input as DeriveInput);
    let data = ast.data;
    let data = if let Data::Enum(data) = data {
        data
    } else {
        panic!("Enum type expected. Got {data:?}")
    };

    let get_associated_types = |variant_field: &Field| {
        let as_trait = quote! {<#variant_field as LatencyGenerator>};
        quote! {#as_trait::OuterID}
    };

    let name = ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut into_impls = TokenStream2::new();
    let (idents, field_types): (Vec<_>, Vec<_>) = data.variants
        .iter()
        .zip(1..)
        .map(
            |(v, i)| (
                &v.ident,
                v.fields.iter().next().unwrap_or_else(|| panic!("No inner fields for {i} variant"))
            )
        )
        .inspect(
            |(ident, field_type)| into_impls.extend(
                quote! {
                    impl #impl_generics From<#field_type>
                    for #name #ty_generics
                    #where_clause {
                        fn from(value: #field_type) -> Self {
                            Self::#ident(value)
                        }
                    }
                }
            )
        )
        .unzip();

    let first_field_type = field_types.first().expect("No inner fields");
    let outer_id = get_associated_types(&first_field_type);


    let (mut outgoing_latency, mut incoming_latency) = (
        TokenStream2::new(),
        TokenStream2::new()
    );

    let process_variant = |variant_name: &Ident| {
        let match_arm = quote! {Self::#variant_name(v) => v};

        outgoing_latency.extend(quote! { #match_arm.outgoing_latency(outer_id, event_dt, rng), });
        incoming_latency.extend(quote! { #match_arm.incoming_latency(outer_id, event_dt, rng), })
    };

    idents.into_iter().for_each(process_variant);

    let tokens = quote! {
        impl #impl_generics LatencyGenerator
        for #name #ty_generics
        #where_clause
        {
            type OuterID = #outer_id;

            #[inline]
            fn outgoing_latency(
                &mut self,
                outer_id: Self::OuterID,
                event_dt: DateTime,
                rng: &mut impl Rng) -> u64
            {
                match self { #outgoing_latency }
            }

            #[inline]
            fn incoming_latency(
                &mut self,
                outer_id: Self::OuterID,
                event_dt: DateTime,
                rng: &mut impl Rng) -> u64
            {
                match self { #incoming_latency }
            }
        }

        #into_impls
    };
    tokens.into()
}

#[proc_macro_derive(GetSettlementLag)]
pub fn derive_get_settlement_lag(input: TokenStream) -> TokenStream
{
    let ast = parse_macro_input!(input as DeriveInput);
    let data = ast.data;
    let data = if let Data::Enum(data) = data {
        data
    } else {
        panic!("Enum type expected. Got {data:?}")
    };

    let name = ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let (mut into_impls, mut get_settlement_lag) = (TokenStream2::new(), TokenStream2::new());
    data.variants.iter().zip(1..).for_each(
        |(var, i)| {
            let ident = &var.ident;
            let field_type = var.fields.iter().next()
                .unwrap_or_else(|| panic!("No inner fields for {i} variant"));

            let match_arm = quote! {Self::#ident(v) => v};
            get_settlement_lag.extend(quote! { #match_arm.get_settlement_lag(transaction_dt), });
            into_impls.extend(
                quote! {
                    impl #impl_generics From<#field_type>
                    for #name #ty_generics
                    #where_clause {
                        #[inline]
                        fn from(value: #field_type) -> Self {
                            Self::#ident(value)
                        }
                    }
                }
            )
        }
    );

    let tokens = quote! {
        impl #impl_generics GetSettlementLag
        for #name #ty_generics
        #where_clause
        {
            #[inline]
            fn get_settlement_lag(&self, transaction_dt: DateTime) -> u64 {
                match self { #get_settlement_lag }
            }
        }

        #into_impls
    };
    tokens.into()
}