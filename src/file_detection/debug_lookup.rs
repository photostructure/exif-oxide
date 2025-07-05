//! Debug utilities for file type lookup

#[cfg(test)]
#[allow(dead_code)]
pub fn debug_file_type_lookup(
    extension: &str,
) -> (Option<(Vec<String>, String)>, Option<&'static str>) {
    use crate::generated::file_types::{lookup_mime_types, resolve_file_type};

    let resolved = resolve_file_type(extension);
    let mime_type = if let Some((ref formats, _)) = resolved {
        // Try the first format for MIME lookup
        formats.first().and_then(|f| lookup_mime_types(f))
    } else {
        lookup_mime_types(extension)
    };

    (resolved, mime_type)
}
