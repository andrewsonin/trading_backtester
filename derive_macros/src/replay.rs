use syn::{Error, parse::{Parse, ParseStream}, Path, Token};

pub(crate) struct ReplayParameters {
    pub exchange_id: Path,
    pub e2r: Path,
    pub r2r: Path,
    pub r2e: Path,
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