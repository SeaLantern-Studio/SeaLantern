//! Portable ZIP archive infrastructure.
//!
//! Archives are written from directory contents and extracted through validated
//! relative paths. Symbolic links are deliberately rejected during extraction:
//! creating them has incompatible permissions and semantics across supported
//! platforms, and callers must opt into a dedicated policy before doing so.

mod error;
mod symbol_link;
mod unzip;
mod zipper;

pub use error::ArchiveError;
pub use symbol_link::{is_symbolic_link, parse_symbolic_link_target};
pub use unzip::{extract_zip, ExtractionSummary};
pub use zipper::{create_zip, ArchiveSummary};
