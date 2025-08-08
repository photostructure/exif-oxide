//! Output schemas for generating Rust code

/// Generated tag definition for Rust code
#[derive(Debug)]
#[allow(dead_code)]
pub struct GeneratedTag {
    pub id: u32,
    pub name: String,
    pub format: String,
    pub groups: Vec<String>,
    pub writable: bool,
    pub description: Option<String>,
    pub print_conv_ref: Option<String>,
    pub value_conv_ref: Option<String>,
    pub notes: Option<String>,
}

/// Generated composite tag definition for Rust code
#[derive(Debug)]
#[allow(dead_code)]
pub struct GeneratedCompositeTag {
    pub name: String,
    pub table: String,
    pub require: Vec<String>,
    pub desire: Vec<String>,
    pub print_conv_ref: Option<String>,
    pub value_conv_ref: Option<String>,
    pub description: Option<String>,
    pub writable: bool,
}

/// Format enum that will be generated in Rust code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TagFormat {
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    RationalU,
    RationalS,
    String,
    Undef,
    Float,
    Double,
}

/// Generated tag structure in Rust code
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TagDef {
    pub id: u32,
    pub name: &'static str,
    pub format: TagFormat,
    pub groups: &'static [&'static str],
    pub writable: bool,
    pub description: Option<&'static str>,
    pub print_conv_ref: Option<&'static str>,
    pub value_conv_ref: Option<&'static str>,
    pub notes: Option<&'static str>,
}

/// Generated composite tag structure in Rust code
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CompositeTagDef {
    pub name: &'static str,
    pub table: &'static str,
    pub require: &'static [&'static str],
    pub desire: &'static [&'static str],
    pub print_conv_ref: Option<&'static str>,
    pub value_conv_ref: Option<&'static str>,
    pub description: Option<&'static str>,
    pub writable: bool,
}

/// File type entry for discriminated union
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FileTypeEntry {
    /// Simple alias pointing to another file type
    Alias(String),
    /// Full file type definition
    Definition {
        formats: Vec<String>,
        description: String,
    },
}
