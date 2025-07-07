//! Debug test to understand file type mappings

#[cfg(test)]
mod tests {
    use crate::generated::simple_tables::file_types::lookup_mime_types;
    // TODO: Re-enable when resolve_file_type is available
    // use crate::generated::simple_tables::file_types::resolve_file_type;

    #[test]
    fn debug_problematic_formats() {
        let problem_formats = ["webp", "avi", "mkv", "3gp", "m2ts"];

        for ext in problem_formats {
            println!("\n=== Extension: {ext} ===");

            // TODO: Re-enable when resolve_file_type is available
            // if let Some((formats, description)) = resolve_file_type(&ext.to_uppercase()) {
            if false {
                let formats: Vec<&str> = vec![];
                let description = "";
                println!("Resolves to formats: {formats:?}");
                println!("Description: {description}");

                for format in &formats {
                    if let Some(mime) = lookup_mime_types(format) {
                        println!("Format '{format}' has MIME: {mime}");
                    } else {
                        println!("Format '{format}' has NO MIME type mapping");
                    }
                }
            } else {
                println!("No resolution found for extension '{ext}'");

                // Try direct MIME lookup
                if let Some(mime) = lookup_mime_types(&ext.to_uppercase()) {
                    println!("Direct MIME lookup found: {mime}");
                }
            }
        }
    }

    #[test]
    fn list_all_mime_types() {
        use crate::generated::simple_tables::file_types::mime_types::MIME_TYPES;

        println!("\n=== All Available MIME Types ===");
        let mut entries: Vec<_> = MIME_TYPES.iter().collect();
        entries.sort();

        for (file_type, mime) in entries {
            println!("{file_type} -> {mime}");
        }
    }
}
