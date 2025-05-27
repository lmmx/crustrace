use crustrace_core::trace_all_impl;
use insta::assert_snapshot;
use proc_macro2::TokenStream;
use quote::quote;
use rust_format::{Formatter, RustFmt};

fn apply_trace_all(input: TokenStream) -> String {
    let output = trace_all_impl(input);
    println!("Traced::::: {}", output);
    let fmt_str = RustFmt::default()
        .format_tokens(output)
        .unwrap_or_else(|e| panic!("Format error: {}", e));
    println!("Formatted:: {}", fmt_str);
    fmt_str
}

#[test]
fn test_async_function() {
    let input = quote! {
        async fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_async_function() {
    let input = quote! {
        pub async fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_const_function() {
    let input = quote! {
        const fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_const_function() {
    let input = quote! {
        pub const fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_const_unsafe_function() {
    let input = quote! {
        pub const unsafe fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}
