# Crustrace TODO: Feature Parity with tracing-attributes

## 1. Return Value Logging (`ret` parameter)

### Summary
Add support for logging function return values when `#[instrument(ret)]` is specified, including support for customizing the level and format mode.

### What's Missing
- No support for `ret` parameter in `InstrumentArgs`
- No wrapping of function body to capture return values
- No event emission for return values
- No support for `ret(level = "...")` or `ret(Display)`/`ret(Debug)` formatting options

### How tracing-attributes Does It
- Parses `ret` in `attr.rs` as part of `InstrumentArgs` with optional `EventArgs`
- In `expand.rs`, wraps the function body to capture the return value
- Emits a tracing event with the return value before returning
- Supports different formatting modes (Debug/Display) and custom levels

### Implementation Guide
**Files to edit:**
- `crustrace-core/src/tracer.rs` - Add ret parsing and body wrapping logic
  - Look at `tracing-attributes/src/attr.rs` lines 122-126 for parsing
  - Look at `tracing-attributes/src/expand.rs` lines 236-275 for return event generation

**Key code to adapt from tracing-attributes:**
```rust
// From expand.rs - wrapping logic for return values
let ret_event = match args.ret_args {
    Some(event_args) => {
        let level_tokens = event_args.level(args_level);
        match event_args.mode {
            FormatMode::Display => Some(quote!(
                tracing::event!(target: #target, #level_tokens, return = %x)
            )),
            FormatMode::Default | FormatMode::Debug => Some(quote!(
                tracing::event!(target: #target, #level_tokens, return = ?x)
            )),
        }
    }
    _ => None,
};
```

## 2. Error Value Logging (`err` parameter)

### Summary
Add support for logging error values when `#[instrument(err)]` is specified on functions returning `Result<T, E>`.

### What's Missing
- No support for `err` parameter in `InstrumentArgs`
- No special handling for `Result` return types
- No error event emission on `Err` returns
- No support for `err(level = "...")` or formatting options

### How tracing-attributes Does It
- Parses `err` similar to `ret` with optional `EventArgs`
- Wraps function body with match statement for `Result` types
- Emits error-level events (by default) when function returns `Err`
- Supports customizing level and format (Debug/Display)

### Implementation Guide
**Files to edit:**
- `crustrace-core/src/tracer.rs` - Add err parsing and Result handling
  - Look at `tracing-attributes/src/attr.rs` lines 118-121 for parsing
  - Look at `tracing-attributes/src/expand.rs` lines 218-235 for error event generation

**Key code pattern from tracing-attributes:**
```rust
// Wrapping for both err and ret
match (err_event, ret_event) {
    (Some(err_event), Some(ret_event)) => quote_spanned!(block.span()=>
        match (move || #block)() {
            Ok(x) => {
                #ret_event;
                Ok(x)
            },
            Err(e) => {
                #err_event;
                Err(e)
            }
        }
    ),
    // ... other combinations
}
```

## 3. Proper Parameter Field Extraction

### Summary
Improve parameter extraction to reliably capture all function parameters with proper type handling and support for complex patterns.

### What's Missing
- Current `extract_param_fields` uses fragile heuristics
- Doesn't handle pattern matching in parameters (e.g., destructuring)
- Doesn't properly handle `self`, `&self`, `&mut self`
- Poor handling of generic types and complex type expressions

### How tracing-attributes Does It
- Uses proper `syn` parsing to extract parameter patterns
- Handles all pattern types: `PatIdent`, `PatStruct`, `PatTuple`, etc.
- Determines whether to use `Value` or `Debug` based on parameter type
- Special handling for `self` parameters and async-trait compatibility

### Implementation Guide
**Files to edit:**
- `crustrace-core/src/tracer.rs` - Replace `extract_param_fields` with proper parsing
  - Look at `tracing-attributes/src/expand.rs` lines 482-562 (`param_names` function)
  - Look at `RecordType` enum and its `parse_from_ty` method (lines 425-480)

**Key functions to adapt:**
```rust
// From expand.rs
fn param_names(pat: Pat, record_type: RecordType) -> Box<dyn Iterator<Item = (Ident, RecordType)>> {
    match pat {
        Pat::Ident(PatIdent { ident, .. }) => Box::new(iter::once((ident, record_type))),
        Pat::Reference(PatReference { pat, .. }) => param_names(*pat, record_type),
        Pat::Struct(PatStruct { fields, .. }) => Box::new(
            fields.into_iter()
                .flat_map(|FieldPat { pat, .. }| param_names(*pat, RecordType::Debug)),
        ),
        // ... other patterns
    }
}
```

## 4. Field Customization (`fields` parameter)

### Summary
Add support for `#[instrument(fields(key = value, ...))]` to allow users to add custom fields to spans.

### What's Missing
- No parsing of `fields` parameter
- No support for field expressions with formatting specifiers (`%`, `?`)
- No merging of custom fields with parameter fields
- No support for dotted field names

### How tracing-attributes Does It
- Parses `fields(...)` with custom syntax supporting `%`/`?` prefixes
- Allows expressions as field values
- Merges custom fields with auto-captured parameters
- Handles field name conflicts (custom fields override parameters)

### Implementation Guide
**Files to edit:**
- `crustrace-core/src/tracer.rs` - Add fields parsing and handling
  - Look at `tracing-attributes/src/attr.rs` lines 264-334 (Fields/Field structs)
  - Look at field parsing logic in `Parse` implementations

**Key structures to implement:**
```rust
struct Fields(Punctuated<Field, Token![,]>);
struct Field {
    name: Punctuated<Ident, Token![.]>,
    value: Option<Expr>,
    kind: FieldKind,
}
enum FieldKind { Debug, Display, Value }
```

## 5. Skip Parameter (`skip` parameter)

### Summary
Add support for `#[instrument(skip(param1, param2))]` to exclude specific parameters from being recorded.

### What's Missing
- No parsing of `skip` parameter
- No filtering of skipped parameters from field generation
- No validation that skipped parameters actually exist

### How tracing-attributes Does It
- Parses `skip(...)` as a set of identifiers
- Validates that skipped parameters exist in function signature
- Filters out skipped parameters when generating fields
- Provides clear compile errors for non-existent parameters

### Implementation Guide
**Files to edit:**
- `crustrace-core/src/tracer.rs` - Add skip parsing and filtering
  - Look at `tracing-attributes/src/attr.rs` lines 237-255 (Skips struct)
  - Look at `tracing-attributes/src/expand.rs` lines 164-176 for filtering logic

## 6. Async Function Support

### Summary
Properly instrument async functions using `tracing::Instrument` trait instead of just entering spans.

### What's Missing
- Current implementation treats async functions the same as sync functions
- Doesn't use `tracing::Instrument` for proper async tracing
- No support for async-trait compatibility

### How tracing-attributes Does It
- Detects async functions and async blocks
- Uses `tracing::Instrument::instrument()` to attach span to futures
- Special handling for async-trait patterns
- Handles both native async and manual future implementations

### Implementation Guide
**Files to edit:**
- `crustrace-core/src/tracer.rs` - Add async detection and instrumentation
  - Look at `tracing-attributes/src/expand.rs` lines 276-313 for async handling
  - Look at `AsyncInfo` struct and its methods (lines 565-795)

**Key async pattern:**
```rust
// For async functions
quote!(
    let __tracing_attr_span = #span;
    let __tracing_instrument_future = async move #block;
    tracing::Instrument::instrument(__tracing_instrument_future, __tracing_attr_span).await
)
```

## 7. Additional Span Configuration

### Summary
Add support for remaining span configuration options: `name`, `target`, `parent`, `follows_from`.

### What's Missing
- No support for custom span names via `name = "..."`
- No support for custom targets via `target = "..."`
- No support for parent span relationships
- No support for follows_from relationships

### How tracing-attributes Does It
- Parses all these as part of `InstrumentArgs`
- Uses them when constructing the span
- Supports both string literals and identifiers for some fields
- Handles expressions for `parent` and `follows_from`

### Implementation Guide
**Files to edit:**
- `crustrace-core/src/tracer.rs` - Add parsing for these parameters
  - Look at `tracing-attributes/src/attr.rs` for complete `InstrumentArgs` parsing
  - Look at span construction in `expand.rs` lines 133-157

## 8. Level Configuration

### Summary
Add support for `#[instrument(level = "debug")]` and other level specifications.

### What's Missing
- No parsing of `level` parameter
- Only supports INFO level currently
- No support for string literals, integers (1-5), or Level constant paths

### How tracing-attributes Does It
- Parses level as strings ("trace", "debug", etc.), integers (1-5), or paths
- Defaults to INFO if not specified
- Case-insensitive string matching
- Supports using `Level` constants directly

### Implementation Guide
**Files to edit:**
- `crustrace-core/src/tracer.rs` - Add level parsing
  - Look at `tracing-attributes/src/attr.rs` lines 375-435 (Level enum and parsing)

**Key parsing logic:**
```rust
match str.value() {
    s if s.eq_ignore_ascii_case("trace") => Ok(Level::Trace),
    s if s.eq_ignore_ascii_case("debug") => Ok(Level::Debug),
    // ... etc
}
```
