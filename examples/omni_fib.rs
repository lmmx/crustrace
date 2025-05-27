#!/usr/bin/env rust-script

//! ```cargo
//! [dependencies]
//! crustrace = { path = "../crustrace" }
//! tracing = { version = "0.1", default-features = false }
//! tracing-subscriber = "0.3"
//! ```

use crustrace::omni;

#[omni]
mod calc {
    pub fn fibonacci(n: u64) -> u64 {
        if n <= 1 {
            n
        } else {
            fibonacci(n - 1) + fibonacci(n - 2)
        }
    }

    pub fn multiply(a: u32, b: u32) -> u32 {
        a * b
    }

    pub fn add_numbers(x: i32, y: i32) -> i32 {
        x + y
    }
}

use calc::*;

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

    println!("\n1. fibonacci(5):");
    let result = fibonacci(5);
    println!("fibonacci(5) = {}", result);

    println!("\n2. multiply(6, 7):");
    let product = multiply(6, 7);
    println!("multiply(6, 7) = {}", product);

    println!("\n3. add_numbers(10, 20):");
    let sum = add_numbers(10, 20);
    println!("add_numbers(10, 20) = {}", sum);
}
