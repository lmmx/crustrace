//! # crustrace-mermaid
//!
//! A [`tracing_subscriber::Layer`] implementation that collects spans
//! and renders them as [Mermaid flowcharts](https://mermaid.js.org/syntax/flowchart.html).
//!
//! This crate is designed to pair with [`crustrace`](https://crates.io/crates/crustrace),
//! which automatically instruments entire modules. Together, you can
//! generate call graphs of your Rust program without hand-writing `#[instrument]`.
//!
//! ## Example
//!
//! ```rust
//! use crustrace::instrument;
//! use crustrace_mermaid::{MermaidLayer, GroupingMode};
//! use tracing_subscriber::prelude::*;
//!
//! #[instrument]
//! fn inner(x: i32) -> i32 { x + 1 }
//!
//! #[instrument]
//! fn outer(y: i32) -> i32 {
//!     inner(y) + inner(y * 2)
//! }
//!
//! fn main() {
//!     let layer = MermaidLayer::with_mode(GroupingMode::MergeByName);
//!     tracing_subscriber::registry().with(layer).init();
//!
//!     outer(5);
//!     // When the root span closes, a Mermaid diagram is printed to stdout
//! }
//! ```
//!
//! By default, the layer flushes automatically when a root span finishes,
//! but you can disable this with [`MermaidLayer::without_auto_flush`] and
//! call [`MermaidLayer::flush`] manually.
//!
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

/// How to group multiple calls in the rendered Mermaid output.
#[derive(Clone, Copy, PartialEq)]
pub enum GroupingMode {
    /// Each call span gets its own subgraph.
    ///
    /// This preserves call uniqueness at the cost of potentially repetitive diagrams.
    UniquePerCall,
    /// Calls with the same function name are grouped into a single subgraph.
    ///
    /// This produces a more compact view where repeated calls appear together.
    MergeByName,
}

/// A [`tracing_subscriber::Layer`] that collects function spans
/// and renders them as a Mermaid flowchart.
///
/// This is the main entry point of the crate.
#[derive(Clone)]
pub struct MermaidLayer {
    roots: Arc<Mutex<Vec<Arc<Mutex<CallNode>>>>>,
    output: OutputTarget,
    grouping: GroupingMode,
    auto_flush: bool,
}

/// Where to write rendered Mermaid diagrams.
///
/// - `Stdout`: print to stdout
/// - `File`: write to a file handle (locked by `Arc<Mutex<...>>`)
///
/// This is set at layer construction time by [`MermaidLayer::new`] or
/// [`MermaidLayer::new_to_file`].
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

const MERMAID_STYLES: &str = r#"
classDef func fill:#c6f6d5,stroke:#2f855a,stroke-width:2px,color:#22543d;
classDef data fill:#bee3f8,stroke:#2b6cb0,stroke-width:1.5px,color:#1a365d;
classDef params fill:none,stroke:#e53e3e,stroke-width:2px,color:#742a2a;
"#;

impl MermaidLayer {
    /// Create a new layer that writes to stdout when dropped.
    ///
    /// By default, uses [`GroupingMode::MergeByName`] and enables auto-flush.
    pub fn new() -> Self {
        Self {
            roots: Arc::new(Mutex::new(Vec::new())),
            output: OutputTarget::Stdout,
            grouping: GroupingMode::MergeByName,
            auto_flush: true,
        }
    }

    /// Like [`MermaidLayer::new`] but lets you choose the [`GroupingMode`].
    pub fn with_mode(mode: GroupingMode) -> Self {
        Self {
            roots: Arc::new(Mutex::new(Vec::new())),
            output: OutputTarget::Stdout,
            grouping: mode,
            auto_flush: true,
        }
    }

    /// Disable automatic flushing on root span close.
    ///
    /// Useful if you want to render once at the very end of your program or test,
    /// instead of printing multiple partial diagrams.
    pub fn without_auto_flush(mut self) -> Self {
        self.auto_flush = false;
        self
    }

    /// Create a new layer that writes diagrams to a given file when dropped.
    ///
    /// The file is overwritten each time.
    pub fn new_to_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            roots: Arc::new(Mutex::new(Vec::new())),
            output: OutputTarget::File(Arc::new(Mutex::new(file))),
            grouping: GroupingMode::MergeByName, // default
            auto_flush: true,
        })
    }

    /// Render all collected spans into Mermaid flowchart text.
    ///
    /// This does not print anything; see [`MermaidLayer::flush`] if you want to
    /// send the result to stdout or to the configured file.
    pub fn render(&self) -> String {
        let roots = self.roots.lock().unwrap();
        let mut out = String::from("flowchart TD\n");

        // Track all param group IDs
        let mut param_ids = Vec::new();

        let mut counter = 1;
        for root in roots.iter() {
            self.render_node(&mut out, root, &mut param_ids, &mut counter);
        }

        out.push_str(MERMAID_STYLES);

        if !param_ids.is_empty() {
            out.push_str("class ");
            out.push_str(&param_ids.join(","));
            out.push_str(" params;\n");
        }

        out
    }

    /// Render all children of a function node, respecting the current grouping mode.
    ///
    /// - In [`GroupingMode::MergeByName`], children with the same function name are grouped
    ///   into a shared subgraph like `innerCalls`.
    /// - In [`GroupingMode::UniquePerCall`], each child is rendered independently.
    fn render_children(
        &self,
        out: &mut String,
        parent_fn_id: &str,
        children: &[Arc<Mutex<CallNode>>],
        param_ids: &mut Vec<String>,
        counter: &mut usize,
    ) {
        if self.grouping == GroupingMode::MergeByName && !children.is_empty() {
            use std::collections::BTreeMap;

            // Group children by function name
            let mut groups: BTreeMap<String, Vec<Arc<Mutex<CallNode>>>> = BTreeMap::new();
            for child in children {
                let cname = child.lock().unwrap().name.clone();
                groups.entry(cname).or_default().push(child.clone());
            }

            // Emit each group as a subgraph
            for (cname, group) in groups {
                let subgraph_id = format!("{}Calls", cname);
                writeln!(out, "subgraph {subgraph_id}[\"{}(...)\"]", cname).unwrap();
                writeln!(out, "  direction TB").unwrap();

                for child in &group {
                    self.render_node(out, child, param_ids, counter);
                }

                writeln!(out, "end").unwrap();

                // Connect parent → all children after closing the subgraph
                for _child in &group {
                    writeln!(out, "  {parent_fn_id} --> Params{}", *counter - 1).unwrap();
                }
            }
        } else {
            // UniquePerCall or no grouping: render each child inline
            for child in children {
                self.render_node(out, child, param_ids, counter);
                writeln!(out, "  {parent_fn_id} --> Params{}", *counter - 1).unwrap();
            }
        }
    }

    /// Render and write the current diagram to the configured output
    /// (`stdout` or file).
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

    /// Recursively render a single call node and its children.
    ///
    /// Each node produces:
    /// - A **parameter subgraph** (red capsule) containing all field values
    /// - A **function node** (green box)
    /// - Edges from parameters → function, and function → child parameters
    ///
    /// The `counter` is used to generate stable unique IDs across the whole tree.
    /// `param_ids` collects parameter group IDs so they can all be styled later.
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

        // Children (delegate to helper)
        self.render_children(out, &fn_id, &node.children, param_ids, counter);
    }
}

impl<S> Layer<S> for MermaidLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    /// Called when a new span is created.
    ///
    /// Here we allocate a new `CallNode` with the span name and any recorded fields,
    /// and attach it into the span’s `extensions` so it can later be retrieved in
    /// `on_close`.
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

    /// Called when a span closes.
    ///
    /// If the span has a parent, we attach the node into its parent’s children.
    /// Otherwise, the span is considered a **root** and added to `self.roots`.
    ///
    /// When `auto_flush` is enabled, closing a root span will immediately flush
    /// a complete Mermaid diagram (useful for short-lived programs).
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
        if self.auto_flush {
            self.flush();
        }
    }
}
