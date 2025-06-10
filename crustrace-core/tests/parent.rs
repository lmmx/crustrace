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
fn test_parent_parsing() {
    let args = quote!(parent = "my_parent_span");
    let item = quote! {
        fn test_function(x: u32) -> u32 {
            x + 1
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_parent_string_literal() {
    let args = quote!(parent = "my_parent_span");
    let item = quote! {
        fn test_function(x: u32) -> u32 {
            x + 1
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_parent_variable() {
    let args = quote!(parent = my_span_variable);
    let item = quote! {
        fn test_function(x: u32) -> u32 {
            x + 1
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_parent_reference() {
    let args = quote!(parent = &parent_span);
    let item = quote! {
        fn test_function(x: u32) -> u32 {
            x + 1
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_parent_function_call() {
    let args = quote!(parent = get_current_span());
    let item = quote! {
        fn test_function(x: u32) -> u32 {
            x + 1
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_parent_method_call() {
    let args = quote!(parent = span_context.current_span());
    let item = quote! {
        fn test_function(x: u32) -> u32 {
            x + 1
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_parent_field_access() {
    let args = quote!(parent = self.parent_span);
    let item = quote! {
        fn test_function(&self, x: u32) -> u32 {
            x + 1
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_parent_with_other_args() {
    let args = quote!(
        level = "debug",
        name = "custom_name",
        parent = &context.span,
        target = "my::module"
    );
    let item = quote! {
        fn complex_function(a: i32, b: String) -> Result<i32, String> {
            Ok(a + b.len() as i32)
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_parent_none_expression() {
    let args = quote!(parent = Option::<tracing::Span>::None);
    let item = quote! {
        fn test_function(x: u32) -> u32 {
            x + 1
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}
