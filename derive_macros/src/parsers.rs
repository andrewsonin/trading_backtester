use syn::{Error, parse::{Parse, ParseStream}, Path, Token};

pub(crate) struct ExchangeParameters {
    pub exchange_id: Path,
    pub broker_id: Path,
    pub r2e: Path,
    pub b2e: Path,
    pub e2r: Path,
    pub e2b: Path,
    pub e2e: Path,
}

pub(crate) struct ReplayParameters {
    pub exchange_id: Path,
    pub e2r: Path,
    pub r2r: Path,
    pub r2e: Path,
}

pub(crate) struct BrokerParameters {
    pub broker_id: Path,
    pub trader_id: Path,
    pub exchange_id: Path,
    pub e2b: Path,
    pub t2b: Path,
    pub b2e: Path,
    pub b2t: Path,
    pub b2b: Path,
    pub sub_cfg: Path,
}

pub(crate) struct TraderParameters {
    pub trader_id: Path,
    pub broker_id: Path,
    pub b2t: Path,
    pub t2b: Path,
    pub t2t: Path,
}

impl Parse for ExchangeParameters {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let content;
        syn::parenthesized!(content in input);
        let exchange_id = content.parse()?;
        content.parse::<Token![,]>()?;
        let broker_id = content.parse()?;
        content.parse::<Token![,]>()?;
        let r2e = content.parse()?;
        content.parse::<Token![,]>()?;
        let b2e = content.parse()?;
        content.parse::<Token![,]>()?;
        let e2r = content.parse()?;
        content.parse::<Token![,]>()?;
        let e2b = content.parse()?;
        content.parse::<Token![,]>()?;
        let e2e = content.parse()?;
        Ok(ExchangeParameters { exchange_id, broker_id, r2e, b2e, e2r, e2b, e2e })
    }
}


impl Parse for ReplayParameters {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let content;
        syn::parenthesized!(content in input);
        let exchange_id = content.parse()?;
        content.parse::<Token![,]>()?;
        let e2r = content.parse()?;
        content.parse::<Token![,]>()?;
        let r2r = content.parse()?;
        content.parse::<Token![,]>()?;
        let r2e = content.parse()?;
        Ok(ReplayParameters { exchange_id, e2r, r2r, r2e })
    }
}

impl Parse for BrokerParameters {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let content;
        syn::parenthesized!(content in input);
        let broker_id = content.parse()?;
        content.parse::<Token![,]>()?;
        let trader_id = content.parse()?;
        content.parse::<Token![,]>()?;
        let exchange_id = content.parse()?;
        content.parse::<Token![,]>()?;
        let e2b = content.parse()?;
        content.parse::<Token![,]>()?;
        let t2b = content.parse()?;
        content.parse::<Token![,]>()?;
        let b2e = content.parse()?;
        content.parse::<Token![,]>()?;
        let b2t = content.parse()?;
        content.parse::<Token![,]>()?;
        let b2b = content.parse()?;
        content.parse::<Token![,]>()?;
        let sub_cfg = content.parse()?;
        Ok(BrokerParameters { broker_id, trader_id, exchange_id, e2b, t2b, b2e, b2t, b2b, sub_cfg })
    }
}

impl Parse for TraderParameters {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let content;
        syn::parenthesized!(content in input);
        let trader_id = content.parse()?;
        content.parse::<Token![,]>()?;
        let broker_id = content.parse()?;
        content.parse::<Token![,]>()?;
        let b2t = content.parse()?;
        content.parse::<Token![,]>()?;
        let t2b = content.parse()?;
        content.parse::<Token![,]>()?;
        let t2t = content.parse()?;
        Ok(TraderParameters { trader_id, broker_id, b2t, t2b, t2t })
    }
}