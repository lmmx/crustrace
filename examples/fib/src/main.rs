use crustrace::trace_all;
use tracing_subscriber;

#[trace_all]
mod calculations {
    pub fn fibonacci(n: u64) -> u64 {
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
    
    let result = calculations::fibonacci(5);
    println!("Result: {}", result);
}
