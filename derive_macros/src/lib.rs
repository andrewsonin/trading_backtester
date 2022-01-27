use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{
        {Data, DeriveInput, Field, Ident, parse_macro_input},
        __private::TokenStream2,
    },
};

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
        let e2b = quote! {#as_trait::E2B};
        let t2b = quote! {#as_trait::T2B};
        let b2e = quote! {#as_trait::B2E};
        let b2t = quote! {#as_trait::B2T};
        let b2b = quote! {#as_trait::B2B};
        let sub_cfg = quote! {#as_trait::SubCfg};

        (outer_id, action, broker_id, trader_id, exchange_id, e2b, t2b, b2e, b2t, b2b, sub_cfg)
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
    let (outer_id, action, broker_id, trader_id, exchange_id, e2b, t2b, b2e, b2t, b2b, sub_cfg)
        = get_associated_types(&first_field_type);

    let (mut time_sync, mut get_latency, mut named,
        mut wakeup, mut process_trader_request,
        mut process_exchange_reply, mut upon_connection_to_exchange, mut register_trader) = (
        TokenStream2::new(),
        TokenStream2::new(),
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
        upon_connection_to_exchange.extend(
            quote! {#match_arm.upon_connection_to_exchange(exchange_id),}
        );
        register_trader.extend(quote! {#match_arm.register_trader(trader_id, sub_cfgs),})
    };

    idents.into_iter().for_each(process_variant);

    let unimplemented_msg = format!("get_latency_generator called for {name}");
    let tokens = quote! {
        #into_impls

        impl #impl_generics Broker
        for #name #ty_generics
        #where_clause
        {
            type BrokerID = #broker_id;
            type TraderID = #trader_id;
            type ExchangeID = #exchange_id;

            type E2B = #e2b;
            type T2B = #t2b;
            type B2E = #b2e;
            type B2T = #b2t;
            type B2B = #b2b;
            type SubCfg = #sub_cfg;

            fn wakeup<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                action_processor: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
                scheduled_action: Self::B2B,
                rng: &mut RNG,
            ) {
                match self { #wakeup }
            }

            fn process_trader_request<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                action_processor: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
                request: Self::T2B,
                trader_id: Self::TraderID,
                rng: &mut RNG,
            ) {
                match self { #process_trader_request }
            }

            fn process_exchange_reply<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                action_processor: impl LatentActionProcessor<Self::Action, Self::ExchangeID, KerMsg=KerMsg>,
                reply: Self::E2B,
                exchange_id: Self::ExchangeID,
                rng: &mut RNG,
            ) {
                match self { #process_exchange_reply }
            }

            fn upon_connection_to_exchange(&mut self, exchange_id: Self::ExchangeID) {
                match self { #upon_connection_to_exchange }
            }

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
            fn current_datetime_mut(&mut self) -> &mut DateTime {
                match self { #time_sync }
            }
        }

        impl #impl_generics Latent
        for #name #ty_generics
        #where_clause {
            type OuterID = #outer_id;
            type LatencyGenerator = Nothing;

            fn get_latency_generator(&self) -> Self::LatencyGenerator {
                unimplemented!(#unimplemented_msg)
            }
        }

        impl #impl_generics Named<#broker_id>
        for #name #ty_generics
        #where_clause {
            fn get_name(&self) -> #broker_id {
                match self { #named }
            }
        }

        impl #impl_generics Agent
        for #name #ty_generics
        #where_clause {
            type Action = #action;
        }
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
        #into_impls

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

            fn wakeup<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
                scheduled_action: Self::E2E,
                rng: &mut RNG,
            ) {
                match self { #wakeup }
            }

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

            fn process_replay_request<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
                request: Self::R2E,
                rng: &mut RNG,
            ) {
                match self { #process_replay_request }
            }

            fn connect_broker(&mut self, broker: Self::BrokerID) {
                match self { #connect_broker }
            }
        }

        impl #impl_generics TimeSync
        for #name #ty_generics
        #where_clause {
            fn current_datetime_mut(&mut self) -> &mut DateTime {
                match self { #time_sync }
            }
        }

        impl #impl_generics Named<#exchange_id>
        for #name #ty_generics
        #where_clause {
            fn get_name(&self) -> #exchange_id {
                match self { #named }
            }
        }

        impl #impl_generics Agent
        for #name #ty_generics
        #where_clause {
            type Action = #action;
        }
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
        #into_impls

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

            fn wakeup<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
                scheduled_action: Self::E2E,
                rng: &mut RNG,
            ) {
                match self { #wakeup }
            }

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

            fn process_replay_request<KerMsg: Ord, RNG: Rng>(
                &mut self,
                message_receiver: MessageReceiver<KerMsg>,
                process_action: impl FnMut(Self::Action, &mut RNG) -> KerMsg,
                request: Self::R2E,
                rng: &mut RNG,
            ) {
                match self { #process_replay_request }
            }

            fn connect_broker(&mut self, broker: Self::BrokerID) {
                match self { #connect_broker }
            }
        }

        impl #impl_generics TimeSync
        for #name #ty_generics
        #where_clause {
            fn current_datetime_mut(&mut self) -> &mut DateTime {
                match self { #time_sync }
            }
        }

        impl #impl_generics Named<#exchange_id>
        for #name #ty_generics
        #where_clause {
            fn get_name(&self) -> #exchange_id {
                match self { #named }
            }
        }

        impl #impl_generics Agent
        for #name #ty_generics
        #where_clause {
            type Action = #action;
        }
    };
    tokens.into()
}