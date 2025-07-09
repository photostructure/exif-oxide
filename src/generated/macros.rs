//! Shared macros for generated code
//!
//! This file contains macros used by the code generation system to reduce boilerplate
//! and maintain consistency across all generated lookup tables.

/// Generate a simple lookup table with associated lookup function
///
/// This macro creates:
/// - A static LazyLock HashMap with the provided entries
/// - A lookup function that returns an Option of the value type
///
/// # Example
/// ```rust
/// make_simple_table!(CANON_WHITE_BALANCE, u8, &'static str, [
///     (0, "Auto"),
///     (1, "Daylight"),
///     (2, "Cloudy"),
/// ]);
/// ```
///
/// This generates:
/// - `pub static CANON_WHITE_BALANCE: LazyLock<HashMap<u8, &'static str>>`
/// - `pub fn lookup_canon_white_balance(key: u8) -> Option<&'static str>`
#[macro_export]
macro_rules! make_simple_table {
    ($name:ident, $key_type:ty, $value_type:ty, [$($key:expr => $value:expr),* $(,)?]) => {
        pub static $name: std::sync::LazyLock<std::collections::HashMap<$key_type, $value_type>> =
            std::sync::LazyLock::new(|| {
                let mut map = std::collections::HashMap::new();
                $(
                    map.insert($key, $value);
                )*
                map
            });

        paste::paste! {
            #[doc = concat!("Look up ", stringify!($name), " value by key")]
            pub fn [<lookup_ $name:snake>](key: $key_type) -> Option<$value_type> {
                $name.get(&key).copied()
            }
        }
    };
}

/// Generate a boolean set with membership check function
///
/// This macro creates:
/// - A static LazyLock HashSet with the provided entries
/// - A check function that returns true if the key is in the set
///
/// # Example
/// ```rust
/// make_boolean_set!(WEAK_MAGIC_TYPES, String, [
///     "TXT",
///     "CSV",
///     "JSON",
/// ]);
/// ```
///
/// This generates:
/// - `pub static WEAK_MAGIC_TYPES: LazyLock<HashSet<String>>`
/// - `pub fn is_weak_magic_type(key: &str) -> bool`
#[macro_export]
macro_rules! make_boolean_set {
    ($name:ident, $key_type:ty, [$($value:expr),* $(,)?]) => {
        pub static $name: std::sync::LazyLock<std::collections::HashSet<$key_type>> =
            std::sync::LazyLock::new(|| {
                let mut set = std::collections::HashSet::new();
                $(
                    set.insert($value.into());
                )*
                set
            });

        paste::paste! {
            #[doc = concat!("Check if key is in ", stringify!($name))]
            pub fn [<is_ $name:snake>](key: &str) -> bool {
                $name.contains(key)
            }
        }
    };
}

/// Generate a regex pattern table
///
/// This macro creates a static LazyLock HashMap of compiled regex patterns
///
/// # Example
/// ```rust
/// make_regex_table!(MAGIC_NUMBER_PATTERNS, [
///     ("JPEG", r"^\xff\xd8\xff"),
///     ("PNG", r"^\x89PNG\r\n\x1a\n"),
/// ]);
/// ```
#[macro_export]
macro_rules! make_regex_table {
    ($name:ident, [$($key:expr => $pattern:expr),* $(,)?]) => {
        pub static $name: std::sync::LazyLock<std::collections::HashMap<&'static str, regex::Regex>> =
            std::sync::LazyLock::new(|| {
                let mut map = std::collections::HashMap::new();
                $(
                    map.insert($key, regex::Regex::new($pattern).expect(concat!("Invalid regex for ", $key)));
                )*
                map
            });

        paste::paste! {
            #[doc = concat!("Get regex pattern for ", stringify!($name))]
            pub fn [<get_ $name:snake>](key: &str) -> Option<&'static regex::Regex> {
                $name.get(key)
            }
        }
    };
}
