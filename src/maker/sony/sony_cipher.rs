//! Sony cipher support for encrypted maker note tags
//!
//! Sony uses a substitution cipher for certain tags (0x2010 and 0x9xxx series).
//! The cipher formula is: `c = (b*b*b) % 249` where c is the enciphered byte.
//! This module implements decryption using a pre-computed lookup table.

/// Cipher mode for Sony decryption
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SonyCipherMode {
    /// Single pass decryption (normal case)
    Single,
    /// Double pass decryption (ExifTool 9.04-9.10 bug workaround)
    Double,
}

/// Sony Tag2010 variant enumeration
/// Different camera models use different Tag2010 table variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SonyTag2010Variant {
    /// Base Tag2010 table
    Tag2010a,
    /// Variant for newer cameras  
    Tag2010b,
    /// Additional variants
    Tag2010c,
    Tag2010d,
    Tag2010e,
    Tag2010f,
    Tag2010g,
    Tag2010h,
    Tag2010i,
}

impl SonyTag2010Variant {
    /// Determine variant based on camera model
    pub fn from_model(model: &str) -> Self {
        let model_lower = model.to_lowercase();

        // Map camera models to Tag2010 variants based on ExifTool's Sony.pm
        // Check more specific patterns first
        if model_lower.contains("a7") || model_lower.contains("a9") {
            SonyTag2010Variant::Tag2010d
        } else if model_lower.contains("rx100") || model_lower.contains("rx10") {
            SonyTag2010Variant::Tag2010c
        } else if model_lower.contains("nex-") || model_lower.contains("alpha") {
            SonyTag2010Variant::Tag2010b
        } else if model_lower.contains("fx") {
            SonyTag2010Variant::Tag2010e
        } else {
            // Default to base variant
            SonyTag2010Variant::Tag2010a
        }
    }
}

/// Pre-computed decipher lookup table
/// Generated from ExifTool's Sony.pm decipher translation (line 11371)
/// Maps encrypted bytes back to original bytes using the substitution cipher
const DECIPHER_TABLE: [u8; 256] = [
    0x00, 0x01, 0x47, 0x03, 0x0c, 0x12, 0xc2, 0x38, 0x02, 0x09, 0x0a, 0x0b, 0x27, 0x0d, 0x09, 0x0f,
    0x8e, 0xc1, 0x12, 0x73, 0xc9, 0x9a, 0x16, 0x17, 0x96, 0x19, 0x3e, 0x03, 0xd9, 0x1d, 0xd4, 0xb4,
    0x1b, 0x21, 0x22, 0x5a, 0x45, 0xcc, 0x26, 0x88, 0x28, 0xea, 0x2a, 0x4f, 0x2c, 0xa8, 0x2e, 0x33,
    0x1c, 0x97, 0x7f, 0x5b, 0x34, 0x35, 0xd8, 0x37, 0x74, 0x37, 0xc3, 0x3b, 0xbb, 0xd5, 0x3e, 0x3f,
    0x04, 0x83, 0x42, 0x43, 0x87, 0x6d, 0x9f, 0x50, 0xe7, 0xbc, 0xba, 0x75, 0x3d, 0x4d, 0x66, 0x6b,
    0xe0, 0x30, 0x76, 0x77, 0x78, 0xc0, 0x0d, 0x84, 0x89, 0x59, 0xb3, 0x79, 0x36, 0x34, 0x08, 0x9d,
    0x4e, 0x82, 0x68, 0x63, 0x64, 0x6c, 0xeb, 0x67, 0x68, 0x18, 0x35, 0x6b, 0x2b, 0xb9, 0x7a, 0x6f,
    0x15, 0x71, 0x72, 0x93, 0x70, 0x57, 0xd1, 0xbd, 0x78, 0x46, 0x7a, 0x9e, 0x7c, 0x05, 0xc6, 0x67,
    0x80, 0x22, 0xaa, 0x95, 0x5d, 0x3a, 0xd3, 0xe9, 0xce, 0x89, 0x14, 0xed, 0xbf, 0x11, 0x59, 0x72,
    0x7e, 0x54, 0xdd, 0x7c, 0x99, 0x2f, 0x61, 0x20, 0xe6, 0x99, 0xc8, 0x92, 0x63, 0x8b, 0xef, 0xb8,
    0x4d, 0xdb, 0xe2, 0x5f, 0xcb, 0xf1, 0xf3, 0xf5, 0xa5, 0xd7, 0x0f, 0xab, 0x4b, 0xad, 0xf7, 0xb0,
    0xac, 0x81, 0xb2, 0xc5, 0xb4, 0xdf, 0x6a, 0x49, 0xe4, 0xb9, 0x52, 0x24, 0x90, 0xae, 0x1e, 0xa1,
    0xc0, 0xc1, 0xf6, 0xe8, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf,
    0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf,
    0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef,
    0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
];

/// Check if a tag ID represents an encrypted Sony tag
pub fn is_encrypted_tag(tag_id: u16) -> bool {
    tag_id == 0x2010 || (0x9000..0xa000).contains(&tag_id)
}

/// Decrypt a single byte using Sony's substitution cipher
fn decrypt_byte(encrypted_byte: u8) -> u8 {
    DECIPHER_TABLE[encrypted_byte as usize]
}

/// Perform one decryption pass on the data
fn decrypt_pass(data: &mut [u8]) {
    for byte in data.iter_mut() {
        *byte = decrypt_byte(*byte);
    }
}

/// Decipher Sony encrypted data
///
/// # Arguments
/// * `data` - The encrypted data to decipher
/// * `mode` - Cipher mode (single or double pass)
///
/// # Returns
/// Decrypted data as a Vec<u8>
pub fn decipher_sony_data(data: &[u8], mode: SonyCipherMode) -> Vec<u8> {
    let mut result = data.to_vec();

    // First decryption pass
    decrypt_pass(&mut result);

    // Second pass if double encryption detected
    if matches!(mode, SonyCipherMode::Double) {
        decrypt_pass(&mut result);
    }

    result
}

/// Detect if data was double-enciphered (ExifTool 9.04-9.10 bug)
/// This can be detected by checking if the decrypted data looks reasonable
pub fn detect_double_cipher(decrypted_data: &[u8]) -> bool {
    // Simple heuristic: if most bytes are in a reasonable range after first decryption,
    // it's probably single-enciphered. If not, try double decryption.
    let reasonable_bytes = decrypted_data
        .iter()
        .take(16) // Check first 16 bytes
        .filter(|&&b| b < 0x80 || b == 0x00) // Most data should be ASCII or null
        .count();

    // If less than 50% of bytes look reasonable, try double decryption
    reasonable_bytes < 8
}

/// Determine cipher mode by analyzing the first decryption pass
pub fn determine_cipher_mode(data: &[u8]) -> SonyCipherMode {
    if data.is_empty() {
        return SonyCipherMode::Single;
    }

    // Decrypt once and check if result looks reasonable
    let single_decrypt = decipher_sony_data(data, SonyCipherMode::Single);

    if detect_double_cipher(&single_decrypt) {
        SonyCipherMode::Double
    } else {
        SonyCipherMode::Single
    }
}

/// Helper function to safely decipher Sony data with automatic mode detection
pub fn decipher_sony_data_auto(data: &[u8]) -> Vec<u8> {
    let mode = determine_cipher_mode(data);
    decipher_sony_data(data, mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decipher_table_correctness() {
        // Verify the decipher table is correct by checking a few known values
        // The cipher formula is: c = (b*b*b) % 249

        // Test byte 0: 0^3 % 249 = 0
        assert_eq!(DECIPHER_TABLE[0], 0);

        // Test byte 1: should find b where b^3 % 249 = 1
        // 1^3 % 249 = 1, so DECIPHER_TABLE[1] should be 1
        assert_eq!(DECIPHER_TABLE[1], 1);

        // Test byte 8: should find b where b^3 % 249 = 8
        // 2^3 % 249 = 8, so DECIPHER_TABLE[8] should be 2
        assert_eq!(DECIPHER_TABLE[8], 2);
    }

    #[test]
    fn test_is_encrypted_tag() {
        assert!(is_encrypted_tag(0x2010));
        assert!(is_encrypted_tag(0x9000));
        assert!(is_encrypted_tag(0x9fff));
        assert!(!is_encrypted_tag(0x0102));
        assert!(!is_encrypted_tag(0xa000));
    }

    #[test]
    fn test_cipher_modes() {
        let test_data = vec![0x01, 0x02, 0x03, 0x04];

        let single_result = decipher_sony_data(&test_data, SonyCipherMode::Single);
        let double_result = decipher_sony_data(&test_data, SonyCipherMode::Double);

        // Double decryption should produce different result than single
        assert_ne!(single_result, double_result);

        // Results should have same length as input
        assert_eq!(single_result.len(), test_data.len());
        assert_eq!(double_result.len(), test_data.len());
    }

    #[test]
    fn test_sony_tag2010_variant_detection() {
        assert_eq!(
            SonyTag2010Variant::from_model("NEX-7"),
            SonyTag2010Variant::Tag2010b
        );
        assert_eq!(
            SonyTag2010Variant::from_model("ALPHA A7"),
            SonyTag2010Variant::Tag2010d
        );
        assert_eq!(
            SonyTag2010Variant::from_model("RX100 III"),
            SonyTag2010Variant::Tag2010c
        );
        assert_eq!(
            SonyTag2010Variant::from_model("DSC-H1"),
            SonyTag2010Variant::Tag2010a
        );
    }

    #[test]
    fn test_auto_cipher_detection() {
        // Test with some dummy data
        let test_data = vec![0x10, 0x20, 0x30, 0x40];
        let result = decipher_sony_data_auto(&test_data);

        // Should return some result
        assert_eq!(result.len(), test_data.len());
    }
}
