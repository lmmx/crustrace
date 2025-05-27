#![warn(missing_docs)]
#![warn(clippy::std_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![forbid(unsafe_code)]
#![doc = include_str!("../../README.md")]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

/// Instruments a function to create and enter a `tracing` span every time
/// the function is called.
///
/// Unless overridden, a span with `info` level will be generated.
/// The generated span's name will be the name of the function.
/// By default, all arguments to the function are included as fields on the span.
///
/// # Examples
///
/// Instrumenting a function:
/// ```
/// # use crustrace::instrument;
/// #[instrument]
/// pub fn my_function(my_arg: usize) {
///     // This creates a span named `my_function` with field `my_arg`
///     println!("inside my_function!");
/// }
/// ```
///
/// Setting the level for the generated span:
/// ```
/// # use crustrace::instrument;
/// #[instrument(level = "debug")]
/// pub fn my_function() {
///     // Creates a DEBUG level span
/// }
/// ```
///
/// Overriding the generated span's name:
/// ```
/// # use crustrace::instrument;
/// #[instrument(name = "my_custom_name")]
/// pub fn my_function() {
///     // Creates a span named `my_custom_name`
/// }
/// ```
#[proc_macro_attribute]
pub fn instrument(args: TokenStream, item: TokenStream) -> TokenStream {
    let args2: TokenStream2 = args.into();
    let item2: TokenStream2 = item.into();

    match crustrace_core::instrument_impl(args2, item2) {
        Ok(tokens) => tokens.into(),
        Err(error_tokens) => error_tokens.into(),
    }
}

/// Instruments all functions within a module or impl block with tracing spans.
///
/// This macro applies the instrumentation behavior to every function found within
/// the annotated module or impl block, automatically creating tracing spans for
/// each function call. This provides a convenient way to add comprehensive tracing
/// to an entire module without having to annotate each function individually.
///
/// The generated spans will use the default configuration (info level, function name
/// as span name, and all function arguments as fields) unless the individual functions
/// are also decorated with `#[instrument]` with custom parameters.
///
/// # Examples
///
/// Instrumenting all functions in a module:
/// ```
/// # use crustrace::omni;
/// #[omni]
/// mod my_module {
///     pub fn function_one(x: i32) {
///         // Automatically gets a span named `function_one` with field `x`
///         println!("Function one called with {}", x);
///     }
///     
///     pub fn function_two() {
///         // Automatically gets a span named `function_two`
///         println!("Function two called");
///     }
/// }
/// ```
///
/// Instrumenting all methods in an impl block:
/// ```
/// # use crustrace::omni;
/// struct MyStruct;
///
/// #[omni]
/// impl MyStruct {
///     pub fn method_one(&self, value: String) {
///         // Automatically gets a span named `method_one` with field `value`
///         println!("Method called with {}", value);
///     }
///     
///     pub fn method_two(&self) {
///         // Automatically gets a span named `method_two`
///         println!("Another method called");
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn omni(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input2: TokenStream2 = input.into();
    let output = crustrace_core::trace_all_impl(input2);
    output.into()
}
