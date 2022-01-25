use {
    crate::replay::ReplayParameters,
    proc_macro::TokenStream,
    quote::quote,
    syn::{
        {Data, DeriveInput, parse2, parse_macro_input},
        __private::TokenStream2,
    },
};

mod replay;

#[proc_macro_derive(Replay, attributes(replay))]
pub fn derive(input: TokenStream) -> TokenStream
{
    let ast = parse_macro_input!(input as DeriveInput);

    let ReplayParameters { exchange_id, e2r, r2r, r2e } = parse_replay_attrs(&ast, "replay");

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

pub(crate) fn parse_replay_attrs(ast: &DeriveInput, replay_attr: &str) -> ReplayParameters
{
    let mut attr_iter = ast.attrs.iter()
        .filter(|a| a.path.segments.len() == 1 && a.path.segments[0].ident == replay_attr);
    let attr = attr_iter.next().unwrap_or_else(
        || panic!("{replay_attr} attribute required for deriving Replay")
    );
    if attr_iter.next().is_some() {
        panic!("{replay_attr} attribute cannot appear more than once")
    }
    parse2(attr.tokens.clone()).unwrap_or_else(
        |attrs| panic!("Invalid {replay_attr} attributes: {attrs}")
    )
}
