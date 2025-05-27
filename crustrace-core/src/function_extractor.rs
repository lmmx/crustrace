use proc_macro2::{Delimiter, TokenStream, TokenTree};

#[derive(Debug)]
pub(crate) struct FunctionInfo {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) tokens: TokenStream,
}

pub(crate) struct FunctionExtractor;

impl FunctionExtractor {
    pub(crate) fn extract(input: TokenStream) -> Vec<FunctionInfo> {
        let tokens: Vec<TokenTree> = input.into_iter().collect();
        let mut functions = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            if let Some(func_info) = Self::try_extract_function(&tokens, i) {
                i = func_info.end;
                functions.push(func_info);
            } else {
                i += 1;
            }
        }

        functions
    }

    pub(crate) fn try_extract_function(tokens: &[TokenTree], start: usize) -> Option<FunctionInfo> {
        if !Self::is_function_start(tokens, start) {
            return None;
        }

        let end = Self::find_function_end(tokens, start)?;
        let func_tokens = tokens[start..=end].iter().cloned().collect();

        Some(FunctionInfo {
            start,
            end: end + 1,
            tokens: func_tokens,
        })
    }

    pub(crate) fn is_function_start(tokens: &[TokenTree], i: usize) -> bool {
        match tokens.get(i) {
            Some(TokenTree::Ident(ident)) => {
                match ident.to_string().as_str() {
                    "fn" => {
                        // Only a function if not preceded by "pub"
                        !(i > 0
                            && matches!(tokens.get(i - 1), Some(TokenTree::Ident(prev)) if prev == "pub"))
                    }
                    "pub" => {
                        // Check if this is "pub fn"
                        matches!(tokens.get(i + 1), Some(TokenTree::Ident(next)) if next == "fn")
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub(crate) fn find_function_end(tokens: &[TokenTree], start: usize) -> Option<usize> {
        for (i, token) in tokens.iter().enumerate().skip(start) {
            match token {
                TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => return Some(i),
                TokenTree::Punct(punct) if punct.as_char() == ';' => return None,
                _ => {}
            }
        }
        None
    }
}
