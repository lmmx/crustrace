use proc_macro2::{Group, TokenStream, TokenTree};
use quote::quote;

mod tracer;
pub use tracer::instrument_impl;

pub fn trace_all_impl(input: TokenStream) -> TokenStream {
    let mut tokens = input.clone().into_iter().peekable();

    // Check if this is a module
    if let Some(TokenTree::Ident(ident)) = tokens.next() {
        if ident == "mod" {
            // Get the module name
            if let Some(mod_name) = tokens.next() {
                // Get the module body
                if let Some(TokenTree::Group(group)) = tokens.next() {
                    // Process the module contents
                    let processed_contents = instrument_functions(group.stream());

                    // Reconstruct the module with processed contents
                    let mut output = TokenStream::new();
                    output.extend(quote! { mod });
                    output.extend(std::iter::once(mod_name));
                    let new_group = Group::new(group.delimiter(), processed_contents);
                    output.extend(std::iter::once(TokenTree::Group(new_group)));
                    return output;
                }
            }
        }
    }

    // Not a module, process normally
    instrument_functions(input)
}

fn instrument_functions(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let mut output = TokenStream::new();
    let mut i = 0;

    while i < tokens.len() {
        // Check if we're at the start of a function declaration
        if let TokenTree::Ident(ident) = &tokens[i] {
            // Case 1: "pub fn identifier"
            if *ident == "pub" && i + 2 < tokens.len() {
                if let (TokenTree::Ident(fn_ident), TokenTree::Ident(_)) =
                    (&tokens[i + 1], &tokens[i + 2])
                {
                    if *fn_ident == "fn" {
                        // Insert attribute before pub
                        let instrument = quote! {
                            #[tracing::instrument(level = "info", ret)]
                        };
                        output.extend(instrument);
                    }
                }
            }
            // Case 2: "fn identifier" (but not preceded by pub)
            else if *ident == "fn"
                && i + 1 < tokens.len()
                && matches!(&tokens[i + 1], TokenTree::Ident(_))
            {
                // Check if this fn is NOT preceded by pub
                let not_preceded_by_pub = i == 0 || {
                    if let TokenTree::Ident(prev_ident) = &tokens[i - 1] {
                        *prev_ident != "pub"
                    } else {
                        true
                    }
                };

                if not_preceded_by_pub {
                    // Insert attribute before fn
                    let instrument = quote! {
                        #[crustrace::instrument(level = "info", ret)]
                    };
                    output.extend(instrument);
                }
            }
        }

        output.extend(std::iter::once(tokens[i].clone()));
        i += 1;
    }

    output
}
