#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! crustrace = { path = "../crustrace" }
//! crustrace-mermaid = { path = "../crustrace-mermaid" }
//! tracing = { version = "0.1", default-features = false }
//! tracing-subscriber = "0.3"
//! ```

use crustrace_mermaid::MermaidLayer;
use tracing_subscriber::prelude::*;

#[crustrace::instrument]
fn inner(x: i32, y: i32) -> i32 {
    x * y
}

#[crustrace::instrument]
fn outer(a: i32, b: i32) -> i32 {
    let va = inner(a, 2);
    let vb = inner(b, 3);
    va + vb
}

fn main() {
    // Attach the Mermaid layer; on drop it prints a flowchart
    tracing_subscriber::registry()
        .with(MermaidLayer::new())
        .init();

    let _result = outer(10, 20);
    // println!("Result = {result}");
}
