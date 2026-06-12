//! XPath `xs:` namespace type constructors and casts
//!
//! This module provides a deterministic mapping from XPath type constructor names in the `xs:`
//! namespace to their Noir implementations. The naming follows patterns for type conversions:
//!
//! - XPath `xs:float(integer)` -> `xpath_xs::float_from_integer`
//! - XPath `xs:double(integer)` -> `xpath_xs::double_from_integer`
//! - XPath `xs:integer(float)` -> `xpath_xs::integer_from_float`
//!
//! This enables code generators to compose function names programmatically for type casts.
//!
//! ## Usage
//!
//! ```noir
//! use xpath::xpath_xs;
//!
//! // xs:float(integer_value)
//! let f = xpath_xs::float_from_integer(42);
//!
//! // xs:integer(double_value)
//! let i = xpath_xs::integer_from_double(d);
//! ```

// ============================================================================
// Type constructors - Integer to Float/Double
// ============================================================================

// xs:float(integer) - cast integer to float
pub use crate::numeric_types::cast_integer_to_float as float_from_integer;

// xs:double(integer) - cast integer to double
pub use crate::numeric_types::cast_integer_to_double as double_from_integer;

// ============================================================================
// Type constructors - Float/Double to Integer
// ============================================================================

// xs:integer(float) - cast float to integer (truncation)
pub use crate::numeric_types::cast_float_to_integer as integer_from_float;

// xs:integer(double) - cast double to integer (truncation)
pub use crate::numeric_types::cast_double_to_integer as integer_from_double;

// ============================================================================
// Type constructors - Float/Double conversions
// ============================================================================

// xs:float(double) - cast double to float (narrowing)
pub use crate::numeric_types::cast_double_to_float as float_from_double;
