use syn::{parse::{ParseStream, Parse}, Ident, Token, DeriveInput, parse2, Error};

const REPLAY_ATTRIBUTE: &str = "replay";

pub(crate) struct ReplayParameters {
    pub exchange_id: Ident,
    pub symbol: Ident,
    pub settlement: Ident,
}

impl Parse for ReplayParameters {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let content;
        syn::parenthesized!(content in input);
        let exchange_id = content.parse()?;
        content.parse::<Token![,]>()?;
        let symbol = content.parse()?;
        content.parse::<Token![,]>()?;
        let settlement = content.parse()?;
        Ok(ReplayParameters { exchange_id, symbol, settlement })
    }
}

pub(crate) fn parse_replay_attrs(ast: &DeriveInput) -> ReplayParameters
{
    let mut attr_iter = ast.attrs.iter()
        .filter(|a| a.path.segments.len() == 1 && a.path.segments[0].ident == REPLAY_ATTRIBUTE);
    let attr = attr_iter.next().unwrap_or_else(
        || panic!("{REPLAY_ATTRIBUTE} attribute required for deriving Replay")
    );
    if attr_iter.next().is_some() {
        panic!("{REPLAY_ATTRIBUTE} attribute cannot appear more than once")
    }
    parse2(attr.tokens.clone()).unwrap_or_else(
        |attrs| panic!("Invalid {REPLAY_ATTRIBUTE} attributes: {attrs}")
    )
}