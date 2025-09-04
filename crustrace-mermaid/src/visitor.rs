use tracing::field::{Field, Visit};

/// A [`Visit`] implementation that captures span fields as `(key, value)` pairs.
///
/// This visitor is used in [`MermaidLayer::on_new_span`](crate::MermaidLayer)
/// to record parameters from instrumented functions.
///
/// Values are formatted with [`Debug`], but with a small tweak:
/// - If the `Debug` output is a quoted string (e.g. `"foo"`),
///   the surrounding quotes are stripped so the Mermaid diagram
///   shows `foo` rather than `"foo"`.
pub(crate) struct FieldVisitor<'a> {
    /// Mutable reference to the accumulator vector where field entries are pushed.
    pub(crate) fields: &'a mut Vec<(String, String)>,
}

impl<'a> Visit for FieldVisitor<'a> {
    /// Record a field using its [`Debug`] representation.
    ///
    /// Called once per span field by the `tracing` framework.
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        // Format the field value.
        let raw = format!("{:?}", value);

        // Strip surrounding quotes from strings for readability.
        let clean = if raw.starts_with('"') && raw.ends_with('"') {
            raw.trim_matches('"').to_string()
        } else {
            raw
        };

        self.fields.push((field.name().to_string(), clean));
    }
}
