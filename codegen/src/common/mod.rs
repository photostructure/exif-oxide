//! Common utilities for code generation

pub mod utils;

pub use utils::{
    escape_regex_for_rust,
    escape_rust_string,
    escape_string,
    find_repo_root,
    normalize_format,
    parse_hex_id,
};