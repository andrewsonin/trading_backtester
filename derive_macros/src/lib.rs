use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput, Data};
use quote::quote;
use syn::__private::TokenStream2;
use crate::replay::{
    ReplayParameters,
    parse_replay_attrs,
};

mod replay;

#[proc_macro_derive(Replay, attributes(replay))]
pub fn derive(input: TokenStream) -> TokenStream
{
    let ast = parse_macro_input!(input as DeriveInput);

    let ReplayParameters { exchange_id, symbol, settlement } = parse_replay_attrs(&ast);

    let name = ast.ident;
    let data = ast.data;
    let (handle_exchange_reply, iterator, time_sync) = if let Data::Enum(data) = data
    {
        let mut handle_exchange_reply = TokenStream2::new();
        let mut iterator = TokenStream2::new();
        let mut time_sync = TokenStream2::new();

        data.variants.iter().for_each(
            |variant| {
                let variant_name = &variant.ident;
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
        (handle_exchange_reply, iterator, time_sync)
    } else {
        panic!("Enum type expected. Got {data:?}")
    };
    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics
        Replay<#exchange_id, #symbol, #settlement>
        for #name #ty_generics
        #where_clause
        {
            fn handle_exchange_reply<KernelMessage: Ord>(
                &mut self,
                message_receiver: MessageReceiver<KernelMessage>,
                process_action: impl Fn(Self::Item) -> KernelMessage,
                reply: ExchangeToReplay<Symbol, Settlement>,
                exchange_id: ExchangeID,
                rng: &mut impl Rng,
            ) {
                match self { #handle_exchange_reply }
            }
        }

        impl #impl_generics Iterator for #name #ty_generics
        #where_clause
        {
            type Item = ReplayAction<#exchange_id, #symbol, #settlement>;

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
