#[cfg(test)]
mod param_keys_tests {
    use crustrace::instrument;
    use crustrace_mermaid::*;
    use tracing::subscriber::set_default;
    use tracing_subscriber::prelude::*;

    #[instrument]
    fn inner(x: i32, y: i32) -> i32 {
        x + y
    }

    #[instrument]
    fn outer(a: i32, b: i32) -> i32 {
        let r1 = inner(a + 1, b / 10);
        let r2 = inner(a * 2, b / 20);
        r1 + r2
    }

    #[test]
    fn snapshot_single_node() {
        let layer = MermaidLayer::new()
            .with_params_mode(ParamRenderMode::SingleNode)
            .without_auto_flush();
        let subscriber = tracing_subscriber::registry().with(layer.clone());
        let _guard = set_default(subscriber);

        outer(10, 20);

        insta::assert_snapshot!(layer.render());
    }

    #[test]
    fn snapshot_single_node_grouped() {
        let layer = MermaidLayer::new()
            .with_params_mode(ParamRenderMode::SingleNodeGrouped)
            .without_auto_flush();
        let subscriber = tracing_subscriber::registry().with(layer.clone());
        let _guard = set_default(subscriber);

        outer(10, 20);

        insta::assert_snapshot!(layer.render());
    }
}
