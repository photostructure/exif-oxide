//! Nikon encryption key management and ProcessNikonEncrypted foundation
//!
//! **Trust ExifTool**: This code translates ExifTool's Nikon encryption system verbatim.
//!
//! ExifTool Reference: lib/Image/ExifTool/Nikon.pm ProcessNikonEncrypted function
//!
//! Nikon's encryption system uses multiple keys derived from camera metadata:
//! - Serial number (tag 0x001d) as primary encryption key
//! - Shutter count (tag 0x00a7) as secondary encryption key
//! - Model-specific decryption algorithms
//!
//! Phase 1 Implementation: Key management and detection only
//! Phase 2 Implementation: Encrypted section detection and cataloging
//! Phase 3+ Implementation: Actual decryption algorithms (future milestone)

use crate::exif::ExifReader;
use crate::generated::nikon_pm::{XLAT_0, XLAT_1};
use crate::tiff_types::{ByteOrder, IfdEntry};
use crate::types::{Result, TagValue};
use tracing::{debug, trace, warn};

// XOR decryption lookup tables now imported from generated code
// Generated from ExifTool Nikon.pm @xlat arrays (lines 13505-13538)
// See: src/generated/nikon_pm/xlat_0.rs and xlat_1.rs

/// Nikon decryption state management
/// ExifTool: $ci0, $cj0, $ck0, $decryptStart variables (lines 13540, 13566-13568)
#[derive(Debug, Clone)]
pub struct NikonDecryptionState {
    pub ci0: u8,
    pub cj0: u8,
    pub ck0: u8,
    pub decrypt_start: Option<usize>,
}

/// Nikon encryption key management system
/// ExifTool: Nikon.pm encryption key storage and validation
#[derive(Debug, Clone)]
pub struct NikonEncryptionKeys {
    /// Camera serial number for encryption (tag 0x001d)
    /// ExifTool: $$et{NikonSerialKey} = $val
    pub serial_number: Option<String>,

    /// Shutter count for encryption (tag 0x00a7)  
    /// ExifTool: $$et{NikonCountKey} = $val
    pub shutter_count: Option<u32>,

    /// Camera model for algorithm selection
    /// ExifTool: Model-specific decryption handling
    pub camera_model: String,

    /// Additional encryption parameters (future use)
    /// ExifTool: Various model-specific encryption parameters
    pub additional_params: std::collections::HashMap<String, String>,

    /// Decryption state for continuous processing
    /// ExifTool: Global variables $ci0, $cj0, $ck0, $decryptStart
    pub decryption_state: Option<NikonDecryptionState>,
}

impl NikonEncryptionKeys {
    /// Create new encryption key manager for a Nikon camera
    /// ExifTool: Initialize encryption context for camera model
    pub fn new(model: String) -> Self {
        debug!("Initializing Nikon encryption keys for model: {}", model);
        Self {
            serial_number: None,
            shutter_count: None,
            camera_model: model,
            additional_params: std::collections::HashMap::new(),
            decryption_state: None,
        }
    }

    /// Store serial number encryption key
    /// ExifTool: Nikon.pm:1234 - if ($tagID == 0x001d) { $$et{NikonSerialKey} = $val; }
    pub fn store_serial_key(&mut self, serial: String) {
        debug!("Storing Nikon serial encryption key: {}", serial);
        self.serial_number = Some(serial);
    }

    /// Store shutter count encryption key
    /// ExifTool: Nikon.pm:1267 - if ($tagID == 0x00a7) { $$et{NikonCountKey} = $val; }
    pub fn store_count_key(&mut self, count: u32) {
        debug!("Storing Nikon shutter count encryption key: {}", count);
        self.shutter_count = Some(count);
    }

    /// Check if required encryption keys are available
    /// ExifTool: Validation before ProcessNikonEncrypted
    pub fn has_required_keys(&self) -> bool {
        let has_keys = self.serial_number.is_some() && self.shutter_count.is_some();

        if has_keys {
            debug!("Nikon encryption keys available (serial + count)");
        } else {
            trace!(
                "Nikon encryption keys incomplete - serial: {}, count: {}",
                self.serial_number.is_some(),
                self.shutter_count.is_some()
            );
        }

        has_keys
    }

    /// Get serial number key if available
    /// ExifTool: Access to $$et{NikonSerialKey}
    pub fn get_serial_key(&self) -> Option<&str> {
        self.serial_number.as_deref()
    }

    /// Get shutter count key if available
    /// ExifTool: Access to $$et{NikonCountKey}
    pub fn get_count_key(&self) -> Option<u32> {
        self.shutter_count
    }

    /// Store additional encryption parameter
    /// ExifTool: Model-specific parameter storage
    pub fn set_parameter(&mut self, key: String, value: String) {
        trace!("Setting Nikon encryption parameter: {} = {}", key, value);
        self.additional_params.insert(key, value);
    }

    /// Get encryption parameter
    /// ExifTool: Model-specific parameter retrieval
    pub fn get_parameter(&self, key: &str) -> Option<&str> {
        self.additional_params.get(key).map(|s| s.as_str())
    }

    /// Get numeric serial key for decryption
    /// ExifTool: SerialKey function (lines 13594-13601)
    pub fn get_serial_key_numeric(&self) -> Option<u32> {
        match &self.serial_number {
            Some(serial) => {
                // Use serial number as key if integral
                if let Ok(numeric) = serial.parse::<u32>() {
                    Some(numeric)
                } else {
                    // Model-specific defaults for non-numeric serials
                    if self.camera_model.contains("D50") {
                        Some(0x22)
                    } else {
                        Some(0x60) // D200, D40X, etc
                    }
                }
            }
            None => None,
        }
    }
}

/// Decrypt Nikon data block using XOR algorithm
/// ExifTool: Decrypt function (lines 13554-13588)  
///
/// Inputs: data block, start offset, number of bytes, serial key, count key, decryption state
/// Returns: decrypted data block
///
/// Notes: First call must provide serial/count keys for initialization.
/// Subsequent calls continue decryption from previous state.
pub fn decrypt_nikon_data(
    data: &[u8],
    start: usize,
    len: Option<usize>,
    serial: Option<u32>,
    count: Option<u32>,
    state: &mut Option<NikonDecryptionState>,
) -> Result<Vec<u8>> {
    use crate::types::ExifError;

    let max_len = data.len().saturating_sub(start);
    let len = len.unwrap_or(max_len).min(max_len);

    if len == 0 || start >= data.len() {
        return Ok(data.to_vec());
    }

    // Initialize decryption parameters if serial/count provided
    if let (Some(serial), Some(count)) = (serial, count) {
        debug!(
            "Initializing Nikon decryption with serial={}, count={}",
            serial, count
        );

        // ExifTool: key generation (lines 13564-13568)
        let mut key = 0u32;
        for i in 0..4 {
            key ^= (count >> (i * 8)) & 0xff;
        }

        let ci0 = XLAT_0[(serial & 0xff) as usize];
        let cj0 = XLAT_1[(key & 0xff) as usize];
        let ck0 = 0x60u8;

        *state = Some(NikonDecryptionState {
            ci0,
            cj0,
            ck0,
            decrypt_start: Some(start),
        });

        trace!(
            "Nikon decryption initialized: ci0={:#x}, cj0={:#x}, ck0={:#x}",
            ci0,
            cj0,
            ck0
        );
    }

    // Get current decryption state
    let decrypt_state = state
        .as_ref()
        .ok_or_else(|| ExifError::ParseError("Nikon decryption not initialized".to_string()))?;

    // Calculate decryption parameters for this position
    // ExifTool: lines 13571-13579
    let (mut cj, mut ck) = if let Some(decrypt_start) = decrypt_state.decrypt_start {
        if start != decrypt_start {
            // Calculate offset-adjusted parameters
            let n = start.saturating_sub(decrypt_start) as u32;
            let cj_offset = (decrypt_state.ci0 as u32
                * (n * decrypt_state.ck0 as u32 + (n * (n.saturating_sub(1))) / 2))
                & 0xff;
            let cj = ((decrypt_state.cj0 as u32 + cj_offset) & 0xff) as u8;
            let ck = ((decrypt_state.ck0 as u32 + n) & 0xff) as u8;
            (cj, ck)
        } else {
            (decrypt_state.cj0, decrypt_state.ck0)
        }
    } else {
        (decrypt_state.cj0, decrypt_state.ck0)
    };

    // Perform XOR decryption
    // ExifTool: lines 13580-13587
    let mut result = data.to_vec();
    let ci0 = decrypt_state.ci0;

    for i in 0..len {
        if start + i >= result.len() {
            break;
        }

        cj = ((cj as u32 + ci0 as u32 * ck as u32) & 0xff) as u8;
        ck = ((ck as u32 + 1) & 0xff) as u8;
        result[start + i] ^= cj;
    }

    trace!("Nikon decryption completed: {} bytes processed", len);
    Ok(result)
}

/// Process Nikon encrypted data with actual decryption
/// ExifTool: Nikon.pm ProcessNikonEncrypted function (lines 13892-14011)
pub fn process_nikon_encrypted(
    reader: &mut crate::exif::ExifReader,
    data: &[u8],
    keys: &mut NikonEncryptionKeys,
) -> crate::types::Result<Vec<u8>> {
    debug!("Processing Nikon encrypted data with decryption");

    if data.is_empty() {
        warn!("No encrypted data to process");
        return Ok(data.to_vec());
    }

    // Check if we have required keys
    let (serial, count) = match (keys.get_serial_key_numeric(), keys.get_count_key()) {
        (Some(serial), Some(count)) => {
            debug!(
                "Nikon decryption keys available: serial={}, count={}",
                serial, count
            );
            (serial, count)
        }
        _ => {
            warn!("Cannot decrypt Nikon information - missing keys");
            let tag_source = reader.create_tag_source_info("MakerNotes");
            reader.store_tag_with_precedence(
                0x00FE, // Custom tag for encryption status
                TagValue::String(
                    "Encrypted data detected (keys required for decryption)".to_string(),
                ),
                tag_source,
            );
            return Ok(data.to_vec()); // Return original data
        }
    };

    // Validate serial number and count format (ExifTool: lines 13898-13910)
    let serial_str = keys.get_serial_key().unwrap_or("");
    let count_str = count.to_string();

    if !serial_str.chars().all(|c| c.is_ascii_digit())
        || !count_str.chars().all(|c| c.is_ascii_digit())
    {
        let msg = if !serial_str.chars().all(|c| c.is_ascii_digit()) {
            "invalid SerialNumber"
        } else {
            "invalid ShutterCount"
        };
        warn!("Can't decrypt Nikon information ({} key)", msg);
        return Ok(data.to_vec());
    }

    // Perform decryption using our decrypt function
    let decrypted = decrypt_nikon_data(
        data,
        0,    // start at beginning
        None, // decrypt all data
        Some(serial),
        Some(count),
        &mut keys.decryption_state,
    )?;

    debug!("Nikon data decryption completed successfully");

    // Store decryption status for debugging
    let tag_source = reader.create_tag_source_info("MakerNotes");
    reader.store_tag_with_precedence(
        0x00FF, // Custom tag for encryption summary
        TagValue::String(format!(
            "Data decrypted successfully (serial={}, count={})",
            serial, count
        )),
        tag_source,
    );

    Ok(decrypted)
}

/// Validate encryption keys for specific camera model
/// ExifTool: Model-specific key validation logic
pub fn validate_encryption_keys(keys: &NikonEncryptionKeys, model: &str) -> Result<()> {
    use crate::types::ExifError;

    // Basic key availability check
    if !keys.has_required_keys() {
        return Err(ExifError::ParseError(
            "Required encryption keys not available".to_string(),
        ));
    }

    // Model-specific validation (skeleton)
    // TODO: Add model-specific key format validation in Phase 2+
    match model {
        model if model.contains("Z 9") => {
            debug!("Validated encryption keys for Nikon Z 9");
        }
        model if model.contains("D850") => {
            debug!("Validated encryption keys for Nikon D850");
        }
        _ => {
            debug!("Generic encryption key validation for model: {}", model);
        }
    }

    Ok(())
}

/// Process encrypted Nikon data sections
/// ExifTool: Nikon.pm ProcessNikonEncrypted function
pub fn process_encrypted_sections(
    reader: &mut ExifReader,
    base_offset: usize,
    keys: &mut NikonEncryptionKeys,
) -> Result<()> {
    trace!(
        "Processing encrypted Nikon sections at offset {:#x}",
        base_offset
    );

    let data = reader.get_data().to_vec();

    // Validate we have enough data for an IFD
    if base_offset + 2 > data.len() {
        debug!(
            "Insufficient data for encrypted section processing at offset {:#x}",
            base_offset
        );
        return Ok(());
    }

    // Get byte order from the reader's header
    let byte_order = match &reader.header {
        Some(header) => header.byte_order,
        None => {
            debug!("No TIFF header available for byte order, using little endian");
            ByteOrder::LittleEndian
        }
    };

    // Read number of IFD entries
    let num_entries = match byte_order.read_u16(&data, base_offset) {
        Ok(count) => count as usize,
        Err(_) => {
            debug!(
                "Failed to read IFD entry count for encrypted section at offset {:#x}",
                base_offset
            );
            return Ok(());
        }
    };

    debug!("Scanning {} entries for encrypted Nikon data", num_entries);

    let mut encrypted_sections_found = 0;
    let mut encrypted_tags = Vec::new();

    // Scan IFD entries for encrypted data indicators
    // ExifTool: Nikon.pm identifies encrypted sections by specific patterns
    for index in 0..num_entries {
        let entry_offset = base_offset + 2 + 12 * index;

        if entry_offset + 12 > data.len() {
            debug!(
                "Entry {} at offset {:#x} beyond data bounds during encryption scan",
                index, entry_offset
            );
            break;
        }

        // Parse IFD entry
        let entry = match IfdEntry::parse(&data, entry_offset, byte_order) {
            Ok(entry) => entry,
            Err(e) => {
                trace!(
                    "Failed to parse IFD entry {} during encryption scan: {:?}",
                    index,
                    e
                );
                continue;
            }
        };

        // Check for known encrypted data tags
        // ExifTool: Nikon.pm ProcessNikonEncrypted identifies these patterns
        if is_encrypted_nikon_tag(entry.tag_id) {
            encrypted_sections_found += 1;
            encrypted_tags.push(entry.tag_id);

            trace!("Found encrypted tag {:#x} at entry {}", entry.tag_id, index);

            // Process the encrypted section if we have keys and data
            if keys.has_required_keys() {
                let data_offset = entry.value_or_offset as usize;
                let data_len = entry.count as usize;

                if data_offset + data_len <= data.len() {
                    let section_data = &data[data_offset..data_offset + data_len];

                    // Dispatch to appropriate processing function based on tag ID
                    let process_result = match entry.tag_id {
                        0x0091 => {
                            // ShotInfo - use model-specific processing
                            crate::implementations::nikon::encrypted_processing::process_encrypted_shotinfo(
                                reader, section_data, keys
                            )
                        }
                        0x0098 => {
                            // LensData
                            crate::implementations::nikon::encrypted_processing::process_encrypted_lensdata(
                                reader, section_data, keys
                            )
                        }
                        0x0097 => {
                            // ColorBalance
                            crate::implementations::nikon::encrypted_processing::process_encrypted_colorbalance(
                                reader, section_data, keys
                            )
                        }
                        _ => {
                            // Other encrypted tags - use generic decryption
                            process_nikon_encrypted(reader, section_data, keys).map(|_| ())
                        }
                    };

                    if let Err(e) = process_result {
                        warn!("Failed to process encrypted tag {:#x}: {}", entry.tag_id, e);
                    }
                } else {
                    trace!("Encrypted tag {:#x} data beyond bounds (offset: {:#x}, len: {}, data_len: {})", 
                           entry.tag_id, data_offset, data_len, data.len());
                }
            } else {
                // Store placeholder when keys not available
                let tag_source = reader.create_tag_source_info("Nikon");
                reader.store_tag_with_precedence(
                    0x1000 + entry.tag_id, // Use offset to avoid conflicts
                    TagValue::String(format!(
                        "Encrypted tag {:#x} (keys required for decryption)",
                        entry.tag_id
                    )),
                    tag_source,
                );
            }
        }
    }

    // Store overall encryption status
    let tag_source = reader.create_tag_source_info("Nikon");

    if encrypted_sections_found > 0 {
        let encryption_summary = if keys.has_required_keys() {
            format!(
                "Found {} encrypted sections (keys available: serial={}, count={})",
                encrypted_sections_found,
                keys.get_serial_key().unwrap_or("none"),
                keys.get_count_key()
                    .map(|c| c.to_string())
                    .unwrap_or("none".to_string())
            )
        } else {
            format!(
                "Found {encrypted_sections_found} encrypted sections (keys incomplete for decryption)"
            )
        };

        reader.store_tag_with_precedence(
            0x00FF, // Custom tag for encryption summary
            TagValue::String(encryption_summary),
            tag_source,
        );

        debug!(
            "Detected {} encrypted Nikon sections: {:?}",
            encrypted_sections_found, encrypted_tags
        );
    } else {
        reader.store_tag_with_precedence(
            0x00FF, // Custom tag for encryption summary
            "No encrypted sections detected".into(),
            tag_source,
        );

        debug!("No encrypted Nikon sections detected");
    }

    debug!(
        "Encrypted section processing completed - {} sections found",
        encrypted_sections_found
    );
    Ok(())
}

/// Check if a tag ID represents encrypted Nikon data
/// ExifTool: Nikon.pm encrypted tag identification
pub fn is_encrypted_nikon_tag(tag_id: u16) -> bool {
    // ExifTool: Nikon.pm identifies these tags as commonly encrypted
    // This is a simplified version - real ExifTool has model-specific lists
    match tag_id {
        // Common encrypted tags from ExifTool Nikon.pm
        0x0088 => true, // AFInfo (often encrypted)
        0x0091 => true, // ShotInfo (often encrypted)
        0x0097 => true, // ColorBalance (often encrypted)
        0x0098 => true, // LensData (often encrypted)
        0x00A8 => true, // FlashInfo (often encrypted)
        0x00B0 => true, // MultiExposure (often encrypted)
        0x00B7 => true, // AFInfo2 (often encrypted)
        0x00B9 => true, // AFTune (often encrypted)
        _ => false,     // Other tags are typically not encrypted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_keys_initialization() {
        let keys = NikonEncryptionKeys::new("NIKON Z 9".to_string());
        assert_eq!(keys.camera_model, "NIKON Z 9");
        assert!(!keys.has_required_keys());
    }

    #[test]
    fn test_serial_key_storage() {
        let mut keys = NikonEncryptionKeys::new("NIKON D850".to_string());
        keys.store_serial_key("12345678".to_string());

        assert_eq!(keys.get_serial_key(), Some("12345678"));
        assert!(!keys.has_required_keys()); // Still need count key
    }

    #[test]
    fn test_count_key_storage() {
        let mut keys = NikonEncryptionKeys::new("NIKON Z 9".to_string());
        keys.store_count_key(1000);

        assert_eq!(keys.get_count_key(), Some(1000));
        assert!(!keys.has_required_keys()); // Still need serial key
    }

    #[test]
    fn test_complete_key_set() {
        let mut keys = NikonEncryptionKeys::new("NIKON D850".to_string());
        keys.store_serial_key("12345678".to_string());
        keys.store_count_key(1500);

        assert!(keys.has_required_keys());
        assert_eq!(keys.get_serial_key(), Some("12345678"));
        assert_eq!(keys.get_count_key(), Some(1500));
    }

    #[test]
    fn test_additional_parameters() {
        let mut keys = NikonEncryptionKeys::new("NIKON Z 9".to_string());
        keys.set_parameter("DecryptStart".to_string(), "0x100".to_string());

        assert_eq!(keys.get_parameter("DecryptStart"), Some("0x100"));
        assert_eq!(keys.get_parameter("Unknown"), None);
    }

    #[test]
    fn test_encryption_validation_without_keys() {
        let keys = NikonEncryptionKeys::new("NIKON D850".to_string());
        let result = validate_encryption_keys(&keys, "NIKON D850");

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Required encryption keys not available"));
    }

    #[test]
    fn test_encryption_validation_with_keys() {
        let mut keys = NikonEncryptionKeys::new("NIKON Z 9".to_string());
        keys.store_serial_key("12345678".to_string());
        keys.store_count_key(2000);

        let result = validate_encryption_keys(&keys, "NIKON Z 9");
        assert!(result.is_ok());
    }

    #[test]
    fn test_serial_key_numeric_conversion() {
        // Test numeric serial number
        let mut keys = NikonEncryptionKeys::new("NIKON D850".to_string());
        keys.store_serial_key("12345678".to_string());
        assert_eq!(keys.get_serial_key_numeric(), Some(12345678));

        // Test D50 model-specific default
        let mut keys_d50 = NikonEncryptionKeys::new("NIKON D50".to_string());
        keys_d50.store_serial_key("non-numeric".to_string());
        assert_eq!(keys_d50.get_serial_key_numeric(), Some(0x22));

        // Test other model default
        let mut keys_d200 = NikonEncryptionKeys::new("NIKON D200".to_string());
        keys_d200.store_serial_key("non-numeric".to_string());
        assert_eq!(keys_d200.get_serial_key_numeric(), Some(0x60));
    }

    #[test]
    fn test_nikon_decrypt_basic() {
        // Test data - simple pattern that should decrypt predictably
        let test_data = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let serial: u32 = 12345678;
        let count: u32 = 1000;
        let mut state = None;

        let result = decrypt_nikon_data(&test_data, 0, None, Some(serial), Some(count), &mut state);

        assert!(result.is_ok());
        let decrypted = result.unwrap();

        // Verify data was modified (XOR should change values)
        assert_ne!(decrypted, test_data);
        assert_eq!(decrypted.len(), test_data.len());

        // Verify state was initialized
        assert!(state.is_some());
        let decrypt_state = state.unwrap();
        assert_eq!(decrypt_state.ci0, XLAT_0[(serial & 0xff) as usize]);
    }

    #[test]
    fn test_nikon_decrypt_state_management() {
        let test_data = vec![0x12, 0x34, 0x56, 0x78];
        let serial: u32 = 12345678;
        let count: u32 = 2000;
        let mut state = None;

        // First decryption initializes state
        let result1 = decrypt_nikon_data(
            &test_data,
            0,
            Some(2),
            Some(serial),
            Some(count),
            &mut state,
        );
        assert!(result1.is_ok());
        assert!(state.is_some());

        // Second decryption continues from state
        let result2 = decrypt_nikon_data(
            &test_data,
            2,
            Some(2),
            None, // No keys needed for continuation
            None,
            &mut state,
        );
        assert!(result2.is_ok());
    }

    #[test]
    fn test_nikon_decrypt_empty_data() {
        let empty_data = vec![];
        let mut state = None;

        let result = decrypt_nikon_data(&empty_data, 0, None, Some(12345), Some(1000), &mut state);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), empty_data);
    }

    #[test]
    fn test_nikon_decrypt_without_initialization() {
        let test_data = vec![0x12, 0x34, 0x56, 0x78];
        let mut state = None;

        // Try to decrypt without providing keys
        let result = decrypt_nikon_data(&test_data, 0, None, None, None, &mut state);

        assert!(result.is_err());
    }

    #[test]
    fn test_nikon_decrypt_key_generation() {
        // Test the key generation algorithm matches ExifTool
        let serial: u32 = 0x12345678;
        let count: u32 = 0xabcdef01;
        let mut state = None;

        let test_data = vec![0x00]; // Single byte for minimal test
        let result = decrypt_nikon_data(&test_data, 0, None, Some(serial), Some(count), &mut state);

        assert!(result.is_ok());
        assert!(state.is_some());

        let decrypt_state = state.unwrap();

        // Verify ci0 uses serial with XLAT[0]
        assert_eq!(decrypt_state.ci0, XLAT_0[(serial & 0xff) as usize]);

        // Verify key calculation: key = count[0] ^ count[1] ^ count[2] ^ count[3]
        let expected_key = (count & 0xff)
            ^ ((count >> 8) & 0xff)
            ^ ((count >> 16) & 0xff)
            ^ ((count >> 24) & 0xff);
        assert_eq!(decrypt_state.cj0, XLAT_1[(expected_key & 0xff) as usize]);

        // Verify ck0 initialization
        assert_eq!(decrypt_state.ck0, 0x60);
    }

    #[test]
    fn test_nikon_decrypt_offset_calculation() {
        let test_data = vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let serial: u32 = 1000;
        let count: u32 = 500;
        let mut state = None;

        // Decrypt from start
        let result1 = decrypt_nikon_data(
            &test_data,
            0,
            Some(4),
            Some(serial),
            Some(count),
            &mut state,
        );
        assert!(result1.is_ok());

        // Decrypt from middle with offset calculation
        let result2 = decrypt_nikon_data(
            &test_data,
            4,
            Some(4),
            None, // Continue from state
            None,
            &mut state,
        );
        assert!(result2.is_ok());

        // Both results should be different from original
        let decrypted1 = result1.unwrap();
        let decrypted2 = result2.unwrap();

        assert_ne!(&decrypted1[0..4], &test_data[0..4]);
        assert_ne!(&decrypted2[4..8], &test_data[4..8]);
    }

    #[test]
    fn test_xlat_tables_consistency() {
        // Verify XLAT tables have correct size and are not all zeros
        assert_eq!(XLAT_0.len(), 256);
        assert_eq!(XLAT_1.len(), 256);

        // Check that tables are not empty (should have varied values)
        let xlat0_sum: u32 = XLAT_0.iter().map(|&x| x as u32).sum();
        let xlat1_sum: u32 = XLAT_1.iter().map(|&x| x as u32).sum();

        assert!(xlat0_sum > 0);
        assert!(xlat1_sum > 0);

        // Tables should be different
        assert_ne!(XLAT_0.to_vec(), XLAT_1.to_vec());
    }

    #[test]
    fn test_encrypted_tag_detection() {
        // Test known encrypted tags
        assert!(is_encrypted_nikon_tag(0x0088)); // AFInfo
        assert!(is_encrypted_nikon_tag(0x0091)); // ShotInfo
        assert!(is_encrypted_nikon_tag(0x0097)); // ColorBalance
        assert!(is_encrypted_nikon_tag(0x0098)); // LensData
        assert!(is_encrypted_nikon_tag(0x00A8)); // FlashInfo

        // Test non-encrypted tags
        assert!(!is_encrypted_nikon_tag(0x0001)); // Version
        assert!(!is_encrypted_nikon_tag(0x0002)); // ISO
        assert!(!is_encrypted_nikon_tag(0x001d)); // SerialNumber
    }
}
