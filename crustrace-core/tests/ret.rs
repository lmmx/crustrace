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

// TODO 1-3: Parser Changes - Basic ret support
#[test]
fn test_bare_ret_parsing() {
    let args = quote!(ret);
    let item = quote! {
        fn test_function(x: u32) -> u32 {
            x + 1
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_ret_with_empty_parens() {
    let args = quote!(ret());
    let item = quote! {
        fn test_function() -> String {
            "hello".to_string()
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

// TODO 3,7,8: Format Mode Support
#[test]
fn test_ret_debug_format() {
    let args = quote!(ret(Debug));
    let item = quote! {
        fn test_function() -> Vec<i32> {
            vec![1, 2, 3]
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_ret_display_format() {
    let args = quote!(ret(Display));
    let item = quote! {
        fn test_function() -> String {
            "hello world".to_string()
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

// TODO 5,9,10: Level Handling
#[test]
fn test_ret_custom_level() {
    let args = quote!(ret(level = "warn"));
    let item = quote! {
        fn test_function() -> i32 {
            42
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_ret_inherits_function_level() {
    let args = quote!(level = "debug", ret);
    let item = quote! {
        fn test_function() -> i32 {
            42
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_ret_overrides_function_level() {
    let args = quote!(level = "info", ret(level = "error"));
    let item = quote! {
        fn test_function() -> Result<i32, String> {
            Ok(42)
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

// TODO 6: Complex Code Generation
#[test]
fn test_ret_with_level_and_format() {
    let args = quote!(ret(level = "warn", Display));
    let item = quote! {
        fn test_function() -> String {
            "test output".to_string()
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_ret_with_format_and_level() {
    let args = quote!(ret(Debug, level = "trace"));
    let item = quote! {
        fn test_function() -> Vec<String> {
            vec!["a".to_string(), "b".to_string()]
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

// TODO 4: Mixed with other arguments
#[test]
fn test_ret_with_name_and_level() {
    let args = quote!(name = "custom_span", level = "debug", ret);
    let item = quote! {
        fn test_function(x: i32) -> i32 {
            x * 2
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_ret_with_all_options() {
    let args = quote!(
        name = "complex_function",
        level = "info",
        ret(level = "warn", Display)
    );
    let item = quote! {
        fn complex_function(input: &str) -> Result<String, Box<dyn std::error::Error>> {
            Ok(input.to_uppercase())
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

// Edge cases and function types
#[test]
fn test_ret_with_async_function() {
    let args = quote!(ret);
    let item = quote! {
        async fn async_function() -> Result<String, std::io::Error> {
            Ok("async result".to_string())
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_ret_with_generic_function() {
    let args = quote!(ret(Debug));
    let item = quote! {
        fn generic_function<T: Clone + std::fmt::Debug>(value: T) -> T {
            value.clone()
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_ret_with_complex_return_type() {
    let args = quote!(ret(level = "debug"));
    let item = quote! {
        fn complex_return() -> impl Iterator<Item = Result<String, std::io::Error>> {
            std::iter::once(Ok("test".to_string()))
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_ret_with_unit_return() {
    let args = quote!(ret);
    let item = quote! {
        fn unit_function() {
            println!("no return value");
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_ret_with_never_return() {
    let args = quote!(ret);
    let item = quote! {
        fn never_returns() -> ! {
            panic!("never returns");
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

// TODO 11: Error Handling (these should fail when validation is implemented)
#[test]
#[should_panic(expected = "duplicate")]
fn test_duplicate_ret_error() {
    let args = quote!(ret, ret);
    let item = quote! {
        fn test_function() -> i32 { 42 }
    };

    apply_instrument(args, item);
}

#[test]
#[should_panic(expected = "unknown")]
fn test_invalid_ret_format() {
    let args = quote!(ret(Invalid));
    let item = quote! {
        fn test_function() -> i32 { 42 }
    };

    apply_instrument(args, item);
}

// Default behavior verification
#[test]
fn test_ret_default_is_debug() {
    let args = quote!(ret);
    let item = quote! {
        fn test_function() -> Vec<i32> {
            vec![1, 2, 3]
        }
    };

    let output = apply_instrument(args, item);
    // Should use Debug format (?) by default
    assert!(output.contains("return_value = ? __tracing_attr_ret"));
}

#[test]
fn test_ret_default_inherits_level() {
    let args = quote!(level = "error", ret);
    let item = quote! {
        fn test_function() -> i32 { 42 }
    };

    let output = apply_instrument(args, item);
    // Both span and ret event should use error level
    assert!(output.contains("tracing :: Level :: ERROR"));
}

// Performance edge cases
#[test]
fn test_ret_with_large_return_value() {
    let args = quote!(ret(Debug));
    let item = quote! {
        fn large_return() -> Vec<Vec<Vec<i32>>> {
            vec![vec![vec![1, 2, 3]; 100]; 100]
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}

#[test]
fn test_ret_with_self_referential_return() {
    let args = quote!(ret);
    let item = quote! {
        fn self_ref() -> Box<dyn Fn() -> String> {
            Box::new(|| "closure".to_string())
        }
    };

    assert_snapshot!(apply_instrument(args, item));
}
