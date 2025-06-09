use crate::instrument_impl;
use proc_macro2::TokenStream;
use unsynn::*;

use crate::parse::{ImplBlockSig, ModuleContent, ModuleItem, ModuleSig, TraitSig};

pub(crate) struct TokenProcessor {
    input: TokenStream,
}

impl TokenProcessor {
    pub(crate) fn new(input: TokenStream) -> Self {
        Self { input }
    }

    pub(crate) fn process(self) -> TokenStream {
        match self
            .input
            .clone()
            .into_token_iter()
            .parse::<ModuleContent>()
        {
            Ok(parsed) => self.process_module_content(parsed),
            Err(_) => {
                // Fallback: if declarative parsing fails, use original input
                self.input
            }
        }
    }

    fn process_module_content(&self, content: ModuleContent) -> TokenStream {
        let mut output = TokenStream::new();

        for item in content.items.0 {
            let processed_item = self.process_module_item(item.value);
            output.extend(processed_item);
        }

        output
    }

    fn process_module_item(&self, item: ModuleItem) -> TokenStream {
        match item {
            ModuleItem::Function(func_sig) => {
                // Convert using unsynn's ToTokens trait
                let mut func_tokens = TokenStream::new();
                quote::ToTokens::to_tokens(&func_sig, &mut func_tokens);
                self.instrument_function(func_tokens)
            }
            ModuleItem::ImplBlock(impl_block) => self.process_impl_block(impl_block),
            ModuleItem::Module(module) => self.process_module_block(module),
            ModuleItem::Trait(trait_def) => self.process_trait_block(trait_def),
            ModuleItem::Other(token) => {
                // Pass through other items unchanged
                let mut tokens = TokenStream::new();
                token.to_tokens(&mut tokens);
                tokens
            }
        }
    }

    fn process_impl_block(&self, impl_block: ImplBlockSig) -> TokenStream {
        // Process the body content to instrument any functions inside
        let processed_body = self.process_brace_group_content(impl_block.body.into());

        // Reconstruct the impl block with processed body
        let mut output = TokenStream::new();

        // Add attributes
        if let Some(attrs) = impl_block.attributes {
            for attr in attrs.0 {
                attr.to_tokens(&mut output);
            }
        }

        // Add impl keyword and generics
        impl_block._impl.to_tokens(&mut output);
        if let Some(generics) = impl_block.generics {
            generics.to_tokens(&mut output);
        }

        // Add target type
        for item in impl_block.target_type.0 {
            item.value.second.to_tokens(&mut output);
        }

        // Add "for Trait" if present
        if let Some(for_part) = impl_block.for_trait {
            for_part.to_tokens(&mut output);
        }

        // Add where clause if present
        if let Some(where_clause) = impl_block.where_clause {
            where_clause.to_tokens(&mut output);
        }

        // Add processed body
        output.extend(processed_body);

        output
    }

    fn process_module_block(&self, module: ModuleSig) -> TokenStream {
        // Process the module body content recursively
        let processed_body = self.process_brace_group_content(module.body.into());

        // Reconstruct the module with processed body
        let mut output = TokenStream::new();

        // Add attributes
        if let Some(attrs) = module.attributes {
            for attr in attrs.0 {
                attr.to_tokens(&mut output);
            }
        }

        // Add visibility
        if let Some(vis) = module.visibility {
            vis.to_tokens(&mut output);
        }

        // Add mod keyword and name
        module._mod.to_tokens(&mut output);
        module.name.to_tokens(&mut output);

        // Add processed body
        output.extend(processed_body);

        output
    }

    fn process_trait_block(&self, trait_def: TraitSig) -> TokenStream {
        // Process the trait body content to instrument any default implementations
        let processed_body = self.process_brace_group_content(trait_def.body.into());

        // Reconstruct the trait with processed body
        let mut output = TokenStream::new();

        // Add attributes
        if let Some(attrs) = trait_def.attributes {
            for attr in attrs.0 {
                attr.to_tokens(&mut output);
            }
        }

        // Add visibility
        if let Some(vis) = trait_def.visibility {
            vis.to_tokens(&mut output);
        }

        // Add unsafe if present
        if let Some(unsafe_kw) = trait_def.unsafe_kw {
            unsafe_kw.to_tokens(&mut output);
        }

        // Add trait keyword and name
        trait_def._trait.to_tokens(&mut output);
        trait_def.name.to_tokens(&mut output);

        // Add generics if present
        if let Some(generics) = trait_def.generics {
            generics.to_tokens(&mut output);
        }

        // Add bounds if present
        if let Some(bounds) = trait_def.bounds {
            bounds.to_tokens(&mut output);
        }

        // Add where clause if present
        if let Some(where_clause) = trait_def.where_clause {
            where_clause.to_tokens(&mut output);
        }

        // Add processed body
        output.extend(processed_body);

        output
    }

    fn process_brace_group_content(&self, brace_group: proc_macro2::Group) -> TokenStream {
        // Recursively process the content inside braces
        let inner_content = brace_group.stream();
        let processed_inner = TokenProcessor::new(inner_content).process();

        // Wrap in braces again
        let mut output = TokenStream::new();
        let new_group = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, processed_inner);
        output.extend(std::iter::once(proc_macro2::TokenTree::Group(new_group)));
        output
    }

    fn instrument_function(&self, func_tokens: TokenStream) -> TokenStream {
        match instrument_impl(TokenStream::new(), func_tokens.clone()) {
            Ok(instrumented) => instrumented,
            Err(e) => {
                eprintln!("instrument_impl failed: {}", e);
                func_tokens // fallback to original
            }
        }
    }
}

#[cfg(test)]
mod tests;
