use crustrace::instrument;

#[instrument]
pub fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

#[instrument(level = "debug")]
pub fn multiply(a: u32, b: u32) -> u32 {
    a * b
}

#[instrument(name = "custom_calculation")]
pub fn add_numbers(x: i32, y: i32) -> i32 {
    x + y
}

#[instrument(ret)]
pub fn hello(target: &str) -> String {
    format!("Hello {}", target)
}
