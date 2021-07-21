use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Token};

#[derive(Debug)]
pub struct GuardAssign {
    pub guard_name: Option<String>,
}

impl Parse for GuardAssign {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if input.is_empty() {
            return Ok(Self { guard_name: None });
        } else if lookahead.peek(Ident) {
            let ident = input.parse::<Ident>()?;
            if ident != "guard" {
                panic!("Only \"guard\" argument allowed (e.g. 'guard = \"function_name\"')");
            }

            let lookahead = input.lookahead1();

            if lookahead.peek(Token![=]) {
                input.parse::<Token![=]>()?;

                let lookahead = input.lookahead1();

                if lookahead.peek(LitStr) {
                    let guard_name = input.parse::<LitStr>()?.value();

                    return Ok(Self {
                        guard_name: Some(guard_name),
                    });
                }
            }
        }

        Err(lookahead.error())
    }
}
