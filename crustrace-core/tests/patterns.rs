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

// Basic function patterns
#[test]
fn test_basic_function() {
    let input = quote! {
        fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_function() {
    let input = quote! {
        pub fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

// Async function patterns
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

// Unsafe function patterns
#[test]
fn test_unsafe_function() {
    let input = quote! {
        unsafe fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_unsafe_function() {
    let input = quote! {
        pub unsafe fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

// Const function patterns
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

// Extern function patterns
#[test]
fn test_extern_function() {
    let input = quote! {
        extern fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_extern_c_function() {
    let input = quote! {
        extern "C" fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_extern_c_function() {
    let input = quote! {
        pub extern "C" fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_extern_system_function() {
    let input = quote! {
        extern "system" fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

// Combined modifier patterns
#[test]
fn test_async_unsafe_function() {
    let input = quote! {
        async unsafe fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_async_unsafe_function() {
    let input = quote! {
        pub async unsafe fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_const_unsafe_function() {
    let input = quote! {
        const unsafe fn hello() {
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

#[test]
fn test_unsafe_extern_c_function() {
    let input = quote! {
        unsafe extern "C" fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_unsafe_extern_c_function() {
    let input = quote! {
        pub unsafe extern "C" fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

// Advanced visibility patterns
#[test]
fn test_pub_crate_function() {
    let input = quote! {
        pub(crate) fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_super_function() {
    let input = quote! {
        pub(super) fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_self_function() {
    let input = quote! {
        pub(self) fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_in_path_function() {
    let input = quote! {
        pub(in crate::module) fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_crate_async_function() {
    let input = quote! {
        pub(crate) async fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_super_unsafe_function() {
    let input = quote! {
        pub(super) unsafe fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_in_path_const_function() {
    let input = quote! {
        pub(in crate::utils) const fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

// Complex combinations
#[test]
fn test_pub_crate_async_unsafe_function() {
    let input = quote! {
        pub(crate) async unsafe fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_super_const_unsafe_function() {
    let input = quote! {
        pub(super) const unsafe fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_in_path_unsafe_extern_c_function() {
    let input = quote! {
        pub(in crate::ffi) unsafe extern "C" fn hello() {
            println!("world");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

// Functions with parameters and return types
#[test]
fn test_async_function_with_params() {
    let input = quote! {
        async fn hello(name: &str, count: usize) -> Result<String, Error> {
            Ok(format!("Hello {} ({})", name, count))
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_pub_unsafe_function_with_generics() {
    let input = quote! {
        pub unsafe fn hello<T: Clone + Send>(value: T) -> T {
            value.clone()
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_const_function_with_where_clause() {
    let input = quote! {
        const fn hello<T>(value: T) -> T
        where
            T: Copy + Default,
        {
            value
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

// Functions in different contexts
#[test]
fn test_impl_block_methods() {
    let input = quote! {
        impl MyStruct {
            fn method(&self) {
                println!("method");
            }

            pub async fn async_method(&mut self) -> i32 {
                42
            }

            unsafe fn unsafe_method() {
                println!("unsafe");
            }

            pub(crate) const fn const_method() -> usize {
                100
            }
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_trait_methods() {
    let input = quote! {
        trait MyTrait {
            fn required_method(&self);

            async fn async_trait_method(&self) -> String {
                "default".to_string()
            }

            unsafe fn unsafe_trait_method();

            const fn const_trait_method() -> i32 {
                0
            }
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_nested_module_functions() {
    let input = quote! {
        mod outer {
            pub fn outer_function() {}

            mod inner {
                async fn inner_async_function() {}
                pub(super) unsafe fn inner_unsafe_function() {}
            }

            impl SomeStruct {
                const fn impl_const_function() -> i32 { 42 }
            }
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

// Edge cases and mixed content
#[test]
fn test_mixed_content_with_functions() {
    let input = quote! {
        use std::collections::HashMap;

        const SOME_CONST: i32 = 42;

        struct MyStruct {
            field: String,
        }

        async fn actual_function() {
            println!("This should be instrumented");
        }

        enum MyEnum {
            Variant1,
            Variant2(i32),
        }

        pub unsafe fn another_function() -> Result<(), Error> {
            Ok(())
        }

        type MyType = HashMap<String, i32>;
    };

    assert_snapshot!(apply_trace_all(input));
}
