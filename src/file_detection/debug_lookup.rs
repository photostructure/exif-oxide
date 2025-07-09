//! Debug utilities for file type lookup

#[cfg(test)]
#[allow(dead_code)]
pub fn debug_file_type_lookup(
    extension: &str,
) -> (
    Option<(Vec<&'static str>, &'static str)>,
    Option<&'static str>,
) {
    use crate::generated::file_types::lookup_mime_types;

    // TODO: Re-enable when resolve_file_type is available
    // use crate::generated::file_types::resolve_file_type;
    // let resolved = resolve_file_type(extension);
    let resolved = None;

    let mime_type = lookup_mime_types(extension);

    (resolved, mime_type)
}
