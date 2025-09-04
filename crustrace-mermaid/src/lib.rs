mod visitor;
use visitor::FieldVisitor;

use std::{
    fmt::Write,
    fs::File,
    io::{self, Write as IoWrite},
    path::Path,
    sync::{Arc, Mutex},
};

use tracing::{span, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

/// A node in the call tree (one function span).
#[derive(Debug, Default)]
struct CallNode {
    name: String,
    fields: Vec<(String, String)>,
    children: Vec<Arc<Mutex<CallNode>>>,
}

impl CallNode {
    fn new(name: String) -> Self {
        Self {
            name,
            fields: Vec::new(),
            children: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum GroupingMode {
    /// Each call span gets a unique subgraph (no merging)
    UniquePerCall,
    /// Calls with the same function name share a subgraph
    MergeByName,
}

/// A tracing Layer that collects spans and renders them as a Mermaid flowchart.
#[derive(Clone)]
pub struct MermaidLayer {
    roots: Arc<Mutex<Vec<Arc<Mutex<CallNode>>>>>,
    output: OutputTarget,
    grouping: GroupingMode,
    auto_flush: bool,
}

#[derive(Clone)]
enum OutputTarget {
    Stdout,
    File(Arc<Mutex<File>>),
}

impl Default for MermaidLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl MermaidLayer {
    /// Create a new Mermaid layer that writes to stdout when dropped.
    pub fn new() -> Self {
        Self {
            roots: Arc::new(Mutex::new(Vec::new())),
            output: OutputTarget::Stdout,
            grouping: GroupingMode::MergeByName,
            auto_flush: true,
        }
    }

    pub fn with_mode(mode: GroupingMode) -> Self {
        Self {
            roots: Arc::new(Mutex::new(Vec::new())),
            output: OutputTarget::Stdout,
            grouping: mode,
            auto_flush: true,
        }
    }

    /// Disable auto flush (for manual control).
    pub fn without_auto_flush(mut self) -> Self {
        self.auto_flush = false;
        self
    }

    /// Create a new Mermaid layer that writes to the given file when dropped.
    pub fn new_to_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            roots: Arc::new(Mutex::new(Vec::new())),
            output: OutputTarget::File(Arc::new(Mutex::new(file))),
            grouping: GroupingMode::MergeByName, // default
            auto_flush: true,
        })
    }

    /// Render all collected spans as Mermaid flowchart text.
    pub fn render(&self) -> String {
        let roots = self.roots.lock().unwrap();
        let mut out = String::from("flowchart TD\n");

        // Track all param group IDs
        let mut param_ids = Vec::new();

        let mut counter = 1;
        for root in roots.iter() {
            self.render_node(&mut out, root, &mut param_ids, &mut counter);
        }

        // Styles
        out.push_str(
            r#"
classDef func fill:#c6f6d5,stroke:#2f855a,stroke-width:2px,color:#22543d;
classDef data fill:#bee3f8,stroke:#2b6cb0,stroke-width:1.5px,color:#1a365d;
classDef params fill:none,stroke:#e53e3e,stroke-width:2px,color:#742a2a;
"#,
        );

        if !param_ids.is_empty() {
            out.push_str("class ");
            out.push_str(&param_ids.join(","));
            out.push_str(" params;\n");
        }

        out
    }

    pub fn flush(&self) {
        let mermaid = self.render();
        match &self.output {
            OutputTarget::Stdout => {
                println!("{}", mermaid);
            }
            OutputTarget::File(file) => {
                if let Ok(mut f) = file.lock() {
                    let _ = f.write_all(mermaid.as_bytes());
                }
            }
        }
    }

    fn render_node(
        &self,
        out: &mut String,
        node: &Arc<Mutex<CallNode>>,
        param_ids: &mut Vec<String>,
        counter: &mut usize,
    ) {
        let node = node.lock().unwrap();
        let fn_id = format!("F{}", *counter);
        let params_id = format!("Params{}", *counter);
        *counter += 1;

        // Remember this param capsule for styling later
        param_ids.push(params_id.clone());

        // Emit the param subgraph
        writeln!(out, "subgraph {params_id}[\" \"]").unwrap();
        for (i, (k, v)) in node.fields.iter().enumerate() {
            let data_id = format!("P{}_{}", *counter, i);
            writeln!(out, "  {data_id}[\"{k} = {v}\"]:::data").unwrap();
            if i > 0 {
                writeln!(out, "  P{}_{} --- P{}_{}", *counter, i - 1, *counter, i).unwrap();
            }
        }
        writeln!(out, "end").unwrap();

        // Function node
        writeln!(out, "{fn_id}[\"{}()\"]:::func", node.name).unwrap();
        writeln!(out, "{params_id} --> {fn_id}").unwrap();

        if self.grouping == GroupingMode::MergeByName && !node.children.is_empty() {
            // --- Merge children by function name ---
            use std::collections::BTreeMap;
            let mut groups: BTreeMap<String, Vec<Arc<Mutex<CallNode>>>> = BTreeMap::new();
            for child in &node.children {
                let cname = child.lock().unwrap().name.clone();
                groups.entry(cname).or_default().push(child.clone());
            }

            for (cname, children) in groups {
                let subgraph_id = format!("{}Calls", cname);
                writeln!(out, "subgraph {subgraph_id}[\"{}(...)\"]", cname).unwrap();
                writeln!(out, "  direction TB").unwrap();

                for child in &children {
                    self.render_node(out, child, param_ids, counter);
                }

                writeln!(out, "end").unwrap();

                // Connect parent AFTER subgraph is closed
                for _child in &children {
                    writeln!(out, "  {fn_id} --> Params{}", *counter - 1).unwrap();
                }
            }
        } else {
            // --- UniquePerCall or no grouping ---
            for child in &node.children {
                self.render_node(out, child, param_ids, counter);
                writeln!(out, "  {fn_id} --> Params{}", *counter - 1).unwrap();
            }
        }
    }
}

impl<S> Layer<S> for MermaidLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &span::Id,
        ctx: Context<'_, S>,
    ) {
        let name = ctx
            .metadata(id)
            .map(|m| m.name().to_string())
            .unwrap_or_default();
        let mut node = CallNode::new(name);

        {
            let mut visitor = FieldVisitor {
                fields: &mut node.fields,
            };
            attrs.record(&mut visitor);
        }

        let node = Arc::new(Mutex::new(node));
        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(node);
        }
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(&id) {
            let exts = span.extensions();
            if let Some(node) = exts.get::<Arc<Mutex<CallNode>>>() {
                if let Some(parent) = span.parent() {
                    if let Some(parent_node) = parent.extensions().get::<Arc<Mutex<CallNode>>>() {
                        parent_node.lock().unwrap().children.push(node.clone());
                        return;
                    }
                }

                // no parent → it's a root
                self.roots.lock().unwrap().push(node.clone());

                // auto-flush whenever a root span closes
                if self.auto_flush {
                    self.flush();
                }
            }
        }
    }
}

impl Drop for MermaidLayer {
    fn drop(&mut self) {
        self.flush();
    }
}
