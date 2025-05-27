use crate::instrument_impl;
use proc_macro2::{Group, TokenStream, TokenTree}; // Import the robust function from tracer.rs

pub fn trace_all_impl(input: TokenStream) -> TokenStream {
    let mut tokens = input.clone().into_iter().peekable();

    // Check if this is a module
    if let Some(TokenTree::Ident(ident)) = tokens.peek() {
        if *ident == "mod" {
            return process_module(input);
        }
    }

    // Check if this is an impl block
    if let Some(TokenTree::Ident(ident)) = tokens.peek() {
        if *ident == "impl" {
            return process_impl_block(input);
        }
    }

    // Not a module or impl block, process as standalone functions
    process_functions(input)
}

fn process_module(input: TokenStream) -> TokenStream {
    let mut tokens = input.into_iter();

    // Get "mod"
    let Some(mod_keyword) = tokens.next() else {
        // If we can't get the mod keyword, reconstruct what we have
        let mut output = TokenStream::new();
        output.extend(tokens);
        return output;
    };

    // Get module name
    let Some(mod_name) = tokens.next() else {
        // If we can't get the module name, reconstruct what we have
        let mut output = TokenStream::new();
        output.extend(core::iter::once(mod_keyword));
        output.extend(tokens);
        return output;
    };

    // Get the module body
    if let Some(TokenTree::Group(group)) = tokens.next() {
        // Recursively process the module contents
        let processed_contents = trace_all_impl(group.stream());

        // Reconstruct the module with processed contents
        let mut output = TokenStream::new();
        output.extend(core::iter::once(mod_keyword));
        output.extend(core::iter::once(mod_name));
        let new_group = Group::new(group.delimiter(), processed_contents);
        output.extend(core::iter::once(TokenTree::Group(new_group)));

        // Add any remaining tokens (shouldn't be any for well-formed modules)
        output.extend(tokens);

        return output;
    }

    // Fallback: collect remaining tokens and return them
    let mut output = TokenStream::new();
    output.extend(core::iter::once(mod_keyword));
    output.extend(core::iter::once(mod_name));
    output.extend(tokens);
    output
}

fn process_impl_block(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let mut i = 0;

    // Find the opening brace of the impl block
    while i < tokens.len() {
        if let TokenTree::Group(group) = &tokens[i] {
            if group.delimiter() == proc_macro2::Delimiter::Brace {
                // Found the impl body - process all functions inside it
                let processed_body = process_functions(group.stream());

                // Reconstruct the impl block
                let mut output = TokenStream::new();
                // Add everything before the brace (impl signature)
                for tok_j in tokens.iter().take(i) {
                    output.extend(core::iter::once(tok_j.clone()));
                }
                // Add the processed body
                let new_group = Group::new(group.delimiter(), processed_body);
                output.extend(core::iter::once(TokenTree::Group(new_group)));
                // Add everything after the brace (shouldn't be anything)
                for tok_j in tokens.iter().skip(i + 1) {
                    output.extend(core::iter::once(tok_j.clone()));
                }
                return output;
            }
        }
        i += 1;
    }

    // Fallback: reconstruct from tokens if no brace found
    tokens.into_iter().collect()
}

fn process_functions(input: TokenStream) -> TokenStream {
    let functions = extract_functions(input.clone());
    let mut output = TokenStream::new();
    let mut processed_ranges = Vec::new();

    // Process each function using the robust instrument_impl
    for func_info in functions {
        let instrumented = match instrument_impl(TokenStream::new(), func_info.tokens.clone()) {
            Ok(instrumented) => instrumented,
            Err(_) => {
                // If instrumentation fails, keep the original function
                func_info.tokens
            }
        };

        // Add any tokens before this function
        if let Some(last_end) = processed_ranges.last() {
            output.extend(extract_tokens_between(&input, *last_end, func_info.start));
        } else {
            output.extend(extract_tokens_between(&input, 0, func_info.start));
        }

        // Add the instrumented function
        output.extend(instrumented);
        processed_ranges.push(func_info.end);
    }

    // Add any remaining tokens after the last function
    if let Some(last_end) = processed_ranges.last() {
        output.extend(extract_tokens_from(&input, *last_end));
    } else {
        // No functions found, return original input
        return input;
    }

    output
}

#[derive(Debug)]
struct FunctionInfo {
    start: usize,
    end: usize,
    tokens: TokenStream,
}

fn extract_functions(input: TokenStream) -> Vec<FunctionInfo> {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let mut functions = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let func_start = i;

        // Look for function patterns
        let is_function = match &tokens[i] {
            // Case 1: "pub fn"
            TokenTree::Ident(ident) if *ident == "pub" => {
                i + 1 < tokens.len()
                    && matches!(&tokens[i + 1], TokenTree::Ident(fn_ident) if *fn_ident == "fn")
            }
            // Case 2: "fn" (not preceded by pub)
            TokenTree::Ident(ident) if *ident == "fn" => {
                // Make sure it's not preceded by "pub"
                func_start == 0
                    || !matches!(&tokens[func_start - 1], TokenTree::Ident(prev) if *prev == "pub")
            }
            _ => false,
        };

        if is_function {
            // Find the end of the function (look for the closing brace of the function body)
            if let Some(func_end) = find_function_end(&tokens, i) {
                // Extract the function tokens
                let func_tokens: TokenStream =
                    tokens[func_start..=func_end].iter().cloned().collect();

                functions.push(FunctionInfo {
                    start: func_start,
                    end: func_end + 1, // +1 because we want the position after the function
                    tokens: func_tokens,
                });

                i = func_end + 1;
                continue;
            }
        }

        i += 1;
    }

    functions
}

fn find_function_end(tokens: &[TokenTree], start: usize) -> Option<usize> {
    let mut i = start;

    // Skip to the function body (look for opening brace)
    while i < tokens.len() {
        if let TokenTree::Group(group) = &tokens[i] {
            if group.delimiter() == proc_macro2::Delimiter::Brace {
                // Found the function body, this token contains the entire body
                return Some(i);
            }
        }
        i += 1;
    }

    None
}

fn extract_tokens_between(input: &TokenStream, start: usize, end: usize) -> TokenStream {
    if start >= end {
        return TokenStream::new();
    }

    let tokens: Vec<TokenTree> = input.clone().into_iter().collect();
    if end > tokens.len() {
        return TokenStream::new();
    }

    tokens[start..end].iter().cloned().collect()
}

fn extract_tokens_from(input: &TokenStream, start: usize) -> TokenStream {
    let tokens: Vec<TokenTree> = input.clone().into_iter().collect();
    if start >= tokens.len() {
        return TokenStream::new();
    }

    tokens[start..].iter().cloned().collect()
}
