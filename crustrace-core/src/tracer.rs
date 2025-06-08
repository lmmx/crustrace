//! Procedural macro attributes for automatically instrumenting functions with `tracing`.
//!
//! This crate provides the [`#[instrument]`] attribute macro using `unsynn` for parsing,
//! offering a lightweight alternative to the standard `tracing-attributes` crate.

use crate::parse::FnParam;
use core::result::Result;
use proc_macro2::TokenStream;
use quote::quote;
use unsynn::*;

use crate::parse::{FnSig, InstrumentArg, InstrumentInner};

pub fn instrument_impl(args: TokenStream, item: TokenStream) -> Result<TokenStream, TokenStream> {
    // Parse the instrument arguments
    let mut args_iter = args.to_token_iter();
    let instrument_args = if args.is_empty() {
        InstrumentArgs::default()
    } else {
        match parse_instrument_args(&mut args_iter) {
            Ok(args) => args,
            Err(e) => return Err(quote! { compile_error!(#e) }),
        }
    };

    // Parse the function
    let mut item_iter = item.to_token_iter();
    let func = match parse_simple_function(&mut item_iter) {
        Ok(func) => func,
        Err(e) => return Err(quote! { compile_error!(#e) }),
    };

    Ok(generate_instrumented_function(instrument_args, func))
}

#[derive(Default)]
struct InstrumentArgs {
    level: Option<String>,
    name: Option<String>,
    ret: bool,
}

struct SimpleFunction {
    attrs: Vec<TokenStream>,
    vis: Option<TokenStream>,
    const_kw: Option<TokenStream>,
    async_kw: Option<TokenStream>,
    unsafe_kw: Option<TokenStream>,
    extern_kw: Option<TokenStream>,
    fn_name: Ident,
    generics: Option<TokenStream>,
    params: TokenStream,
    ret_type: Option<TokenStream>,
    where_clause: Option<TokenStream>,
    body: TokenStream,
}

fn parse_instrument_args(input: &mut TokenIter) -> Result<InstrumentArgs, String> {
    match input.parse::<InstrumentInner>() {
        Ok(parsed) => {
            let mut args = InstrumentArgs::default();

            if let Some(arg_list) = parsed.args {
                for arg in arg_list.0 {
                    match arg.value {
                        InstrumentArg::Level(level_arg) => {
                            args.level = Some(level_arg.value.as_str().to_string());
                        }
                        InstrumentArg::Name(name_arg) => {
                            args.name = Some(name_arg.value.as_str().to_string());
                        }
                        InstrumentArg::Ret(_) => {
                            args.ret = true;
                        }
                    }
                }
            }

            Ok(args)
        }
        Err(e) => Err(format!("Failed to parse instrument args: {}", e)),
    }
}

fn parse_simple_function(input: &mut TokenIter) -> Result<SimpleFunction, String> {
    match input.parse::<FnSig>() {
        Ok(parsed) => {
            // Handle attributes
            let attrs = if let Some(attr_list) = parsed.attributes {
                attr_list
                    .0
                    .into_iter()
                    .map(|attr| {
                        let mut tokens = TokenStream::new();
                        unsynn::ToTokens::to_tokens(&attr, &mut tokens);
                        tokens
                    })
                    .collect()
            } else {
                Vec::new()
            };

            // Handle visibility
            let vis = parsed.visibility.map(|v| {
                let mut tokens = TokenStream::new();
                quote::ToTokens::to_tokens(&v, &mut tokens);
                tokens
            });

            // Handle const keyword
            let const_kw = parsed.const_kw.map(|k| {
                let mut tokens = TokenStream::new();
                unsynn::ToTokens::to_tokens(&k, &mut tokens);
                tokens
            });

            // Handle async keyword
            let async_kw = parsed.async_kw.map(|k| {
                let mut tokens = TokenStream::new();
                unsynn::ToTokens::to_tokens(&k, &mut tokens);
                tokens
            });

            // Handle unsafe keyword
            let unsafe_kw = parsed.unsafe_kw.map(|k| {
                let mut tokens = TokenStream::new();
                unsynn::ToTokens::to_tokens(&k, &mut tokens);
                tokens
            });

            // Handle extern keyword
            let extern_kw = parsed.extern_kw.map(|k| {
                let mut tokens = TokenStream::new();
                unsynn::ToTokens::to_tokens(&k, &mut tokens);
                tokens
            });

            let fn_name = parsed.name;

            let generics = parsed.generics.map(|g| {
                let mut tokens = TokenStream::new();
                unsynn::ToTokens::to_tokens(&g, &mut tokens);
                tokens
            });

            let mut params = TokenStream::new();
            unsynn::ToTokens::to_tokens(&parsed.params, &mut params);

            let ret_type = parsed.return_type.map(|rt| {
                let mut tokens = TokenStream::new();
                unsynn::ToTokens::to_tokens(&rt, &mut tokens);
                tokens
            });

            let where_clause = parsed.where_clause.map(|wc| {
                let mut tokens = TokenStream::new();
                unsynn::ToTokens::to_tokens(&wc, &mut tokens);
                tokens
            });

            let mut body = TokenStream::new();
            unsynn::ToTokens::to_tokens(&parsed.body, &mut body);

            Ok(SimpleFunction {
                attrs,
                vis,
                const_kw,
                async_kw,
                unsafe_kw,
                extern_kw,
                fn_name,
                generics,
                params,
                ret_type,
                where_clause,
                body,
            })
        }
        Err(e) => Err(format!("Failed to parse function: {}", e)),
    }
}

fn generate_instrumented_function(args: InstrumentArgs, func: SimpleFunction) -> TokenStream {
    let SimpleFunction {
        attrs,
        vis,
        const_kw,
        async_kw,
        unsafe_kw,
        extern_kw,
        fn_name,
        generics,
        params,
        ret_type,
        where_clause,
        body,
    } = func;

    // Determine span name
    let span_name = args.name.unwrap_or_else(|| fn_name.to_string());

    // Determine level
    let level = match args.level.as_deref() {
        Some("trace") => quote!(tracing::Level::TRACE),
        Some("debug") => quote!(tracing::Level::DEBUG),
        Some("info") => quote!(tracing::Level::INFO),
        Some("warn") => quote!(tracing::Level::WARN),
        Some("error") => quote!(tracing::Level::ERROR),
        _ => quote!(tracing::Level::INFO),
    };

    // Extract parameter fields (simplified)
    let param_fields = extract_param_fields(&params);

    // Generate visibility tokens
    let vis_tokens = vis.unwrap_or_default();

    // Generate modifier tokens
    let const_tokens = const_kw.unwrap_or_default();
    let async_tokens = async_kw.unwrap_or_default();
    let unsafe_tokens = unsafe_kw.unwrap_or_default();
    let extern_tokens = extern_kw.unwrap_or_default();

    // Generate generics tokens
    let generics_tokens = generics.unwrap_or_default();

    // Generate return type tokens
    let ret_tokens = ret_type.unwrap_or_default();

    // Generate where clause tokens
    let where_tokens = where_clause.unwrap_or_default();

    // Generate the body handling based on whether ret is enabled
    let body_handling = if args.ret {
        quote! {
            let __tracing_attr_ret = (|| #body)();
            tracing::event!(#level, return_value = ?__tracing_attr_ret);
            __tracing_attr_ret
        }
    } else {
        body
    };

    // Generate the instrumented function
    quote! {
        #(#attrs)*
        #vis_tokens #const_tokens #async_tokens #unsafe_tokens #extern_tokens fn #fn_name #generics_tokens #params #ret_tokens #where_tokens {
            let __tracing_attr_span = tracing::span!(
                #level,
                #span_name
                #param_fields
            );
            let __tracing_attr_guard = __tracing_attr_span.enter();

            #body_handling
        }
    }
}

/// Extract parameter names from function parameters for tracing fields
fn extract_param_fields(params: &TokenStream) -> TokenStream {
    // Parse the params TokenStream to extract the parameter list
    let mut param_iter = params.clone().into_token_iter();

    // Try to parse as ParenthesisGroupContaining<Option<CommaDelimitedVec<FnParam>>>
    let parsed_params = match param_iter
        .parse::<ParenthesisGroupContaining<Option<CommaDelimitedVec<FnParam>>>>()
    {
        Ok(params) => params,
        Err(_) => return quote!(), // Parse failed, return empty
    };

    let Some(param_list) = &parsed_params.content else {
        return quote!(); // No parameters
    };

    let mut fields = Vec::new();

    for param in &param_list.0 {
        match &param.value {
            FnParam::Named(named_param) => {
                let param_name = &named_param.name;
                fields.push(quote!(, #param_name = #param_name));
            }
            FnParam::SelfParam(_) => {
                // Skip self parameters for tracing
            }
            FnParam::Pattern(pattern_param) => {
                let identifiers = pattern_param.pattern.extract_identifiers();
                for ident in identifiers {
                    fields.push(quote!(, #ident = #ident));
                }
            }
        }
    }

    quote!(#(#fields)*)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_format::{Formatter, RustFmt};

    // Helper function to format and print tokens
    fn format_and_print(tokens: proc_macro2::TokenStream) -> String {
        let fmt_str = RustFmt::default()
            .format_tokens(tokens)
            .unwrap_or_else(|e| panic!("Format error: {}", e));
        println!("Generated code: {}", fmt_str);
        fmt_str
    }

    #[test]
    fn test_basic_instrumentation() {
        let args = quote!();
        let item = quote! {
            fn test_function(x: u32) -> u32 {
                x + 1
            }
        };

        let result = instrument_impl(args, item);
        if let Err(ref e) = result {
            eprintln!("Error: {}", e);
        }
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = format_and_print(output);

        // Check that instrumentation was added
        assert!(output_str.contains("tracing::span!"));
        assert!(output_str.contains("test_function"));
        assert!(output_str.contains("tracing::Level::INFO"));
    }

    #[test]
    fn test_custom_level() {
        let args = quote!(level = "debug");
        let item = quote! {
            fn test_function() {}
        };

        let result = instrument_impl(args, item);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = format_and_print(output);

        assert!(output_str.contains("tracing::Level::DEBUG"));
    }

    #[test]
    fn test_custom_name() {
        let args = quote!(name = "custom_span");
        let item = quote! {
            fn test_function() {}
        };

        let result = instrument_impl(args, item);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = format_and_print(output);

        assert!(output_str.contains("\"custom_span\""));
    }

    #[test]
    fn test_async_function() {
        let args = quote!();
        let item = quote! {
            async fn test_async() {
                println!("async test");
            }
        };

        let result = instrument_impl(args, item);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = format_and_print(output);

        assert!(output_str.contains("async fn test_async"));
        assert!(output_str.contains("tracing::span!"));
    }

    #[test]
    fn test_unsafe_function() {
        let args = quote!();
        let item = quote! {
            unsafe fn test_unsafe() {
                println!("unsafe test");
            }
        };

        let result = instrument_impl(args, item);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = format_and_print(output);

        assert!(output_str.contains("unsafe fn test_unsafe"));
        assert!(output_str.contains("tracing::span!"));
    }

    #[test]
    fn test_const_function() {
        let args = quote!();
        let item = quote! {
            const fn test_const() {

            }
        };

        let result = instrument_impl(args, item);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = format_and_print(output);

        assert!(output_str.contains("const fn test_const"));
        assert!(output_str.contains("tracing::span!"));
    }

    #[test]
    fn test_pub_async_function() {
        let args = quote!();
        let item = quote! {
            pub async fn test_pub_async() {
                println!("pub async test");
            }
        };

        let result = instrument_impl(args, item);
        assert!(result.is_ok());

        let output = result.unwrap();
        let output_str = format_and_print(output);

        assert!(output_str.contains("pub async fn test_pub_async"));
        assert!(output_str.contains("tracing::span!"));
    }
}

#[cfg(test)]
mod extract_param_fields_tests {
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
}
