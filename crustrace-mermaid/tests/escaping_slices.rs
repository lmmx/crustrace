// tests/escaping_slices.rs
#[cfg(test)]
mod escaping_slices_tests {
    use crustrace::instrument;
    use crustrace_mermaid::*;
    use tracing::subscriber::set_default;
    use tracing_subscriber::prelude::*;

    /// Function that takes references and slices to reproduce
    /// the `&[]` / `&["document"]` style output seen in Mermaid.
    #[instrument]
    fn slice_cases(name: &str, avro_schema: &[&str], record_stack: &[&str]) {
        let _ = (name, avro_schema, record_stack);
    }

    #[test]
    fn snapshot_slice_cases() {
        let layer = MermaidLayer::new().without_auto_flush();
        let subscriber = tracing_subscriber::registry().with(layer.clone());
        let _guard = set_default(subscriber);

        // Case 1: non-empty slice
        slice_cases("document", &["field1", "field2"], &["document"]);

        // Case 2: empty slice
        slice_cases("empty_case", &[], &[]);

        insta::assert_snapshot!(layer.render());
    }
}
