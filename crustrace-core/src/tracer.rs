//! Procedural macro attributes for automatically instrumenting functions with `tracing`.
//!
//! This crate provides the [`#[instrument]`] attribute macro using `unsynn` for parsing,
//! offering a lightweight alternative to the standard `tracing-attributes` crate.

use core::result::Result;
use proc_macro2::TokenStream;
use quote::quote;
use unsynn::*;

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
    fn_name: proc_macro2::Ident,
    generics: Option<TokenStream>,
    params: TokenStream,
    ret_type: Option<TokenStream>,
    body: TokenStream,
}

fn parse_instrument_args(input: &mut TokenIter) -> Result<InstrumentArgs, String> {
    let mut args = InstrumentArgs::default();

    while let Ok(ident) = Ident::parse(input) {
        let ident_str = ident.to_string();

        if ident_str == "level" {
            if Operator::<'='>::parse(input).is_ok() {
                if let Ok(level_lit) = LiteralString::parse(input) {
                    args.level = Some(level_lit.as_str().to_string());
                }
            }
        } else if ident_str == "name" && Operator::<'='>::parse(input).is_ok() {
            if let Ok(name_lit) = LiteralString::parse(input) {
                args.name = Some(name_lit.as_str().to_string());
            }
        }

        // Skip optional comma
        let _ = Comma::parse(input);

        // Prevent infinite loop
        if input.counter() > 100 {
            break;
        }
    }

    Ok(args)
}

fn parse_simple_function(input: &mut TokenIter) -> Result<SimpleFunction, String> {
    let mut attrs = Vec::new();
    let mut vis = None;

    // Parse attributes (#[...])
    while Operator::<'#'>::parse(input).is_ok() {
        if let Ok(bracket_group) = BracketGroup::parse(input) {
            let mut attr_tokens = TokenStream::new();
            Operator::<'#'>::new().to_tokens(&mut attr_tokens);
            bracket_group.to_tokens(&mut attr_tokens);
            attrs.push(attr_tokens);
        }
    }

    // Parse visibility (pub, pub(crate), etc.) or fn
    let first_ident = match Ident::parse(input) {
        Ok(ident) => ident,
        Err(_) => return Err("Expected 'pub' or 'fn' keyword".to_string()),
    };

    let fn_kw = if first_ident == "pub" {
        let mut vis_tokens = TokenStream::new();
        first_ident.to_tokens(&mut vis_tokens);

        // Check for pub(crate), pub(super), etc.
        if let Ok(paren_group) = ParenthesisGroup::parse(input) {
            paren_group.to_tokens(&mut vis_tokens);
        }

        vis = Some(vis_tokens);

        // Now parse the fn keyword
        match Ident::parse(input) {
            Ok(ident) => ident,
            Err(_) => return Err("Expected 'fn' keyword after visibility".to_string()),
        }
    } else if first_ident == "fn" {
        // No visibility, this is the fn keyword
        first_ident
    } else {
        return Err(format!("Expected 'pub' or 'fn', found '{}'", first_ident));
    };
    if fn_kw != "fn" {
        return Err("Expected 'fn' keyword".to_string());
    }

    // Parse function name
    let fn_name = match Ident::parse(input) {
        Ok(ident) => ident,
        Err(_) => return Err("Expected function name".to_string()),
    };

    // Parse optional generics
    let mut generics = None;
    if let Ok(angle_group) = parse_angle_brackets(input) {
        generics = Some(angle_group);
    }

    // Parse parameters
    let params_group = match ParenthesisGroup::parse(input) {
        Ok(group) => group,
        Err(_) => return Err("Expected function parameters".to_string()),
    };
    let mut params = TokenStream::new();
    params_group.to_tokens(&mut params);

    // Parse optional return type
    let mut ret_type = None;
    if Operator::<'-', '>'>::parse(input).is_ok() {
        let mut ret_tokens = TokenStream::new();
        Operator::<'-', '>'>::new().to_tokens(&mut ret_tokens);

        // Collect tokens until we see a brace, but don't consume the brace
        loop {
            let mut cloned_input = input.clone();
            if let Ok(next_token) = TokenTree::parse(&mut cloned_input) {
                if let TokenTree::Group(ref group) = next_token {
                    if group.delimiter() == Delimiter::Brace {
                        // Stop here, don't consume the brace
                        break;
                    }
                }
                // Consume and add the token from the real input
                if let Ok(token) = TokenTree::parse(input) {
                    token.to_tokens(&mut ret_tokens);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        ret_type = Some(ret_tokens);
    }

    // Parse function body - if we consumed a brace in return type parsing, handle that
    // println!(
    //     "Remaining tokens before body parsing: {}",
    //     input.to_token_stream()
    // );

    let body_group = match BraceGroup::parse(input) {
        Ok(group) => {
            // println!("Successfully parsed body group");
            group
        }
        Err(e) => {
            eprintln!("Failed to parse body group: {:?}", e);

            // Try to see what the next token actually is
            if let Ok(next_token) = TokenTree::parse(input) {
                eprintln!("Next token is: {:?}", next_token);
                match next_token {
                    TokenTree::Group(group) => {
                        eprintln!("It's a group with delimiter: {:?}", group.delimiter());
                        if group.delimiter() == Delimiter::Brace {
                            eprintln!("It IS a brace group! Using it directly.");
                            // Use this group directly
                            let mut body = TokenStream::new();
                            group.to_tokens(&mut body);

                            return Ok(SimpleFunction {
                                attrs,
                                vis,
                                fn_name,
                                generics,
                                params,
                                ret_type,
                                body,
                            });
                        }
                    }
                    _ => eprintln!("It's not a group"),
                }
            }

            return Err("Expected function body".to_string());
        }
    };

    // Handle the successful BraceGroup::parse case
    let mut body = TokenStream::new();
    body_group.to_tokens(&mut body);

    Ok(SimpleFunction {
        attrs,
        vis,
        fn_name,
        generics,
        params,
        ret_type,
        body,
    })
}

fn parse_angle_brackets(input: &mut TokenIter) -> unsynn::Result<TokenStream> {
    // Look for < ... > generics manually since unsynn doesn't have AngleBracketGroup
    if let Ok(lt) = Operator::<'<'>::parse(input) {
        let mut generics = TokenStream::new();
        lt.to_tokens(&mut generics);

        let mut depth = 1;
        while depth > 0 {
            if let Ok(token) = TokenTree::parse(input) {
                match &token {
                    TokenTree::Punct(p) if p.as_char() == '<' => depth += 1,
                    TokenTree::Punct(p) if p.as_char() == '>' => depth -= 1,
                    _ => {}
                }
                token.to_tokens(&mut generics);
            } else {
                break;
            }
        }

        Ok(generics)
    } else {
        Err(Error::unexpected_token(input)?)
    }
}

fn generate_instrumented_function(args: InstrumentArgs, func: SimpleFunction) -> TokenStream {
    let SimpleFunction {
        attrs,
        vis,
        fn_name,
        generics,
        params,
        ret_type,
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

    // Generate generics tokens
    let generics_tokens = generics.unwrap_or_default();

    // Generate return type tokens
    let ret_tokens = ret_type.unwrap_or_default();

    // Generate the instrumented function
    quote! {
        #(#attrs)*
        #vis_tokens fn #fn_name #generics_tokens #params #ret_tokens {
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
}
