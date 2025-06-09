use super::*;
use quote::quote;

#[test]
fn test_no_parameters() {
    let params = quote! { () };
    let result = extract_param_fields(&params);

    println!("No params input: {}", params);
    println!("No params output: {}", result);

    // Should be empty - no trailing commas or spaces
    assert_eq!(result.to_string().trim(), "");
}

#[test]
fn test_single_parameter() {
    let params = quote! { (x: i32) };
    let result = extract_param_fields(&params);

    println!("Single param input: {}", params);
    println!("Single param output: {}", result);

    let result_str = result.to_string();
    assert!(
        result_str.contains(", x = x"),
        "Should contain ', x = x' but got: {}",
        result_str
    );
}

#[test]
fn test_multiple_parameters() {
    let params = quote! { (name: &str, count: usize) };
    let result = extract_param_fields(&params);

    println!("Multiple params input: {}", params);
    println!("Multiple params output: {}", result);

    let result_str = result.to_string();
    assert!(
        result_str.contains(", name = name"),
        "Should contain ', name = name'"
    );
    assert!(
        result_str.contains(", count = count"),
        "Should contain ', count = count'"
    );
}

#[test]
fn test_mut_parameter() {
    let params = quote! { (mut data: Vec<u8>) };
    let result = extract_param_fields(&params);

    println!("Mut param input: {}", params);
    println!("Mut param output: {}", result);

    let result_str = result.to_string();
    assert!(
        result_str.contains(", data = data"),
        "Should extract parameter name 'data' despite 'mut' keyword"
    );
}

#[test]
fn test_self_parameter_skipped() {
    let params = quote! { (&self, value: i32) };
    let result = extract_param_fields(&params);

    println!("Self param input: {}", params);
    println!("Self param output: {}", result);

    let result_str = result.to_string();
    // Should contain value but not self
    assert!(
        result_str.contains(", value = value"),
        "Should contain 'value' parameter"
    );
    assert!(
        !result_str.contains("self"),
        "Should NOT include 'self' in tracing fields"
    );
}

#[test]
fn test_mut_self_parameter_skipped() {
    let params = quote! { (&mut self, new_value: String) };
    let result = extract_param_fields(&params);

    println!("Mut self param input: {}", params);
    println!("Mut self param output: {}", result);

    let result_str = result.to_string();
    // Should contain new_value but not self
    assert!(
        result_str.contains(", new_value = new_value"),
        "Should contain 'new_value' parameter"
    );
    assert!(
        !result_str.contains("self"),
        "Should NOT include '&mut self' in tracing fields"
    );
}

#[test]
fn test_complex_types() {
    let params = quote! { (callback: fn(i32) -> String, data: Option<Vec<T>>) };
    let result = extract_param_fields(&params);

    println!("Complex types input: {}", params);
    println!("Complex types output: {}", result);

    let result_str = result.to_string();
    assert!(
        result_str.contains(", callback = callback"),
        "Should handle function pointer types"
    );
    assert!(
        result_str.contains(", data = data"),
        "Should handle complex generic types"
    );
}

#[test]
fn test_generic_parameter() {
    let params = quote! { (value: T, other: Option<U>) };
    let result = extract_param_fields(&params);

    println!("Generic param input: {}", params);
    println!("Generic param output: {}", result);

    let result_str = result.to_string();
    assert!(
        result_str.contains(", value = value"),
        "Should handle generic type T"
    );
    assert!(
        result_str.contains(", other = other"),
        "Should handle generic type Option<U>"
    );
}

#[test]
fn test_mixed_parameters() {
    let params = quote! { (&self, mut count: usize, name: &str, callback: impl Fn()) };
    let result = extract_param_fields(&params);

    println!("Mixed params input: {}", params);
    println!("Mixed params output: {}", result);

    let result_str = result.to_string();
    // Should have count, name, callback but not self
    assert!(
        result_str.contains(", count = count"),
        "Should extract 'count' despite 'mut'"
    );
    assert!(
        result_str.contains(", name = name"),
        "Should extract 'name' reference parameter"
    );
    assert!(
        result_str.contains(", callback = callback"),
        "Should extract 'callback' impl parameter"
    );
    assert!(!result_str.contains("self"), "Should skip 'self' parameter");
}

#[test]
fn test_reference_parameters() {
    let params = quote! { (data: &[u8], text: &mut String) };
    let result = extract_param_fields(&params);

    println!("Reference params input: {}", params);
    println!("Reference params output: {}", result);

    let result_str = result.to_string();
    assert!(
        result_str.contains(", data = data"),
        "Should handle &[u8] slice reference"
    );
    assert!(
        result_str.contains(", text = text"),
        "Should handle &mut String reference"
    );
}

#[test]
fn test_pattern_parameter_skipped() {
    // Pattern parameters should be skipped (for now)
    let params = quote! { ((x, y): (i32, i32)) };
    let result = extract_param_fields(&params);

    println!("Pattern param input: {}", params);
    println!("Pattern param output: {}", result);

    // Pattern parameters should be skipped, so result should be empty
    let result_str = result.to_string();
    assert_eq!(
        result_str.trim(),
        "",
        "Pattern parameters should be skipped"
    );
}

#[test]
#[ignore]
fn test_tuple_destructuring_parameter() {
    // Test that tuple destructuring works (when implemented)
    let params = quote! { ((a, b): (i32, i32), c: String) };
    let result = extract_param_fields(&params);

    println!("Tuple destructure input: {}", params);
    println!("Tuple destructure output: {}", result);

    let result_str = result.to_string();
    // Should contain c but skip the pattern parameter (a, b)
    assert!(result_str.contains(", c = c"));
    // Should NOT contain individual pattern components for now
    assert!(!result_str.contains(", a = a"));
    assert!(!result_str.contains(", b = b"));
}

#[test]
fn test_parsing_pipeline_debug() {
    let params = quote! { (name: &str, count: usize) };

    println!("=== DEBUGGING EXTRACT_PARAM_FIELDS PIPELINE ===");
    println!("Input: {}", params);

    // Let's manually trace what happens in extract_param_fields
    let mut param_iter = params.clone().into_token_iter();

    match param_iter.parse::<ParenthesisGroupContaining<Option<CommaDelimitedVec<FnParam>>>>() {
        Ok(parsed_params) => {
            println!("✅ Parsed ParenthesisGroupContaining successfully");

            if let Some(param_list) = &parsed_params.content {
                println!("✅ Found parameter list with {} params", param_list.0.len());

                for (i, param) in param_list.0.iter().enumerate() {
                    match &param.value {
                        FnParam::Named(named_param) => {
                            println!(
                                "  Param {}: Named '{}' (mut: {})",
                                i,
                                named_param.name,
                                named_param.mut_kw.is_some()
                            );
                        }
                        FnParam::SelfParam(_self_param) => {
                            println!("  Param {}: Self variant", i);
                        }
                        FnParam::Pattern(_) => {
                            println!("  Param {}: Pattern", i);
                        }
                    }
                }
            } else {
                println!("❌ No parameter list found");
            }
        }
        Err(e) => {
            println!("❌ Parse failed: {}", e);
        }
    }

    let result = extract_param_fields(&params);
    println!("Final result: {}", result);
}
