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
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_basic_function_processing() {
        let input = quote! { fn hello() { println!("world"); } };
        let processor = TokenProcessor::new(input.clone());
        let output = processor.process();

        println!("Input: {}", input);
        println!("Output: {}", output);

        let output_str = output.to_string();
        assert!(output_str.contains("fn hello"));
        assert!(output_str.contains("tracing :: span !"));
    }

    #[test]
    fn test_async_function_processing() {
        let input = quote! { async fn hello() { println!("world"); } };
        let processor = TokenProcessor::new(input.clone());
        let output = processor.process();

        println!("Input: {}", input);
        println!("Output: {}", output);

        let output_str = output.to_string();
        assert!(
            output_str.contains("async fn hello"),
            "Should preserve async keyword"
        );
        assert!(
            output_str.contains("tracing :: span !"),
            "Should add instrumentation"
        );
    }

    #[test]
    fn test_unsafe_function_processing() {
        let input = quote! { unsafe fn hello() { println!("world"); } };
        let processor = TokenProcessor::new(input.clone());
        let output = processor.process();

        println!("Input: {}", input);
        println!("Output: {}", output);

        let output_str = output.to_string();
        assert!(
            output_str.contains("unsafe fn hello"),
            "Should preserve unsafe keyword"
        );
    }

    #[test]
    fn test_const_function_processing() {
        let input = quote! { const fn hello() { println!("world"); } };
        let processor = TokenProcessor::new(input.clone());
        let output = processor.process();

        println!("Input: {}", input);
        println!("Output: {}", output);

        let output_str = output.to_string();
        assert!(
            output_str.contains("const fn hello"),
            "Should preserve const keyword"
        );
    }

    #[test]
    fn test_extern_function_processing() {
        let input = quote! { extern "C" fn hello() { println!("world"); } };
        let processor = TokenProcessor::new(input.clone());
        let output = processor.process();

        println!("Input: {}", input);
        println!("Output: {}", output);

        let output_str = output.to_string();
        assert!(
            output_str.contains("extern"),
            "Should preserve extern keyword"
        );
        assert!(output_str.contains("\"C\""), "Should preserve ABI");
    }

    #[test]
    fn test_pub_async_function_processing() {
        let input = quote! { pub async fn hello() { println!("world"); } };
        let processor = TokenProcessor::new(input.clone());
        let output = processor.process();

        println!("Input: {}", input);
        println!("Output: {}", output);

        let output_str = output.to_string();
        assert!(
            output_str.contains("pub async fn hello"),
            "Should preserve pub async"
        );
    }

    #[test]
    fn test_module_content_parsing() {
        let input = quote! { async fn hello() { println!("world"); } };

        println!("=== STEP BY STEP DEBUG ===");
        println!("Input: {}", input);

        // Test if ModuleContent parsing works
        let mut iter = input.clone().into_token_iter();
        match iter.parse::<ModuleContent>() {
            Ok(content) => {
                println!("✅ ModuleContent parsed successfully");
                println!("Number of items: {}", content.items.0.len());

                if let Some(first_item) = content.items.0.first() {
                    match &first_item.value {
                        ModuleItem::Function(func_sig) => {
                            println!("✅ Found function");
                            println!("  async_kw: {:?}", func_sig.async_kw.is_some());
                            println!("  name: {}", func_sig.name);

                            // Test conversion back to tokens
                            let mut converted = TokenStream::new();
                            func_sig.to_tokens(&mut converted);
                            println!("Converted function: {}", converted);
                        }
                        _ => println!("❌ First item is not a function"),
                    }
                }
            }
            Err(e) => {
                println!("❌ ModuleContent parsing failed: {}", e);
                println!("This would cause fallback to original input!");
            }
        }
    }

    #[test]
    fn test_process_module_item_function() {
        let input = quote! { async fn hello() { println!("world"); } };

        // Parse the content manually to test process_module_item
        let mut iter = input.into_token_iter();
        let content = iter.parse::<ModuleContent>().expect("Should parse");

        if let Some(first_item) = content.items.0.into_iter().next() {
            match first_item.value {
                ModuleItem::Function(func_sig) => {
                    println!("=== TESTING process_module_item ===");
                    println!("Function before processing:");
                    println!("  async_kw: {:?}", func_sig.async_kw.is_some());

                    // Create a processor to call the method
                    let dummy_input = quote! {};
                    let processor = TokenProcessor::new(dummy_input);

                    let result = processor.process_module_item(ModuleItem::Function(func_sig));
                    println!("Processed result: {}", result);

                    let result_str = result.to_string();
                    assert!(
                        result_str.contains("async"),
                        "Should preserve async in process_module_item"
                    );
                }
                _ => panic!("Expected function"),
            }
        } else {
            panic!("No items found");
        }
    }

    #[test]
    fn test_fallback_behavior() {
        // Test with something that shouldn't parse as ModuleContent
        let input = quote! { this is not valid rust };
        let processor = TokenProcessor::new(input.clone());
        let output = processor.process();

        println!("Invalid input: {}", input);
        println!("Fallback output: {}", output);

        // Should fallback to original input
        assert_eq!(output.to_string(), input.to_string());
    }

    #[test]
    fn test_impl_block_processing() {
        let input = quote! {
            impl MyStruct {
                async fn method(&self) {
                    println!("async method");
                }
            }
        };

        let processor = TokenProcessor::new(input.clone());
        let output = processor.process();

        println!("Impl block input: {}", input);
        println!("Impl block output: {}", output);

        let output_str = output.to_string();
        assert!(
            output_str.contains("async fn method"),
            "Should preserve async in impl block"
        );
        assert!(
            output_str.contains("tracing :: span !"),
            "Should add instrumentation"
        );
    }

    #[test]
    fn test_pipeline_debug() {
        let input = quote! { async fn hello() { println!("world"); } };

        println!("=== FULL PIPELINE DEBUG ===");
        println!("1. Input: {}", input);

        // Step 1: Create processor
        let processor = TokenProcessor::new(input.clone());

        // Step 2: Test what process() does step by step
        let result = match input.clone().into_token_iter().parse::<ModuleContent>() {
            Ok(parsed) => {
                println!("2. ✅ ModuleContent parsing succeeded");
                processor.process_module_content(parsed)
            }
            Err(e) => {
                println!("2. ❌ ModuleContent parsing failed: {}", e);
                println!("   Falling back to original input");
                input.clone()
            }
        };

        println!("3. Final result: {}", result);

        let result_str = result.to_string();
        if result_str.contains("async") {
            println!("✅ async keyword preserved!");
        } else {
            println!("❌ async keyword lost! {}", result_str);
        }
    }
}
