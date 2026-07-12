// [FABLE-5] sq-lsp7k.1.1 (forms F1): single-source the crate overview from README.md
// so crates.io and the docs.rs front page render identical content; the README's
// `## Usage` rust fence compiles as a doctest.
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod derive;
pub mod description;
pub mod diff;
pub mod widgets;

pub use description::{
    Constraints, FormDescription, FormField, FormGroup, FormValue, GroupKind, Mode, ShapeChoice,
    ShapeVia, TermRef, WidgetChoice,
};
pub use diff::{to_sparql_update, FieldValueDiff, FormDiff};
pub use widgets::{Resolution, Scorer, Suitability, WidgetContext, WidgetRegistry, DASH};

use oxrdf::Term;
use sparq_core::Graph;
use sparq_shacl::view::GraphView;
use sparq_shacl::ShapesModel;

/// Options for a form derivation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormOptions {
    /// View (read-only) or edit (the default) — see [`Mode`].
    pub mode: Mode,
    /// Opaque role echoed into the description (`dash:propertyRole` filtering
    /// lands with F5, sq-lsp7k.1.5).
    pub role: Option<String>,
    /// Explicit node-shape choice: derive against THIS shape (it need not
    /// target the focus node) instead of the first applicable one.
    pub shape: Option<Term>,
    /// Maximum `sh:node` sub-form nesting depth (default 3); deeper values
    /// render without a nested form (the renderer can derive on demand).
    pub max_depth: usize,
}

impl Default for FormOptions {
    fn default() -> Self {
        FormOptions {
            mode: Mode::Edit,
            role: None,
            shape: None,
            max_depth: 3,
        }
    }
}

/// Derives the [`FormDescription`] for `focus` from a data graph and a SHACL
/// shapes graph, using the default DASH widget registry.
///
/// Parses the shapes graph on every call — use [`derive_form_with_model`] to
/// amortise shape parsing (and/or supply a custom [`WidgetRegistry`]) across
/// many focus nodes.
pub fn derive_form(
    data: &Graph,
    shapes: &Graph,
    focus: &Term,
    opts: &FormOptions,
) -> FormDescription {
    let model = ShapesModel::parse(shapes);
    derive_form_with_model(data, shapes, &model, focus, opts, &WidgetRegistry::dash())
}

/// [`derive_form`] against an already-parsed [`ShapesModel`] and an explicit
/// [`WidgetRegistry`]. `shapes` must be the graph `model` was parsed from: the
/// model carries the constraint components while the graph supplies the
/// presentation vocabulary (`sh:name`/`sh:description`/`sh:order`/`sh:group`,
/// `dash:editor`/`dash:viewer`/`dash:singleLine`/`dash:applicableToClass`).
pub fn derive_form_with_model(
    data: &Graph,
    shapes: &Graph,
    model: &ShapesModel,
    focus: &Term,
    opts: &FormOptions,
    registry: &WidgetRegistry,
) -> FormDescription {
    derive::derive(
        &GraphView::new(data),
        &GraphView::new(shapes),
        model,
        focus,
        opts,
        registry,
    )
}

/// The node shapes applicable to `focus` — the renderer's shape switcher:
/// `sh:targetNode`, `sh:targetClass` / implicit class targets (matched against
/// the focus node's `rdf:type`s with `rdfs:subClassOf` closure in the data
/// graph), and `dash:applicableToClass`.
pub fn applicable_shapes(
    data: &Graph,
    shapes: &Graph,
    model: &ShapesModel,
    focus: &Term,
) -> Vec<ShapeChoice> {
    derive::applicable(&GraphView::new(data), &GraphView::new(shapes), model, focus)
}
