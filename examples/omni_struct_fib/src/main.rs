use omni_fib_struct::*;

fn main() {
    // Initialize tracing subscriber to see the spans
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_span_events(
            tracing_subscriber::fmt::format::FmtSpan::ENTER
                | tracing_subscriber::fmt::format::FmtSpan::EXIT,
        )
        .init();

    println!("=== Testing crustrace::instrument ===");

    let calc = Calculator;

    println!("\n1. fibonacci(5):");
    let result = calc.fibonacci(5);
    println!("fibonacci(5) = {}", result);

    println!("\n2. multiply(6, 7):");
    let product = calc.multiply(6, 7);
    println!("multiply(6, 7) = {}", product);

    println!("\n3. add_numbers(10, 20):");
    let sum = calc.add_numbers(10, 20);
    println!("add_numbers(10, 20) = {}", sum);
}
