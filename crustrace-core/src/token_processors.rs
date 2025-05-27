use crate::instrument_impl;
use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};

use crate::function_extractor::*;

pub(crate) struct TokenProcessor {
    tokens: Vec<TokenTree>,
}

#[derive(Debug)]
pub(crate) enum ItemType {
    Module,
    Impl,
    Trait,
    Function,
    Other,
}

impl TokenProcessor {
    pub(crate) fn new(input: TokenStream) -> Self {
        Self {
            tokens: input.into_iter().collect(),
        }
    }

    pub(crate) fn process(self) -> TokenStream {
        self.process_items(0).0
    }

    pub(crate) fn process_items(&self, start: usize) -> (TokenStream, usize) {
        let mut output = TokenStream::new();
        let mut i = start;

        while i < self.tokens.len() {
            match self.identify_item(i) {
                ItemType::Module => {
                    let (processed, next_i) =
                        self.process_container_item(i, Self::process_module_body);
                    output.extend(processed);
                    i = next_i;
                }
                ItemType::Impl => {
                    let (processed, next_i) =
                        self.process_container_item(i, Self::process_impl_body);
                    output.extend(processed);
                    i = next_i;
                }
                ItemType::Trait => {
                    let (processed, next_i) =
                        self.process_container_item(i, Self::process_trait_body);
                    output.extend(processed);
                    i = next_i;
                }
                ItemType::Function => {
                    let (processed, next_i) = self.process_functions_from(i);
                    output.extend(processed);
                    return (output, next_i);
                }
                ItemType::Other => {
                    output.extend(std::iter::once(self.tokens[i].clone()));
                    i += 1;
                }
            }
        }

        (output, i)
    }

    fn identify_item(&self, i: usize) -> ItemType {
        match &self.tokens[i] {
            TokenTree::Ident(ident) => match ident.to_string().as_str() {
                "mod" => ItemType::Module,
                "impl" => ItemType::Impl,
                "trait" => ItemType::Trait,
                "pub" if self.is_pub_fn(i) => ItemType::Function,
                "fn" if !self.is_preceded_by_pub(i) => ItemType::Function,
                _ => ItemType::Other,
            },
            _ => ItemType::Other,
        }
    }

    fn is_pub_fn(&self, i: usize) -> bool {
        i + 1 < self.tokens.len()
            && matches!(&self.tokens[i + 1], TokenTree::Ident(ident) if ident == "fn")
    }

    fn is_preceded_by_pub(&self, i: usize) -> bool {
        i > 0 && matches!(&self.tokens[i - 1], TokenTree::Ident(ident) if ident == "pub")
    }

    fn process_container_item<F>(&self, start: usize, body_processor: F) -> (TokenStream, usize)
    where
        F: Fn(&Self, TokenStream) -> TokenStream,
    {
        if let Some(brace_idx) = self.find_brace_group(start) {
            let mut output = TokenStream::new();

            // Add everything before the brace
            output.extend(self.tokens[start..brace_idx].iter().cloned());

            // Process the body
            if let TokenTree::Group(group) = &self.tokens[brace_idx] {
                let processed_body = body_processor(self, group.stream());
                let new_group = Group::new(group.delimiter(), processed_body);
                output.extend(std::iter::once(TokenTree::Group(new_group)));
            }

            (output, brace_idx + 1)
        } else {
            // Fallback: no brace found, return as-is
            let output = self.tokens[start..].iter().cloned().collect();
            (output, self.tokens.len())
        }
    }

    fn process_module_body(&self, body: TokenStream) -> TokenStream {
        TokenProcessor::new(body).process()
    }

    fn process_impl_body(&self, body: TokenStream) -> TokenStream {
        self.process_function_container(body)
    }

    fn process_trait_body(&self, body: TokenStream) -> TokenStream {
        self.process_function_container(body)
    }

    fn process_function_container(&self, body: TokenStream) -> TokenStream {
        let functions = FunctionExtractor::extract(body.clone());

        if functions.is_empty() {
            return body;
        }

        let mut output = TokenStream::new();
        let body_tokens: Vec<TokenTree> = body.into_iter().collect();
        let mut last_end = 0;

        for func in functions {
            // Add tokens before this function
            output.extend(body_tokens[last_end..func.start].iter().cloned());

            // Add instrumented function
            let instrumented = self.instrument_function(func.tokens);
            output.extend(instrumented);

            last_end = func.end;
        }

        // Add remaining tokens
        output.extend(body_tokens[last_end..].iter().cloned());
        output
    }

    fn process_functions_from(&self, start: usize) -> (TokenStream, usize) {
        let remaining: TokenStream = self.tokens[start..].iter().cloned().collect();
        (
            self.process_function_container(remaining),
            self.tokens.len(),
        )
    }

    fn instrument_function(&self, func_tokens: TokenStream) -> TokenStream {
        instrument_impl(TokenStream::new(), func_tokens.clone()).unwrap_or(func_tokens)
    }

    fn find_brace_group(&self, start: usize) -> Option<usize> {
        (start..self.tokens.len())
            .find(|&i| matches!(&self.tokens[i], TokenTree::Group(g) if g.delimiter() == Delimiter::Brace))
    }
}
