#[cfg(test)]
mod escaping_tests {
    use crustrace::instrument;
    use crustrace_mermaid::*;
    use tracing::subscriber::set_default;
    use tracing_subscriber::prelude::*;

    /// This function has parameters with problematic values
    /// that will currently break Mermaid until we add escaping.
    #[instrument]
    fn funky(json_object: &str, record_stack: &str) {
        // just a dummy call so spans close
        let _ = (json_object, record_stack);
    }

    #[test]
    fn snapshot_needs_escaping() {
        let layer = MermaidLayer::new().without_auto_flush();
        let subscriber = tracing_subscriber::registry().with(layer.clone());

        let _guard = set_default(subscriber);

        // These strings contain Mermaid-breaking characters
        funky(
            r#"{ "$schema": "https://json-schema.org/draft/2020-12/schema" }"#,
            r#"["document"]"#,
        );

        insta::assert_snapshot!(layer.render());
    }
}
