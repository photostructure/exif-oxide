//! Boolean membership testing data sets
//! 
//! This module handles generation of HashSet-based boolean membership tests
//! like weak_magic_types which just need to check if a key exists.

pub mod boolean;

pub use boolean::generate_boolean_set;