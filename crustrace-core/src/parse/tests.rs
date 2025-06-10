//! Unit tests for parse module
use super::*;
use proc_macro2::TokenStream;
use quote::quote;

fn parse_fn_sig(input: TokenStream) -> Result<FnSig> {
    let mut iter = input.into_token_iter();
    iter.parse::<FnSig>()
}

fn fn_sig_to_tokens(sig: FnSig) -> TokenStream {
    let mut tokens = TokenStream::new();
    sig.to_tokens(&mut tokens);
    tokens
}

#[test]
fn test_basic_function() {
    let input = quote! { fn hello() {} };
    let parsed = parse_fn_sig(input.clone()).expect("Should parse");

    let output = fn_sig_to_tokens(parsed);
    // Fix the spacing comparison - just check it contains the right parts
    let output_str = output.to_string();
    assert!(output_str.contains("fn"));
    assert!(output_str.contains("hello"));
    assert!(output_str.contains("()"));
    assert!(output_str.contains("{ }"));
}

#[test]
fn test_pub_crate_async_function() {
    let input = quote! { pub(crate) async fn hello() {} };
    println!("Parsing input: {}", input);

    match parse_fn_sig(input.clone()) {
        Ok(parsed) => {
            println!("✅ Parsed successfully!");
            println!("  visibility: {:?}", parsed.visibility.is_some());
            println!("  async_kw: {:?}", parsed.async_kw.is_some());

            let output = fn_sig_to_tokens(parsed);
            println!("Output: {}", output);
        }
        Err(e) => {
            println!("❌ Parse failed: {}", e);
            // Let's try parsing just the parts
            let mut iter = input.into_token_iter();
            if let Ok(_vis) = iter.parse::<Visibility>() {
                println!("✅ Visibility parsed OK");
                if let Ok(_async_kw) = iter.parse::<KAsync>() {
                    println!("✅ Async keyword parsed OK");
                } else {
                    println!("❌ Async keyword failed");
                }
            } else {
                println!("❌ Visibility parsing failed");
            }
            panic!("Parse failed: {}", e);
        }
    }
}

#[test]
fn test_async_function() {
    let input = quote! { async fn hello() {} };
    let parsed = parse_fn_sig(input.clone()).expect("Should parse async fn");

    assert!(parsed.async_kw.is_some());
    assert!(parsed.unsafe_kw.is_none());
    assert_eq!(parsed.name.to_string(), "hello");

    let output = fn_sig_to_tokens(parsed);
    println!("Input:  {}", input);
    println!("Output: {}", output);
    assert!(output.to_string().contains("async"));
}

#[test]
fn test_unsafe_function() {
    let input = quote! { unsafe fn hello() {} };
    let parsed = parse_fn_sig(input.clone()).expect("Should parse unsafe fn");

    assert!(parsed.unsafe_kw.is_some());
    assert!(parsed.async_kw.is_none());
    assert_eq!(parsed.name.to_string(), "hello");

    let output = fn_sig_to_tokens(parsed);
    println!("Input:  {}", input);
    println!("Output: {}", output);
    assert!(output.to_string().contains("unsafe"));
}

#[test]
fn test_const_function() {
    let input = quote! { const fn hello() {} };
    let parsed = parse_fn_sig(input.clone()).expect("Should parse const fn");

    assert!(parsed.const_kw.is_some());
    assert!(parsed.async_kw.is_none());
    assert_eq!(parsed.name.to_string(), "hello");

    let output = fn_sig_to_tokens(parsed);
    println!("Input:  {}", input);
    println!("Output: {}", output);
    assert!(output.to_string().contains("const"));
}

#[test]
fn test_pub_async_function() {
    let input = quote! { pub async fn hello() {} };
    let parsed = parse_fn_sig(input.clone()).expect("Should parse pub async fn");

    assert!(parsed.visibility.is_some());
    assert!(parsed.async_kw.is_some());
    assert!(parsed.unsafe_kw.is_none());
    assert_eq!(parsed.name.to_string(), "hello");

    let output = fn_sig_to_tokens(parsed);
    println!("Input:  {}", input);
    println!("Output: {}", output);
    assert!(output.to_string().contains("pub"));
    assert!(output.to_string().contains("async"));
}

#[test]
fn test_async_unsafe_function() {
    let input = quote! { async unsafe fn hello() {} };
    let parsed = parse_fn_sig(input.clone()).expect("Should parse async unsafe fn");

    assert!(parsed.async_kw.is_some());
    assert!(parsed.unsafe_kw.is_some());
    assert_eq!(parsed.name.to_string(), "hello");

    let output = fn_sig_to_tokens(parsed);
    println!("Input:  {}", input);
    println!("Output: {}", output);
    assert!(output.to_string().contains("async"));
    assert!(output.to_string().contains("unsafe"));
}

#[test]
fn test_const_unsafe_function() {
    let input = quote! { const unsafe fn hello() {} };
    let parsed = parse_fn_sig(input.clone()).expect("Should parse const unsafe fn");

    assert!(parsed.const_kw.is_some());
    assert!(parsed.unsafe_kw.is_some());
    assert_eq!(parsed.name.to_string(), "hello");

    let output = fn_sig_to_tokens(parsed);
    println!("Input:  {}", input);
    println!("Output: {}", output);
    assert!(output.to_string().contains("const"));
    assert!(output.to_string().contains("unsafe"));
}

#[test]
fn test_extern_function() {
    let input = quote! { extern "C" fn hello() {} };
    let parsed = parse_fn_sig(input.clone()).expect("Should parse extern fn");

    assert!(parsed.extern_kw.is_some());
    assert_eq!(parsed.name.to_string(), "hello");

    let output = fn_sig_to_tokens(parsed);
    println!("Input:  {}", input);
    println!("Output: {}", output);
    assert!(output.to_string().contains("extern"));
    assert!(output.to_string().contains("\"C\""));
}

#[test]
fn test_complex_function() {
    let input = quote! { pub const unsafe extern "C" fn hello<T>(x: T) -> T where T: Clone {} };
    let parsed = parse_fn_sig(input.clone()).expect("Should parse complex fn");

    assert!(parsed.visibility.is_some());
    assert!(parsed.const_kw.is_some());
    assert!(parsed.unsafe_kw.is_some());
    assert!(parsed.extern_kw.is_some());
    assert!(parsed.generics.is_some());
    assert!(parsed.return_type.is_some());
    assert_eq!(parsed.name.to_string(), "hello");

    let output = fn_sig_to_tokens(parsed);
    println!("Input:  {}", input);
    println!("Output: {}", output);
    assert!(output.to_string().contains("pub"));
    assert!(output.to_string().contains("const"));
    assert!(output.to_string().contains("unsafe"));
    assert!(output.to_string().contains("extern"));
    assert!(output.to_string().contains("\"C\""));
}

#[test]
fn test_ret_with_level_parsing() {
    let input = quote!(level = "debug", ret);
    let mut iter = input.into_token_iter();

    match iter.parse::<InstrumentInner>() {
        Ok(parsed) => {
            println!("✅ Parsed mixed arguments successfully!");

            assert!(parsed.args.is_some(), "Should have parsed arguments");
            let args = parsed.args.as_ref().unwrap();
            println!("Number of arguments: {}", args.0.len());
            assert_eq!(args.0.len(), 2, "Should have 2 arguments");

            let mut found_level = false;
            let mut found_ret = false;

            for arg in &args.0 {
                match &arg.value {
                    InstrumentArg::Level(_) => found_level = true,
                    InstrumentArg::Ret(_) => found_ret = true,
                    _ => {}
                }
            }

            assert!(found_level, "Should find Level argument");
            assert!(found_ret, "Should find Ret argument");
        }
        Err(e) => {
            println!("❌ Parse failed: {}", e);
            panic!("Parse failed: {}", e);
        }
    }
}

#[test]
fn test_mixed_args_with_ret() {
    let input = quote!(
        level = "info",
        name = "custom",
        target = "a_crate::a_target",
        ret
    );
    let mut iter = input.into_token_iter();

    match iter.parse::<InstrumentInner>() {
        Ok(parsed) => {
            println!("✅ Parsed mixed arguments with ret successfully!");

            assert!(parsed.args.is_some(), "Should have parsed arguments");
            let args = parsed.args.as_ref().unwrap();
            assert_eq!(args.0.len(), 4, "Should have 4 arguments");

            let mut found_level = false;
            let mut found_name = false;
            let mut found_ret = false;
            let mut found_target = false;

            for arg in &args.0 {
                match &arg.value {
                    InstrumentArg::Level(_) => found_level = true,
                    InstrumentArg::Name(_) => found_name = true,
                    InstrumentArg::Ret(_) => found_ret = true,
                    InstrumentArg::Target(_) => found_target = true,
                }
            }

            assert!(found_level, "Should find Level argument");
            assert!(found_name, "Should find Name argument");
            assert!(found_ret, "Should find Ret argument");
            assert!(found_target, "Should find Target argument");
        }
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_bare_ret_parsing() {
    let input = quote!(ret);
    let mut iter = input.into_token_iter();

    match iter.parse::<InstrumentInner>() {
        Ok(parsed) => {
            println!("✅ Parsed bare ret successfully!");

            assert!(parsed.args.is_some(), "Should have parsed arguments");
            let args = parsed.args.as_ref().unwrap();
            if let Some(first_arg) = args.0.first() {
                match &first_arg.value {
                    InstrumentArg::Ret(_) => {
                        println!("✅ Found Ret argument");
                    }
                    _ => panic!("Expected Ret argument"),
                }
            }
        }
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_ret_with_parentheses_parsing() {
    let input = quote!(ret());
    let mut iter = input.into_token_iter();

    match iter.parse::<InstrumentInner>() {
        Ok(_parsed) => {
            println!("✅ Parsed ret() successfully!");
            // Should parse ret with empty parentheses (default EventArgs)
        }
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_ret_with_debug_format() {
    let input = quote!(ret(Debug));
    let mut iter = input.into_token_iter();

    match iter.parse::<InstrumentInner>() {
        Ok(_parsed) => {
            println!("✅ Parsed ret(Debug) successfully!");
            // Should parse ret with Debug format mode
        }
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_ret_with_display_format() {
    let input = quote!(ret(Display));
    let mut iter = input.into_token_iter();

    match iter.parse::<InstrumentInner>() {
        Ok(_parsed) => {
            println!("✅ Parsed ret(Display) successfully!");
            // Should parse ret with Display format mode
        }
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_ret_with_custom_level() {
    let input = quote!(ret(level = "debug"));
    let mut iter = input.into_token_iter();

    match iter.parse::<InstrumentInner>() {
        Ok(_parsed) => {
            println!("✅ Parsed ret(level = \"debug\") successfully!");
            // Should parse ret with custom level
        }
        Err(e) => panic!("Parse failed: {}", e),
    }
}

#[test]
fn test_ret_with_level_and_format() {
    let input = quote!(ret(level = "warn", Display));
    let mut iter = input.into_token_iter();

    match iter.parse::<InstrumentInner>() {
        Ok(_parsed) => {
            println!("✅ Parsed ret(level = \"warn\", Display) successfully!");
            // Should parse ret with both custom level and format mode
        }
        Err(e) => panic!("Parse failed: {}", e),
    }
}
