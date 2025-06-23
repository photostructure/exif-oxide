//! XMP data types and structures

use std::collections::HashMap;

/// XMP packet containing standard and extended data
#[derive(Debug, Clone)]
pub struct XmpPacket {
    /// Main XMP packet data (raw XML)
    pub standard: Vec<u8>,

    /// Extended XMP data if present
    pub extended: Option<ExtendedXmp>,
}

impl XmpPacket {
    /// Create a new XMP packet with standard data only
    pub fn new(standard: Vec<u8>) -> Self {
        Self {
            standard,
            extended: None,
        }
    }

    /// Get the standard packet as a string
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.standard)
    }
}

/// Extended XMP information for packets >64KB
#[derive(Debug, Clone)]
pub struct ExtendedXmp {
    /// GUID linking to main packet
    pub guid: String,

    /// Total size of extended data
    pub total_length: u32,

    /// MD5 hash for validation
    pub md5: Option<[u8; 16]>,

    /// Reassembled extended data
    pub data: Vec<u8>,
}

/// Hierarchical XMP value types
#[derive(Debug, Clone, PartialEq)]
pub enum XmpValue {
    /// Simple text value
    Simple(String),

    /// Array of values
    Array(XmpArray),

    /// Structured value (nested properties)
    Struct(HashMap<String, XmpValue>),
}

impl XmpValue {
    /// Get as simple string if possible
    pub fn as_str(&self) -> Option<&str> {
        match self {
            XmpValue::Simple(s) => Some(s),
            _ => None,
        }
    }

    /// Get as array if possible
    pub fn as_array(&self) -> Option<&XmpArray> {
        match self {
            XmpValue::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Get as struct if possible
    pub fn as_struct(&self) -> Option<&HashMap<String, XmpValue>> {
        match self {
            XmpValue::Struct(s) => Some(s),
            _ => None,
        }
    }
}

/// XMP array types corresponding to RDF containers
#[derive(Debug, Clone, PartialEq)]
pub enum XmpArray {
    /// Ordered array (rdf:Seq)
    Ordered(Vec<XmpValue>),

    /// Unordered array (rdf:Bag)
    Unordered(Vec<XmpValue>),

    /// Alternative array (rdf:Alt) - typically for language alternatives
    Alternative(Vec<LanguageAlternative>),
}

impl XmpArray {
    /// Get the values as a slice regardless of array type
    pub fn values(&self) -> Vec<&XmpValue> {
        match self {
            XmpArray::Ordered(v) | XmpArray::Unordered(v) => v.iter().collect(),
            XmpArray::Alternative(alts) => alts.iter().map(|a| &a.value).collect(),
        }
    }
}

/// Language alternative for rdf:Alt arrays
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageAlternative {
    /// Language code (e.g., "en-US", "x-default")
    pub lang: String,

    /// The value in this language
    pub value: XmpValue,
}

/// Parsed XMP metadata with namespace support
#[derive(Debug, Clone)]
pub struct XmpMetadata {
    /// Properties organized by namespace
    pub properties: HashMap<String, HashMap<String, XmpValue>>,

    /// Namespace prefix to URI mapping
    pub namespaces: HashMap<String, String>,
}

impl Default for XmpMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl XmpMetadata {
    /// Create empty XMP metadata
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            namespaces: HashMap::new(),
        }
    }

    /// Get a property by namespace and name
    pub fn get(&self, namespace: &str, name: &str) -> Option<&XmpValue> {
        self.properties.get(namespace)?.get(name)
    }

    /// Get all properties in a namespace
    pub fn get_namespace(&self, namespace: &str) -> Option<&HashMap<String, XmpValue>> {
        self.properties.get(namespace)
    }
}
