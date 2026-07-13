//! The `ipc`-feature-gated Arrow IPC stream serialization of the RDF-term projection.

use arrow_ipc::reader::StreamReader;
use arrow_ipc::writer::StreamWriter;
use arrow_schema::ArrowError as ArrowLibraryError;
use oxrdf::Variable;
use sparq_engine::QueryResult;

use crate::import::variables_from_schema;
use crate::{from_record_batch, to_record_batch, ArrowError, ArrowExportError};

fn import_error(error: ArrowLibraryError) -> ArrowError {
    ArrowError::invalid(format!("Arrow IPC import failed: {error}"))
}

fn decode_error(error: ArrowLibraryError) -> ArrowError {
    ArrowError::invalid(format!("Arrow IPC batch decode failed: {error}"))
}

/// Serialize a SPARQL `SELECT` [`QueryResult`] as an Arrow IPC stream.
///
/// The stream contains the exact batch produced by [`to_record_batch`]: one nullable
/// five-field RDF-term struct per variable. No IPC-specific term encoding is introduced,
/// so [`from_ipc_bytes`] is a row-for-row inverse.
///
/// # Errors
///
/// Returns [`ArrowExportError`] if the result cannot be projected to the RDF-term Arrow
/// schema or the IPC writer rejects or fails to finalize the batch.
pub fn to_ipc_bytes(result: &QueryResult) -> Result<Vec<u8>, ArrowExportError> {
    let batch = to_record_batch(result)?;
    let mut writer = StreamWriter::try_new(Vec::new(), batch.schema().as_ref())?;
    writer.write(&batch)?;
    writer.into_inner().map_err(ArrowExportError::from)
}

/// Read the SPARQL `SELECT` variables from an Arrow IPC stream schema.
///
/// This validates the same variable names and nullable five-field RDF-term struct
/// schema as [`from_ipc_bytes`], but does not decode any record batches.
///
/// # Errors
///
/// Returns [`ArrowError`] if `bytes` do not contain a readable Arrow IPC stream header
/// or its schema violates this crate's RDF-term projection contract.
pub fn ipc_variables_from_bytes(bytes: &[u8]) -> Result<Vec<Variable>, ArrowError> {
    let reader = StreamReader::try_new(bytes, None).map_err(import_error)?;
    variables_from_schema(reader.schema().as_ref())
}

/// Deserialize an Arrow IPC stream containing this crate's RDF-term projection.
///
/// Every decoded batch is validated through [`from_record_batch`]. The stream schema is
/// validated before batches are consumed, preserving the variable projection when the
/// result has no rows. Multiple batches are concatenated in stream order.
///
/// # Errors
///
/// Returns [`ArrowError`] if `bytes` are not a readable Arrow IPC stream, their schema
/// differs from the exact nullable five-field term-struct schema, or any term cell is
/// invalid.
pub fn from_ipc_bytes(bytes: &[u8]) -> Result<QueryResult, ArrowError> {
    let reader = StreamReader::try_new(bytes, None).map_err(import_error)?;

    // Validate the stream header independently of its batches so a zero-row stream
    // cannot bypass the schema contract.
    let mut result = QueryResult {
        vars: variables_from_schema(reader.schema().as_ref())?,
        rows: Vec::new(),
    };

    for batch in reader {
        let decoded = from_record_batch(&batch.map_err(decode_error)?)?;
        if decoded.vars != result.vars {
            return Err(ArrowError::invalid(
                "Arrow IPC batches decoded with inconsistent SELECT variables",
            ));
        }
        result.rows.extend(decoded.rows);
    }
    Ok(result)
}
