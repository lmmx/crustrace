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
