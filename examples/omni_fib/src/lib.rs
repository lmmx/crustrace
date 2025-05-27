#[crustrace::omni]
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

pub use calc::*;
