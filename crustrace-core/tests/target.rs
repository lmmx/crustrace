use crustrace_core::instrument_impl;
use insta::assert_snapshot;
use proc_macro2::TokenStream;
use quote::quote;
use rust_format::{Formatter, RustFmt};

fn apply_instrument(args: TokenStream, input: TokenStream) -> String {
    let output = instrument_impl(args, input).expect("Should instrument successfully");
    println!("Instrumented: {}", output);
    let fmt_str = RustFmt::default()
        .format_tokens(output)
        .unwrap_or_else(|e| panic!("Format error: {}", e));
    println!("Formatted: {}", fmt_str);
    fmt_str
}

#[test]
fn test_target_parsing() {
    let args = quote!(target = "my_crate::my_target");
    let item = quote! {
        fn test_function(x: u32) -> u32 {
            x + 1
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}
