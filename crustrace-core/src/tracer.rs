//! Procedural macro attributes for automatically instrumenting functions with `tracing`.
//!
//! This crate provides the [`#[instrument]`] attribute macro using `unsynn` for parsing,
//! offering a lightweight alternative to the standard `tracing-attributes` crate.

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
}

struct SimpleFunction {
    attrs: Vec<TokenStream>,
    vis: Option<TokenStream>,
    const_kw: Option<TokenStream>,
    async_kw: Option<TokenStream>,
    unsafe_kw: Option<TokenStream>,
    extern_kw: Option<TokenStream>,
    fn_name: proc_macro2::Ident,
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

            #body
        }
    }
}

/// Extract parameter names from function parameters for tracing fields
/// This is a simplified version that attempts basic parameter extraction
fn extract_param_fields(params: &TokenStream) -> TokenStream {
    // Simple heuristic: look for identifiers that could be parameter names
    let params_clone = params.clone();
    let mut param_iter = params_clone.to_token_iter();
    let mut fields = Vec::new();
    let mut prev_token: Option<TokenTree> = None;

    while let Ok(token) = TokenTree::parse(&mut param_iter) {
        if let TokenTree::Ident(ref ident) = token {
            let ident_str = ident.to_string();

            // Skip keywords and types
            if ![
                "mut", "self", "Self", "ref", "&", "usize", "i32", "u32", "i64", "u64", "String",
                "str", "bool", "f32", "f64",
            ]
            .contains(&ident_str.as_str())
            {
                // Check if previous token was ":" which would indicate this is a type, not a param name
                let is_type =
                    matches!(&prev_token, Some(TokenTree::Punct(p)) if p.as_char() == ':');

                if !is_type && ident_str.chars().next().is_some_and(|c| c.is_lowercase()) {
                    // This looks like a parameter name
                    fields.push(quote!(, #ident = #ident));
                }
            }
        }
        prev_token = Some(token);
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
