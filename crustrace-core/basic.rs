use insta::assert_snapshot;
use quote::quote;
use proc_macro2::TokenStream;
use crustrace_core::trace_all_impl;

fn apply_trace_all(input: TokenStream) -> String {
    let output = trace_all_impl(input);
    let syntax_tree = syn::parse2(output).expect("Failed to parse generated code");
    prettyplease::unparse(&syntax_tree)
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
        let x = "fn not_a_function";
        struct Foo { field: i32 }
        fn actual_function() {}
    };
    
    assert_snapshot!(apply_trace_all(input));
}
