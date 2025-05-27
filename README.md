# Crustrace

<!-- [![CodeCov Status](https://codecov.io/gh/lmmx/crustrace/graph/badge.svg?token=UCFLM1MD9D)](https://codecov.io/gh/lmmx/crustrace) -->
<!-- [![crates.io](https://img.shields.io/crates/v/crustrace.svg)](https://crates.io/crates/crustrace) -->
<!-- [![documentation](https://docs.rs/crustrace/badge.svg)](https://docs.rs/crustrace) -->
<!-- [![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/lmmx/crustrace/binaries.yml)](https://github.com/lmmx/crustrace/actions/workflows/binaries.yml) -->
[![free of syn](https://img.shields.io/badge/free%20of-syn-hotpink)](https://github.com/fasterthanlime/free-of-syn)
[![MIT/Apache-2.0 licensed](https://img.shields.io/crates/l/crustrace.svg)](./LICENSE)
[![pre-commit.ci status](https://results.pre-commit.ci/badge/github/lmmx/crustrace/master.svg)](https://results.pre-commit.ci/latest/github/lmmx/crustrace/master)

Crustrace is a procedural macro that automatically instruments all functions in a module with `tracing` spans, eliminating the need to manually add `#[instrument]` to every function.

Use Crustrace when you want comprehensive tracing with minimal effort to add and remove.

Stick with manual instrumentation when you need fine-grained control over which functions are traced.

## Motivation

When adding distributed tracing to Rust applications, developers typically need to annotate every function they want to trace:

```rust
#[tracing::instrument(level = "info", ret)]
fn foo() { ... }

#[tracing::instrument(level = "info", ret)]
fn bar() { ... }

#[tracing::instrument(level = "info", ret)]
fn baz() { ... }
```

This is tedious and a barrier to quick instrumentation of anything more than a function or two (we really want module and crate-level instrumentation).

Crustrace solves this by automatically instrumenting all functions in a module, giving you complete call-chain tracing with minimal code changes.
It's a simple initial solution but would be extensible to filter the functions it's applied to by
name, by crate in a workspace, and so on.

## Installation

Add Crustrace to your `Cargo.toml`:

```toml
[dependencies]
crustrace = "0.1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Usage

### Basic Usage

Apply the `#[trace_all]` attribute to any module:

```rust
use crustrace::trace_all;

#[trace_all]
mod my_functions {
    fn foo(x: i32) -> i32 {
        bar(x * 2)
    }
    
    fn bar(y: i32) -> i32 {
        baz(y + 1)
    }
    
    fn baz(z: i32) -> i32 {
        z * 3
    }
}
```

or more typically, by putting `#![trace_all]` (note the `!`) at the top of a module not declared by a `mod` block.

All functions in the module are then automatically instrumented as if you had written:

```rust
#[tracing::instrument(level = "info", ret)]
fn foo(x: i32) -> i32 { ... }

#[tracing::instrument(level = "info", ret)]  
fn bar(y: i32) -> i32 { ... }

#[tracing::instrument(level = "info", ret)]
fn baz(z: i32) -> i32 { ... }
```

**WORK IN PROGRESS**

`crustrace::instrument` is a `syn`-free (simpler, yet functional!) version
of the `tracing-attributes::instrument` macro.

In turn, `crustrace::trace_all` no longer uses `tracing::instrument`, it is
entirely using `crustrace::instrument`.

### Complete Example

```rust
use crustrace::trace_all;
use tracing_subscriber;

#[trace_all]
mod calculations {
    fn fibonacci(n: u64) -> u64 {
        if n <= 1 {
            n
        } else {
            add_numbers(fibonacci(n - 1), fibonacci(n - 2))
        }
    }
    
    fn add_numbers(a: u64, b: u64) -> u64 {
        a + b
    }
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ENTER | 
                          tracing_subscriber::fmt::format::FmtSpan::EXIT)
        .init();
    
    use calculations::*;
    let result = fibonacci(5);
    println!("Result: {}", result);
}
```

This produces detailed tracing output showing the complete call hierarchy:

```
INFO fibonacci{n=5}: enter
  INFO fibonacci{n=4}: enter  
    INFO fibonacci{n=3}: enter
      INFO fibonacci{n=2}: enter
        INFO fibonacci{n=1}: enter
        INFO fibonacci{n=1}: return=1
        INFO fibonacci{n=1}: exit
        INFO fibonacci{n=0}: enter
        INFO fibonacci{n=0}: return=0  
        INFO fibonacci{n=0}: exit
        INFO add_numbers{a=1 b=0}: enter
        INFO add_numbers{a=1 b=0}: return=1
        INFO add_numbers{a=1 b=0}: exit
      INFO fibonacci{n=2}: return=1
      INFO fibonacci{n=2}: exit
      // ... and so on
```

## How It Works

Crustrace uses a procedural macro to parse the token stream of a module and automatically inject `#[tracing::instrument(level = "info", ret)]` before every function definition.

It uses `proc-macro2` (it's [free of `syn`](https://github.com/fasterthanlime/free-of-syn)!) to
parse tokens rather than doing string replacement or full on AST creation.

### Implementation Details

The macro:

1. **Parses tokens** rather than doing string replacement to avoid false positives
2. **Validates function definitions** by ensuring `fn` is followed by an identifier
3. **Preserves existing attributes** and doesn't interfere with other procedural macros
4. **Handles edge cases** like generic functions, async functions, and various formatting styles

### What Gets Instrumented

- Regular functions: `fn foo() { ... }`
- Generic functions: `fn foo<T>(x: T) { ... }`
- Functions with complex signatures: `fn foo(x: impl Display) -> Result<String, Error> { ... }`

### What Doesn't Get Instrumented

- Function calls within expressions: `some_fn_call()`
- String literals containing "fn": `"fn not a function"`
- Comments: `// fn something`
- Already instrumented functions (won't double-instrument)

## Configuration

By default, Crustrace applies these instrument settings:

- **Level**: `info`
- **Return values**: Logged (`ret`)
- **Parameters**: All function parameters are automatically captured

Future versions may support customising these settings via macro parameters (please feel free to
suggest ideas and submit at least some test for it if you can't figure out how it'd be implemented).
PRs would be ideal!

## Performance Considerations

### Tracing Overhead

- Instrumented functions have minimal overhead when tracing is disabled
- When tracing is enabled, overhead is proportional to the number of function calls
- It might be wise to consider using `RUST_LOG` environment variable to control tracing levels in future

### Compilation Impact

- Crustrace processes modules at compile time with minimal impact on build performance
- The generated code is equivalent to manually writing `#[instrument]` attributes
- No runtime dependencies beyond the standard `tracing` crate

### Ideas

- Depth limits
- Specified crates/function patterns to trace
- Environment variable enabled tracing
- More nuanced control of traced events, log level, etc.

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
