use crustrace_attributes::instrument;

#[instrument]
fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

#[instrument(level = "debug")]
fn multiply(a: u32, b: u32) -> u32 {
    a * b
}

#[instrument(name = "custom_calculation")]
fn add_numbers(x: i32, y: i32) -> i32 {
    x + y
}

#[allow(dead_code)]
fn main() {
    // Initialize tracing subscriber to see the spans
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_span_events(
            tracing_subscriber::fmt::format::FmtSpan::ENTER
                | tracing_subscriber::fmt::format::FmtSpan::EXIT,
        )
        .init();

    println!("=== Testing crustrace-attributes ===");

    println!("\n1. Basic instrumentation:");
    let result = fibonacci(5);
    println!("fibonacci(5) = {}", result);

    println!("\n2. Custom level (debug):");
    let product = multiply(6, 7);
    println!("multiply(6, 7) = {}", product);

    println!("\n3. Custom span name:");
    let sum = add_numbers(10, 20);
    println!("add_numbers(10, 20) = {}", sum);
}
