use instrument_fib::*;

fn main() {
    // Initialize tracing subscriber to see the spans
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_span_events(
            tracing_subscriber::fmt::format::FmtSpan::ENTER
                | tracing_subscriber::fmt::format::FmtSpan::EXIT,
        )
        .init();

    println!("=== Testing crustrace ===");

    println!("\n1. Basic instrumentation:");
    let result = fibonacci(5);
    println!("fibonacci(5) = {}", result);

    println!("\n2. Custom level (debug):");
    let product = multiply(6, 7);
    println!("multiply(6, 7) = {}", product);

    println!("\n3. Custom span name:");
    let sum = add_numbers(10, 20);
    println!("add_numbers(10, 20) = {}", sum);

    println!("\n4. Return value:");
    let greeting = hello("world");
    println!(r#"hello("world") = {}"#, greeting);
}
