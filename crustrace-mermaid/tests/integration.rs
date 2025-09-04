#[cfg(test)]
mod tests {
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
    fn snapshot_merge_by_name() {
        let layer = MermaidLayer::with_mode(GroupingMode::MergeByName).without_auto_flush();
        let subscriber = tracing_subscriber::registry().with(layer.clone());

        let _guard = set_default(subscriber); // scoped subscriber
        outer(10, 20);

        insta::assert_snapshot!(layer.render());
    }

    #[test]
    fn snapshot_unique_per_call() {
        let layer = MermaidLayer::with_mode(GroupingMode::UniquePerCall).without_auto_flush();
        let subscriber = tracing_subscriber::registry().with(layer.clone());

        let _guard = set_default(subscriber); // scoped subscriber
        outer(10, 20);

        insta::assert_snapshot!(layer.render());
    }
}
