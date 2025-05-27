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
fn test_ignores_function_calls_in_expressions() {
    let input = quote! {
        fn outer_function() {
            let result = some_fn_call();
            another_fn_call(42, "hello");
            nested::module::fn_call();
            obj.method_fn_call();
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_ignores_fn_in_string_literals() {
    let input = quote! {
        fn real_function() {
            let msg = "This fn is not a function";
            let code = r#"fn fake_function() { return "not real"; }"#;
            println!("fn appears in this string too");
        }

        const TEMPLATE: &str = "fn template_function() {}";
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_ignores_fn_in_comments() {
    let input = quote! {
        // fn this_is_commented_out() {}
        /* fn this_is_also_commented() {} */

        fn actual_function() {
            // fn another_comment_function() {}
            /*
             * fn multiline_comment_function() {
             *     // More comments with fn
             * }
             */
            println!("Hello");
        }

        /// Documentation comment with fn example() {}
        fn documented_function() {}
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
#[ignore]
fn test_ignores_already_instrumented_functions() {
    let input = quote! {
        #[crustrace::instrument]
        fn already_instrumented() {
            println!("This function already has instrumentation");
        }

        #[tracing::instrument]
        fn also_already_instrumented() {
            println!("This one too");
        }

        #[instrument(level = "debug")]
        fn custom_instrumented() {
            println!("Custom instrumentation");
        }

        fn needs_instrumentation() {
            println!("This one should get instrumented");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_complex_edge_cases() {
    let input = quote! {
        // This should get instrumented
        fn legitimate_function() {
            // fn this_is_just_a_comment
            let variable = "fn not_a_function";
            some_function_call();

            if condition {
                another_fn_call();
            }

            match value {
                Pattern => yet_another_fn_call(),
                _ => final_fn_call(),
            }
        }

        struct MyStruct {
            field: String, // fn not a function in a comment
        }

        const CODE_SAMPLE: &str = r#"
            fn example() {
                println!("This fn is in a string");
            }
        "#;
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
#[ignore]
fn test_fn_keyword_in_various_contexts() {
    let input = quote! {
        fn real_function() {
            println!("Real function");
        }

        type FnPointer = fn() -> i32;

        fn function_with_fn_param(callback: fn(i32) -> String) -> String {
            callback(42)
        }

        fn returns_fn() -> fn() -> i32 {
            || 42
        }

        trait MyTrait {
            fn trait_method(&self);
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_basic_function_gets_instrumented() {
    let input = quote! {
        fn real_function() {
            println!("Real function");
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_ignores_type_alias_with_fn() {
    let input = quote! {
        type FnPointer = fn() -> i32;
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_function_with_fn_type_parameter() {
    let input = quote! {
        fn function_with_fn_param(callback: fn(i32) -> String) -> String {
            callback(42)
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_function_returning_fn_type() {
    let input = quote! {
        fn returns_fn() -> fn() -> i32 {
            || 42
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_trait_method_declarations() {
    let input = quote! {
        trait MyTrait {
            fn trait_method(&self);

            fn default_method(&self) {
                println!("This has a body and should be instrumented");
            }
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_trait_with_default_method() {
    let input = quote! {
        trait MyTrait {
            fn default_method(&self) {
                println!("This has a body and should be instrumented");
            }
        }

        struct MyStruct;

        impl MyTrait for MyStruct {}

        fn main() {
            let my_struct = MyStruct;
            my_struct.default_method();
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_trait_with_trait_method_and_default_method() {
    let input = quote! {
        trait MyTrait {
            fn trait_method(&self);

            fn default_method(&self) {
                println!("This has a body and should be instrumented");
            }
        }

        struct MyStruct;

        impl MyTrait for MyStruct {
            fn trait_method(&self) {
                println!("This has an impl method and should be instrumented");
            }
        }

        fn main() {
            let my_struct = MyStruct;
            my_struct.default_method();
            my_struct.trait_method();
        }
    };

    assert_snapshot!(apply_trace_all(input));
}

#[test]
fn test_trait_with_trait_method_and_default_method_rev() {
    let input = quote! {
        trait MyTrait {
            fn default_method(&self) {
                println!("This has a body and should be instrumented");
            }

            fn trait_method(&self);
        }

        struct MyStruct;

        impl MyTrait for MyStruct {
            fn trait_method(&self) {
                println!("This has an impl method and should be instrumented");
            }
        }

        fn main() {
            let my_struct = MyStruct;
            my_struct.default_method();
            my_struct.trait_method();
        }
    };

    assert_snapshot!(apply_trace_all(input));
}
