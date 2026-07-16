//! The `csv`-feature-gated CSV serialization of the flattened RDF-term projection.
//!
//! CSV has no nested types (arrow-csv rejects `Struct` columns), so each SELECT
//! variable's five-field term struct is flattened to five `Utf8` columns named
//! `var.kind` / `var.value` / `var.datatype` / `var.language` / `var.direction`.
//! CSV also has no value-level null — the writer encodes null as the empty field and
//! the reader yields null for an empty field — so boundness is carried by the `kind`
//! column: an unbound cell is five empty fields, while a bound empty-string literal
//! keeps `kind=literal` plus its explicit `xsd:string` datatype. Since a bound term's
//! `kind` is never empty and a valid datatype IRI / language tag / direction is never
//! the empty string, this flattening loses nothing the term struct can express.
//! [FABLE-5]

use std::sync::Arc;

use arrow_array::builder::StringBuilder;
use arrow_array::{Array, ArrayRef, RecordBatch, StringArray, StructArray};
use arrow_buffer::NullBuffer;
use arrow_csv::reader::Format;
use arrow_csv::{ReaderBuilder, WriterBuilder};
use arrow_schema::{ArrowError as ArrowLibraryError, DataType, Field, Schema};
use oxrdf::Variable;
use sparq_engine::QueryResult;

use crate::export::term_fields;
use crate::import::variables_from_schema;
use crate::{
    from_record_batch, term_struct_type, to_record_batch, ArrowError, ArrowExportError,
    FIELD_KIND, FIELD_VALUE, RDF_TERM_FIELDS,
};

fn import_error(error: ArrowLibraryError) -> ArrowError {
    ArrowError::invalid(format!("Arrow CSV import failed: {}", error))
}

fn decode_error(error: ArrowLibraryError) -> ArrowError {
    ArrowError::invalid(format!("Arrow CSV batch decode failed: {}", error))
}

/// The flattened CSV column name for one variable's term-struct field. A SPARQL
/// variable name can never contain `.` (the `VARNAME` production excludes it, and
/// `oxrdf::Variable` enforces that), so the `var.field` spelling is unambiguous.
fn flat_column_name(variable: &str, field: &str) -> String {
    format!("{}.{}", variable, field)
}

/// Serialize a SPARQL `SELECT` [`QueryResult`] as CSV bytes (header row included).
///
/// The CSV contains the exact batch produced by [`to_record_batch`], flattened: CSV has
/// no nested types, so each variable's five-field RDF-term struct becomes the five
/// `Utf8` columns `var.kind` / `var.value` / `var.datatype` / `var.language` /
/// `var.direction`. No CSV-specific term encoding is introduced beyond that flattening,
/// so [`from_csv_bytes`] is a row-for-row inverse.
///
/// An unbound cell is five empty fields. Boundness is carried by the `kind` column, so
/// an unbound cell stays distinct from a bound empty-string literal (`kind=literal`
/// with the explicit `xsd:string` datatype).
///
/// # Errors
///
/// Returns [`ArrowExportError`] if the result cannot be projected to the RDF-term Arrow
/// schema, has zero variables (CSV cannot represent zero-column rows, unlike the IPC
/// and Parquet containers), or the CSV writer rejects the batch.
pub fn to_csv_bytes(result: &QueryResult) -> Result<Vec<u8>, ArrowExportError> {
    let batch = to_record_batch(result)?;
    if batch.num_columns() == 0 {
        return Err(ArrowExportError::Arrow(ArrowLibraryError::CsvError(
            "a zero-variable SELECT result cannot be represented as CSV rows".to_string(),
        )));
    }
    let flat = flatten(&batch)?;
    let mut writer = WriterBuilder::new().with_header(true).build(Vec::new());
    writer.write(&flat)?;
    Ok(writer.into_inner())
}

// Flatten each variable's term-struct column into its five Utf8 children.
// `to_record_batch` already writes null children for unbound cells, but the
// struct-level validity is still merged into each child defensively so an unbound slot
// can never leak a child value into the CSV. [FABLE-5]
fn flatten(batch: &RecordBatch) -> Result<RecordBatch, ArrowLibraryError> {
    let schema = batch.schema();
    let width = schema.fields().len() * RDF_TERM_FIELDS.len();
    let mut fields = Vec::with_capacity(width);
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(width);
    for (index, field) in schema.fields().iter().enumerate() {
        let column = batch
            .column(index)
            .as_any()
            .downcast_ref::<StructArray>()
            .ok_or_else(|| {
                ArrowLibraryError::CsvError(format!(
                    "Arrow field '{}' is not backed by a StructArray",
                    field.name()
                ))
            })?;
        for (child_index, child_name) in RDF_TERM_FIELDS.iter().enumerate() {
            let child = column
                .column(child_index)
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    ArrowLibraryError::CsvError(format!(
                        "Arrow field '{}.{}' is not backed by a UTF-8 array",
                        field.name(),
                        child_name
                    ))
                })?;
            // Null wherever either the child or the enclosing struct slot is null.
            let nulls = NullBuffer::union(column.nulls(), child.nulls());
            let merged =
                StringArray::try_new(child.offsets().clone(), child.values().clone(), nulls)?;
            fields.push(Field::new(
                flat_column_name(field.name(), child_name),
                DataType::Utf8,
                true,
            ));
            columns.push(Arc::new(merged) as ArrayRef);
        }
    }
    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
}

/// Read the SPARQL `SELECT` variables from a CSV header row.
///
/// This validates the same flattened five-columns-per-variable header contract as
/// [`from_csv_bytes`] — and routes the reconstructed variables through the shared
/// RDF-term schema validation — but does not decode any data rows.
///
/// # Errors
///
/// Returns [`ArrowError`] if `bytes` do not start with a readable CSV header row or the
/// header does not follow the `var.kind` … `var.direction` flattened projection.
pub fn csv_variables_from_bytes(bytes: &[u8]) -> Result<Vec<Variable>, ArrowError> {
    let (schema, _) = Format::default()
        .with_header(true)
        .infer_schema(bytes, Some(0))
        .map_err(import_error)?;
    let names: Vec<&str> = schema
        .fields()
        .iter()
        .map(|field| field.name().as_str())
        .collect();
    if names.is_empty() || !names.len().is_multiple_of(RDF_TERM_FIELDS.len()) {
        return Err(ArrowError::invalid(format!(
            "CSV header has {} columns; the flattened RDF-term projection needs a non-zero multiple of {}",
            names.len(),
            RDF_TERM_FIELDS.len()
        )));
    }

    let kind_suffix = format!(".{}", FIELD_KIND);
    let mut fields = Vec::with_capacity(names.len() / RDF_TERM_FIELDS.len());
    for group in names.chunks(RDF_TERM_FIELDS.len()) {
        let variable = group[0].strip_suffix(&kind_suffix).unwrap_or("");
        if variable.is_empty() {
            return Err(ArrowError::invalid(format!(
                "CSV column '{}' does not name a variable's '{}' field",
                group[0], FIELD_KIND
            )));
        }
        for (name, field) in group.iter().zip(RDF_TERM_FIELDS) {
            let expected = flat_column_name(variable, field);
            if *name != expected {
                return Err(ArrowError::invalid(format!(
                    "CSV column '{}' does not match the expected flattened column '{}'",
                    name, expected
                )));
            }
        }
        fields.push(Field::new(variable, term_struct_type(), true));
    }
    // One schema-validation path: route the reconstructed struct schema through the
    // same validator as the RecordBatch / IPC / Parquet readers so the CSV header
    // reader cannot drift from the full row decoder.
    variables_from_schema(&Schema::new(fields))
}

/// Deserialize CSV bytes containing the flattened RDF-term projection.
///
/// The header row is validated before any data row is decoded, preserving the variable
/// projection when the result has no rows (the IPC/Parquet empty-schema precedent).
/// Every decoded batch is unflattened back into the term-struct RecordBatch and
/// validated through [`from_record_batch`]; multiple reader batches are concatenated in
/// row order. All batches share the single header-derived schema, so — unlike a
/// multi-batch IPC stream — their variables cannot disagree.
///
/// CSV cannot distinguish an empty field from an absent one: on a row whose `kind`
/// field is non-empty (a bound term), an empty `value` field reads back as the empty
/// string (the bound empty-string-literal case), while a row whose `kind` field is
/// empty is an unbound cell and must have all five fields empty.
///
/// # Errors
///
/// Returns [`ArrowError`] if `bytes` are not readable CSV, the header does not follow
/// the flattened five-columns-per-variable projection, or any term cell is invalid.
pub fn from_csv_bytes(bytes: &[u8]) -> Result<QueryResult, ArrowError> {
    // Validate the header independently of the data rows so a zero-row file cannot
    // bypass the schema contract.
    let vars = csv_variables_from_bytes(bytes)?;

    let read_fields: Vec<Field> = vars
        .iter()
        .flat_map(|variable| {
            RDF_TERM_FIELDS.iter().map(move |field| {
                Field::new(
                    flat_column_name(variable.as_str(), field),
                    DataType::Utf8,
                    true,
                )
            })
        })
        .collect();
    let reader = ReaderBuilder::new(Arc::new(Schema::new(read_fields)))
        .with_header(true)
        .build(bytes)
        .map_err(import_error)?;

    let mut result = QueryResult {
        vars,
        rows: Vec::new(),
    };
    for batch in reader {
        let decoded = from_record_batch(&unflatten(&batch.map_err(decode_error)?, &result.vars)?)?;
        result.rows.extend(decoded.rows);
    }
    Ok(result)
}

// Rebuild each variable's term struct from its five flattened Utf8 columns. Boundness
// is decided by `kind` (never empty for a bound term): a bound row's empty `value`
// field is the empty string, while an empty `kind` marks an unbound row. Non-`value`
// fields keep any non-empty content even on unbound rows, so `from_record_batch` still
// rejects a malformed "unbound slot carrying data" instead of silently dropping it.
// [FABLE-5]
fn unflatten(flat: &RecordBatch, vars: &[Variable]) -> Result<RecordBatch, ArrowError> {
    let row_count = flat.num_rows();
    let mut fields = Vec::with_capacity(vars.len());
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(vars.len());

    for (var_index, variable) in vars.iter().enumerate() {
        let mut raw_children = Vec::with_capacity(RDF_TERM_FIELDS.len());
        for (child_index, child_name) in RDF_TERM_FIELDS.iter().enumerate() {
            raw_children.push(
                flat.column(var_index * RDF_TERM_FIELDS.len() + child_index)
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .ok_or_else(|| {
                        ArrowError::invalid(format!(
                            "CSV column '{}.{}' did not decode as a UTF-8 array",
                            variable.as_str(),
                            child_name
                        ))
                    })?,
            );
        }

        let mut builders: Vec<StringBuilder> = (0..RDF_TERM_FIELDS.len())
            .map(|_| StringBuilder::with_capacity(row_count, row_count * 8))
            .collect();
        let mut valid = Vec::with_capacity(row_count);

        for row in 0..row_count {
            let bound = present(raw_children[0], row).is_some();
            valid.push(bound);
            for ((child, builder), field_name) in raw_children
                .iter()
                .zip(builders.iter_mut())
                .zip(RDF_TERM_FIELDS)
            {
                match present(child, row) {
                    Some(text) => builder.append_value(text),
                    // A bound row's empty `value` field is the bound empty-string
                    // literal, not an absent component.
                    None if bound && field_name == FIELD_VALUE => builder.append_value(""),
                    None => builder.append_null(),
                }
            }
        }

        let children: Vec<ArrayRef> = builders
            .into_iter()
            .map(|mut builder| Arc::new(builder.finish()) as ArrayRef)
            .collect();
        let column = StructArray::try_new(term_fields(), children, Some(NullBuffer::from(valid)))
            .map_err(decode_error)?;
        fields.push(Field::new(variable.as_str(), term_struct_type(), true));
        columns.push(Arc::new(column) as ArrayRef);
    }

    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns).map_err(decode_error)
}

// A field is present only when non-null AND non-empty: the CSV writer encodes null as
// the empty field and the reader yields null for an empty field, so both spellings of
// absence are normalized here.
fn present(array: &StringArray, row: usize) -> Option<&str> {
    array
        .is_valid(row)
        .then(|| array.value(row))
        .filter(|text| !text.is_empty())
}
