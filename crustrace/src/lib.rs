use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

/// Instruments a function to create and enter a `tracing` [span] every time
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

#[proc_macro_attribute]
pub fn trace_all(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input2: TokenStream2 = input.into();
    let output = crustrace_core::trace_all_impl(input2);
    output.into()
}
