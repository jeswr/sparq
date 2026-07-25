//! The `arrow`-feature-gated export: `QueryResult` → Arrow `RecordBatch`.
//!
//! The whole module compiles only with `--features arrow`; the default build of the
//! crate carries none of this (and no Arrow dependency). The mapping it implements is
//! documented at the crate root.

use std::sync::Arc;

use arrow_array::builder::StringBuilder;
use arrow_array::{ArrayRef, RecordBatch, StructArray};
use arrow_buffer::NullBuffer;
use arrow_schema::{ArrowError, DataType, Field, Fields, Schema};
use oxrdf::{Term, Variable};
use sparq_engine::QueryResult;

use crate::{FIELD_DATATYPE, FIELD_DIRECTION, FIELD_KIND, FIELD_LANGUAGE, FIELD_VALUE};

/// Failure converting a [`QueryResult`] to Arrow or Parquet output.
///
/// The only failure modes are an invalid SELECT projection (a duplicate variable name,
/// which would make two Arrow fields collide) and an Arrow-side construction error
/// (e.g. a column length mismatch — should not happen for a well-formed result, but is
/// surfaced rather than panicked). Constructed honestly so the caller can `?` it.
#[derive(Debug)]
pub enum ArrowExportError {
    /// Two SELECT variables share a name, so they cannot be distinct Arrow columns.
    DuplicateVariable(String),
    /// The Arrow or Parquet library rejected the assembled output.
    Arrow(ArrowError),
}

impl std::fmt::Display for ArrowExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Positional args (not inline `{var}`) to avoid the CodeQL
            // rust/unused-variable false positive flagged in the SPARQ contract.
            ArrowExportError::DuplicateVariable(v) => {
                write!(
                    f,
                    "duplicate SELECT variable '{}': cannot map to two Arrow columns",
                    v
                )
            }
            ArrowExportError::Arrow(e) => write!(f, "Arrow/Parquet export failed: {}", e),
        }
    }
}

impl std::error::Error for ArrowExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ArrowExportError::Arrow(e) => Some(e),
            ArrowExportError::DuplicateVariable(_) => None,
        }
    }
}

impl From<ArrowError> for ArrowExportError {
    fn from(e: ArrowError) -> Self {
        ArrowExportError::Arrow(e)
    }
}

/// The Arrow `Struct` data type each SELECT variable is projected to: a struct of the
/// five nullable `Utf8` fields documented at the crate root (`kind`, `value`, `datatype`,
/// `language`, `direction`). Every field is nullable: `value` is non-null for any bound
/// term but the struct itself is nulled out for an unbound cell, so marking the leaf
/// fields nullable keeps Arrow's validity bookkeeping consistent.
pub fn term_struct_type() -> DataType {
    DataType::Struct(term_fields())
}

fn term_fields() -> Fields {
    Fields::from(vec![
        Field::new(FIELD_KIND, DataType::Utf8, true),
        Field::new(FIELD_VALUE, DataType::Utf8, true),
        Field::new(FIELD_DATATYPE, DataType::Utf8, true),
        Field::new(FIELD_LANGUAGE, DataType::Utf8, true),
        Field::new(FIELD_DIRECTION, DataType::Utf8, true),
    ])
}

/// The Arrow [`Schema`] a [`QueryResult`] with these `vars` projects to: one nullable
/// column per variable, named after the variable, of [`term_struct_type`]. Useful to
/// build the schema without materialising any rows (e.g. for a streaming/empty batch).
///
/// Errors if two variables share a name (they would be indistinguishable columns).
pub fn term_schema(vars: &[Variable]) -> Result<Schema, ArrowExportError> {
    let mut seen = std::collections::HashSet::with_capacity(vars.len());
    let mut fields = Vec::with_capacity(vars.len());
    for v in vars {
        if !seen.insert(v.as_str()) {
            return Err(ArrowExportError::DuplicateVariable(v.as_str().to_string()));
        }
        fields.push(Field::new(v.as_str(), term_struct_type(), true));
    }
    Ok(Schema::new(fields))
}

/// Per-variable accumulator: the five child string builders plus the per-row validity
/// for the enclosing struct (true = bound, false = the cell was unbound).
struct ColumnBuilder {
    kind: StringBuilder,
    value: StringBuilder,
    datatype: StringBuilder,
    language: StringBuilder,
    direction: StringBuilder,
    valid: Vec<bool>,
}

impl ColumnBuilder {
    fn with_capacity(rows: usize) -> Self {
        ColumnBuilder {
            kind: StringBuilder::with_capacity(rows, rows * 8),
            value: StringBuilder::with_capacity(rows, rows * 16),
            datatype: StringBuilder::with_capacity(rows, rows * 16),
            language: StringBuilder::with_capacity(rows, rows * 4),
            direction: StringBuilder::with_capacity(rows, rows * 4),
            valid: Vec::with_capacity(rows),
        }
    }

    /// Append an unbound cell: every leaf is null and the struct slot is invalid.
    fn push_unbound(&mut self) {
        self.kind.append_null();
        self.value.append_null();
        self.datatype.append_null();
        self.language.append_null();
        self.direction.append_null();
        self.valid.push(false);
    }

    /// Append a bound term, decomposed into the five fields per the crate-root mapping.
    fn push_term(&mut self, term: &Term) {
        match term {
            Term::NamedNode(n) => {
                self.kind.append_value("uri");
                self.value.append_value(n.as_str());
                self.datatype.append_null();
                self.language.append_null();
                self.direction.append_null();
            }
            Term::BlankNode(b) => {
                self.kind.append_value("bnode");
                self.value.append_value(b.as_str());
                self.datatype.append_null();
                self.language.append_null();
                self.direction.append_null();
            }
            Term::Literal(l) => {
                self.kind.append_value("literal");
                self.value.append_value(l.value());
                if let Some(lang) = l.language() {
                    // Language-tagged: the language tag, no datatype (its datatype is the
                    // rdf:langString/dirLangString machinery, implied by the tag's presence).
                    self.datatype.append_null();
                    self.language.append_value(lang);
                    match l.direction() {
                        Some(oxrdf::BaseDirection::Ltr) => self.direction.append_value("ltr"),
                        Some(oxrdf::BaseDirection::Rtl) => self.direction.append_value("rtl"),
                        None => self.direction.append_null(),
                    }
                } else {
                    // Typed (or plain) literal: carry the datatype IRI explicitly,
                    // including xsd:string (we do NOT elide it — see the crate docs).
                    self.datatype.append_value(l.datatype().as_str());
                    self.language.append_null();
                    self.direction.append_null();
                }
            }
            Term::Triple(_) => {
                // RDF 1.2 triple term: stringify to N-Triples in `value`. oxrdf's
                // Display for Term emits the N-Triples form `<< s p o >>`.
                self.kind.append_value("triple");
                self.value.append_value(term.to_string());
                self.datatype.append_null();
                self.language.append_null();
                self.direction.append_null();
            }
        }
        self.valid.push(true);
    }

    /// Finish into a `StructArray` (one column), with the per-row struct validity applied.
    fn finish(mut self) -> Result<ArrayRef, ArrowError> {
        let children: Vec<ArrayRef> = vec![
            Arc::new(self.kind.finish()),
            Arc::new(self.value.finish()),
            Arc::new(self.datatype.finish()),
            Arc::new(self.language.finish()),
            Arc::new(self.direction.finish()),
        ];
        let nulls = NullBuffer::from(self.valid);
        let arr = StructArray::try_new(term_fields(), children, Some(nulls))?;
        Ok(Arc::new(arr))
    }
}

/// Convert a SPARQL `SELECT` [`QueryResult`] into an Arrow [`RecordBatch`].
///
/// The batch has one column per SELECT variable (named after the variable), each of the
/// [`term_struct_type`] struct type; an unbound binding is a `null` struct slot. The
/// mapping from RDF terms to the struct's fields is documented at the crate root. The
/// batch has exactly `result.rows.len()` rows.
///
/// # Errors
/// Returns [`ArrowExportError::DuplicateVariable`] if two SELECT variables share a name,
/// or [`ArrowExportError::Arrow`] if the Arrow library rejects the assembled batch.
///
/// # Example
/// ```
/// # use sparq_engine::QueryResult;
/// # use oxrdf::{NamedNode, Term, Variable};
/// let result = QueryResult {
///     vars: vec![Variable::new("s").unwrap()],
///     rows: vec![vec![Some(Term::NamedNode(NamedNode::new("http://ex/a").unwrap()))]],
/// };
/// let batch = sparq_arrow::to_record_batch(&result).unwrap();
/// assert_eq!(batch.num_columns(), 1);
/// assert_eq!(batch.num_rows(), 1);
/// assert_eq!(batch.schema().field(0).name(), "s");
/// ```
pub fn to_record_batch(result: &QueryResult) -> Result<RecordBatch, ArrowExportError> {
    let schema = term_schema(&result.vars)?;
    let nrows = result.rows.len();

    let mut builders: Vec<ColumnBuilder> = (0..result.vars.len())
        .map(|_| ColumnBuilder::with_capacity(nrows))
        .collect();

    for row in &result.rows {
        for (ci, builder) in builders.iter_mut().enumerate() {
            // A row is one Option<Term> per variable position; a row shorter than `vars`
            // (should not happen for a well-formed result) reads as unbound, never panics.
            match row.get(ci).and_then(|c| c.as_ref()) {
                Some(term) => builder.push_term(term),
                None => builder.push_unbound(),
            }
        }
    }

    let columns: Vec<ArrayRef> = builders
        .into_iter()
        .map(ColumnBuilder::finish)
        .collect::<Result<_, _>>()?;

    // try_new validates column count/length against the schema; an empty-var result still
    // yields a valid 0-column batch carrying the row count via options.
    if columns.is_empty() {
        let opts = arrow_array::RecordBatchOptions::new().with_row_count(Some(nrows));
        return Ok(RecordBatch::try_new_with_options(
            Arc::new(schema),
            columns,
            &opts,
        )?);
    }
    Ok(RecordBatch::try_new(Arc::new(schema), columns)?)
}
