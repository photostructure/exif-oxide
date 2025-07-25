//! Runtime evaluation system for subdirectory conditions
//!
//! This module provides runtime evaluation capabilities for complex subdirectory
//! conditions that cannot be handled at code generation time. It extends the
//! existing expression evaluation system with subdirectory-specific logic.
//!
//! ## Key Features
//!
//! - **$valPt binary pattern matching**: Evaluates binary data patterns from ExifTool
//! - **$self{Make}/$self{Model} context matching**: Model and manufacturer matching
//! - **Complex condition evaluation**: AND/OR logic, count conditions, format checks
//! - **Integration with tag kit dispatch**: Seamless integration with generated subdirectory processors
//!
//! ## ExifTool Reference
//!
//! ExifTool uses various runtime condition patterns in SubDirectory definitions:
//! ```perl
//! {
//!     Condition => '$$valPt =~ /^0204/',
//!     SubDirectory => { ProcessProc => \&ProcessNikonEncrypted }
//! },
//! {
//!     Condition => '$$self{Model} =~ /EOS R5/',
//!     SubDirectory => { ProcessProc => \&ProcessCanonSerialDataMkII }
//! }
//! ```

pub mod condition_evaluator;
pub mod enhanced_processors;
pub mod integration;

// Re-export core types
pub use condition_evaluator::{SubdirectoryConditionEvaluator, SubdirectoryContext};
pub use enhanced_processors::{enhanced_canon_subdirectory_processor, wrap_existing_processor};
pub use integration::{
    create_subdirectory_context_from_exif, RuntimeSubdirectoryDispatcher,
    RuntimeSubdirectoryProcessor,
};
