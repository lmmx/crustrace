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
        match &tokens[i] {
            TokenTree::Ident(ident) if ident == "pub" => {
                i + 1 < tokens.len()
                    && matches!(&tokens[i + 1], TokenTree::Ident(fn_ident) if fn_ident == "fn")
            }
            TokenTree::Ident(ident) if ident == "fn" => {
                i == 0 || !matches!(&tokens[i - 1], TokenTree::Ident(prev) if prev == "pub")
            }
            _ => false,
        }
    }

    pub(crate) fn find_function_end(tokens: &[TokenTree], start: usize) -> Option<usize> {
        for (i, tok_i) in tokens.iter().enumerate().skip(start) {
            match &tok_i {
                TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => return Some(i),
                TokenTree::Punct(punct) if punct.as_char() == ';' => return None,
                _ => {}
            }
        }
        None
    }
}
