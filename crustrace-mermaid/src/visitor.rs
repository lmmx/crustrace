use tracing::field::{Field, Visit};

/// A visitor that collects span fields into a vector of (name, value) pairs.
pub(crate) struct FieldVisitor<'a> {
    pub(crate) fields: &'a mut Vec<(String, String)>,
}

impl<'a> Visit for FieldVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        // By default, Debug on strings gives quotes. Let's strip them if present.
        let raw = format!("{:?}", value);
        let clean = if raw.starts_with('"') && raw.ends_with('"') {
            raw.trim_matches('"').to_string()
        } else {
            raw
        };

        self.fields.push((field.name().to_string(), clean));
    }
}
