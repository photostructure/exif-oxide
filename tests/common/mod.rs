//! Common test utilities shared across integration tests

use std::path::{Path, PathBuf};

/// Helper to validate JPEG data
pub fn validate_jpeg(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    // Check for JPEG SOI marker (0xFFD8)
    let has_soi = data[0] == 0xFF && data[1] == 0xD8;

    // Check for JPEG EOI marker (0xFFD9) - may not be at the very end due to padding
    // Search for it in the last 32 bytes
    let search_start = if data.len() > 32 { data.len() - 32 } else { 2 };
    let has_eoi = data[search_start..]
        .windows(2)
        .any(|window| window[0] == 0xFF && window[1] == 0xD9);

    has_soi && has_eoi
}

/// Check if a test image exists and return its path, or skip the test
#[macro_export]
macro_rules! test_image_path {
    ($path:expr) => {{
        let path = std::path::Path::new($path);
        if !path.exists() {
            eprintln!("Warning: Test image {} not found, skipping test", $path);
            return;
        }
        path
    }};
}

/// Get the path to a test image in the test-images directory
pub fn get_test_image_path(relative_path: &str) -> PathBuf {
    Path::new("test-images").join(relative_path)
}

/// Get the path to an ExifTool test image
pub fn get_exiftool_test_image_path(relative_path: &str) -> PathBuf {
    Path::new("exiftool/t/images").join(relative_path)
}

/// Common test image paths organized by manufacturer
pub mod test_images {
    pub mod canon {
        pub const T3I_JPG: &str = "test-images/canon/Canon_T3i.JPG";
        pub const T3I_CR2: &str = "test-images/canon/Canon_T3i.CR2";
        pub const R5_MARK_II: &str = "test-images/canon/canon_eos_r5_mark_ii_10.jpg";
    }

    pub mod nikon {
        pub const Z8_JPG: &str = "test-images/nikon/nikon_z8_73.jpg";
    }

    pub mod sony {
        pub const A7C_II: &str = "test-images/sony/sony_a7c_ii_02.jpg";
    }

    pub mod panasonic {
        pub const LUMIX_G9_II: &str = "test-images/panasonic/panasonic_lumix_g9_ii_35.jpg";
    }

    pub mod exiftool {
        pub const EXIFTOOL_JPG: &str = "exiftool/t/images/ExifTool.jpg";
        pub const CANON_JPG: &str = "exiftool/t/images/Canon.jpg";
        pub const CANON_1D_MK_III: &str = "exiftool/t/images/Canon1DmkIII.jpg";
        pub const NIKON_JPG: &str = "exiftool/t/images/Nikon.jpg";
        pub const PNG: &str = "exiftool/t/images/PNG.png";
    }
}

/// Print test result with consistent formatting
pub fn print_test_result(test_name: &str, success: bool, message: &str) {
    let icon = if success { "✓" } else { "✗" };
    println!("{} {}: {}", icon, test_name, message);
}

/// Print info message with consistent formatting
pub fn print_info(test_name: &str, message: &str) {
    println!("ℹ {}: {}", test_name, message);
}

/// Print warning message with consistent formatting
pub fn print_warning(test_name: &str, message: &str) {
    println!("⚠ {}: {}", test_name, message);
}
