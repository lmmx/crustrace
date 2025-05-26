use proc_macro2::{TokenStream, TokenTree};
use quote::quote;

pub fn trace_all_impl(input: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();
    let mut tokens = input.into_iter().peekable();

    while let Some(token) = tokens.next() {
        match &token {
            TokenTree::Ident(ident) if ident.to_string() == "fn" => {
                if let Some(TokenTree::Ident(_)) = tokens.peek() {
                    let instrument = quote! {
                        #[tracing::instrument(level = "info", ret)]
                    };
                    output.extend(instrument);
                }
                output.extend(std::iter::once(token));
            }
            _ => {
                output.extend(std::iter::once(token));
            }
        }
    }

    output
}