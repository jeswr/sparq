//! The `parquet`-feature-gated serialization of the Arrow RDF-term projection.

use arrow_array::RecordBatch;
use arrow_schema::ArrowError as ArrowLibraryError;
use bytes::Bytes;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use parquet::errors::ParquetError;
use sparq_engine::QueryResult;

use crate::{from_record_batch, to_record_batch, ArrowError, ArrowExportError};

// [GPT-5.6] Keep Parquet failures inside the existing export error surface instead of
// adding an enum variant that would break downstream exhaustive matches.
fn export_error(error: ParquetError) -> ArrowExportError {
    ArrowExportError::Arrow(ArrowLibraryError::ParquetError(error.to_string()))
}

fn import_error(error: ParquetError) -> ArrowError {
    ArrowError::invalid(format!("Parquet import failed: {error}"))
}

fn decode_error(error: ArrowLibraryError) -> ArrowError {
    ArrowError::invalid(format!("Parquet Arrow decode failed: {error}"))
}

/// Serialize a SPARQL `SELECT` [`QueryResult`] as Parquet bytes.
///
/// The Parquet file contains the exact Arrow batch produced by [`to_record_batch`]: one
/// nullable five-field RDF-term struct per variable. No Parquet-specific term encoding
/// is introduced, so [`from_parquet_bytes`] is a row-for-row inverse.
///
/// # Errors
///
/// Returns [`ArrowExportError`] if the result cannot be projected to the RDF-term Arrow
/// schema or the Parquet writer rejects or fails to finalize the batch.
pub fn to_parquet_bytes(result: &QueryResult) -> Result<Vec<u8>, ArrowExportError> {
    let batch = to_record_batch(result)?;
    let mut writer =
        ArrowWriter::try_new(Vec::new(), batch.schema(), None).map_err(export_error)?;
    writer.write(&batch).map_err(export_error)?;
    writer.into_inner().map_err(export_error)
}

/// Deserialize Parquet bytes containing this crate's RDF-term Arrow projection.
///
/// Every decoded batch is validated through [`from_record_batch`]. This includes files
/// with no rows: their stored Arrow schema is validated before the reader is consumed,
/// preserving the variable projection for an empty result. Multiple Parquet row groups
/// are concatenated in file order.
///
/// # Errors
///
/// Returns [`ArrowError`] if `bytes` are not a readable Parquet file, their Arrow schema
/// differs from the exact nullable five-field term-struct schema, or any term cell is
/// invalid.
pub fn from_parquet_bytes(bytes: &[u8]) -> Result<QueryResult, ArrowError> {
    let builder = ParquetRecordBatchReaderBuilder::try_new(Bytes::copy_from_slice(bytes))
        .map_err(import_error)?;

    // Validate independently of decoded batches so a zero-row file cannot bypass the
    // schema contract. `new_empty` retains all variable names and struct field metadata.
    let empty = RecordBatch::new_empty(builder.schema().clone());
    let mut result = from_record_batch(&empty)?;

    let reader = builder.build().map_err(import_error)?;
    for batch in reader {
        let decoded = from_record_batch(&batch.map_err(decode_error)?)?;
        if decoded.vars != result.vars {
            return Err(ArrowError::invalid(
                "Parquet row groups decoded with inconsistent SELECT variables",
            ));
        }
        result.rows.extend(decoded.rows);
    }
    Ok(result)
}
