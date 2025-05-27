mod tracer;
pub use tracer::instrument_impl;
mod omnibus;
pub use omnibus::trace_all_impl;
