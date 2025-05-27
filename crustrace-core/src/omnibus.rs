use proc_macro2::TokenStream;

use crate::token_processors::TokenProcessor;

pub fn trace_all_impl(input: TokenStream) -> TokenStream {
    TokenProcessor::new(input).process()
}
