use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

#[proc_macro_attribute]
pub fn trace_all(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input2: TokenStream2 = input.into();
    let output = crustrace_core::trace_all_impl(input2);
    output.into()
}