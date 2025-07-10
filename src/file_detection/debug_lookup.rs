//! Debug utilities for file type lookup

#[cfg(test)]
#[allow(dead_code)]
pub fn debug_file_type_lookup(
    extension: &str,
) -> (
    Option<(Vec<&'static str>, &'static str)>,
    Option<&'static str>,
) {
    use crate::generated::ExifTool_pm::lookup_mime_types;

    use crate::generated::file_types::resolve_file_type;
    let resolved = resolve_file_type(extension);

    let mime_type = lookup_mime_types(extension);

    (resolved, mime_type)
}
