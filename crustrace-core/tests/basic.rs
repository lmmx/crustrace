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
fn test_single_function() {
    let input = quote! {
        fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_multiple_functions() {
    let input = quote! {
        fn foo(x: i32) -> i32 {
            bar(x + 1)
        }

        fn bar(y: i32) -> i32 {
            y * 2
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_generic_function() {
    let input = quote! {
        fn generic<T: Clone>(value: T) -> T {
            value.clone()
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_ignores_non_functions() {
    let input = quote! {
        const x: String = "fn not_a_function";
        struct Foo { field: i32 }
        fn actual_function() {}
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_module_with_functions() {
    let input = quote! {
        mod calculations {
            pub fn fibonacci(n: u64) -> u64 {
                if n <= 1 {
                    n
                } else {
                    add_numbers(fibonacci(n - 1), fibonacci(n - 2))
                }
            }

            fn add_numbers(a: u64, b: u64) -> u64 {
                a + b
            }
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_impl_block_methods() {
    let input = quote! {
        impl Calculator {
            pub fn new() -> Self {
                Self
            }

            pub fn add(&self, a: i32, b: i32) -> i32 {
                a + b
            }

            pub fn multiply(&self, x: i32, y: i32) -> i32 {
                x * y
            }

            fn internal_helper(&self, value: i32) -> i32 {
                value * 2
            }
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_impl_block_with_generics() {
    let input = quote! {
        impl<T> Container<T>
        where
            T: Clone + std::fmt::Debug,
        {
            pub fn new(value: T) -> Self {
                Self { inner: value }
            }

            pub fn get(&self) -> &T {
                &self.inner
            }

            pub fn set(&mut self, new_value: T) {
                self.inner = new_value;
            }
        }
    };

    assert_snapshot!(apply_trace_all(input));
}
