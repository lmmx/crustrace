use super::*;
use quote::quote;

#[test]
fn test_ret_dbg_parse_bare() {
    let args = quote!(ret);
    let mut iter = args.into_token_iter();

    let result = parse_instrument_args(&mut iter);
    println!("Bare ret parse result: {:?}", result);

    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert!(parsed.ret_args.is_some());
}

#[test]
fn test_ret_dbg_parse_display() {
    let args = quote!(ret(Display));
    let mut iter = args.into_token_iter();

    let result = parse_instrument_args(&mut iter);
    println!("ret(Display) parse result: {:?}", result);

    if let Ok(parsed) = result {
        if let Some(ret_args) = parsed.ret_args {
            println!("Format mode: {:?}", ret_args.format_mode());
            assert_eq!(ret_args.format_mode(), crate::parse::FormatMode::Display);
        }
    }
    // Don't assert success yet - this will fail until we implement parsing
}

#[test]
fn test_ret_dbg_codegen_display() {
    let args = quote!(ret(Display));
    let item = quote! { fn test() -> String { "hello".to_string() } };

    let result = instrument_impl(args, item);
    assert!(result.is_ok());

    let output = result.unwrap();
    let output_str = output.to_string();
    println!("Generated code: {}", output_str);

    // THIS SHOULD FAIL until Display format is implemented
    assert!(
        output_str.contains("return_value = % __tracing_attr_ret"),
        "Expected Display format (%) but got: {}",
        output_str
    );
}

#[test]
fn test_ret_dbg_codegen_custom_level() {
    let args = quote!(ret(level = "warn"));
    let item = quote! { fn test() -> i32 { 42 } };

    let result = instrument_impl(args, item);
    assert!(result.is_ok());

    let output = result.unwrap();
    let output_str = output.to_string();
    println!("Generated code: {}", output_str);

    // THIS SHOULD FAIL until custom level is implemented
    assert!(
        output_str.contains("tracing :: event ! (tracing :: Level :: WARN"),
        "Expected WARN level but got: {}",
        output_str
    );
}

#[test]
fn test_ret_dbg_duplicate_validation() {
    let args = quote!(ret, ret);
    let mut iter = args.into_token_iter();

    let result = parse_instrument_args(&mut iter);
    println!("Duplicate ret parse result: {:?}", result);

    // THIS SHOULD FAIL until duplicate validation is implemented
    assert!(
        result.is_err(),
        "Expected error for duplicate ret arguments but parsing succeeded"
    );
}
