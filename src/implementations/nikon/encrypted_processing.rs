//! Nikon encrypted section processing and model-specific binary data extraction
//!
//! **Trust ExifTool**: This code translates ExifTool's ProcessNikonEncrypted and model-specific
//! ShotInfo table processing exactly.
//!
//! ExifTool Reference: lib/Image/ExifTool/Nikon.pm lines 2507-2636 (model detection),
//! lines 13808-13849 (offset processing), lines 8198-8977 (ShotInfo tables)
//!
//! This module handles the ProcessBinaryData dispatch for popular Nikon cameras:
//! - D850: ShotInfo version 0243, offset table at 0x0c
//! - Z8: ShotInfo version 0806, offset table at 0x24  
//! - Z9: ShotInfo version 0805, offset table at 0x24
//! - Z7/Z7II: ShotInfo version 0800/0803, offset table at 0x24

use crate::exif::ExifReader;
use crate::implementations::nikon::encryption::{decrypt_nikon_data, NikonEncryptionKeys};
use crate::tiff_types::ByteOrder;
use crate::types::{ExifError, Result, TagValue};
use tracing::{debug, trace, warn};

/// Nikon camera model identification based on ShotInfo version
/// ExifTool: Nikon.pm lines 2507-2636 - model detection conditions
#[derive(Debug, Clone, PartialEq)]
pub enum NikonCameraModel {
    /// D850 (firmware 1.00b) - ShotInfo version 0243
    /// ExifTool: line 2507, condition '$$valPt =~ /^0243/'
    D850,

    /// Z8 (firmware 1.00) - ShotInfo version 0806
    /// ExifTool: line 2619, condition '$$valPt =~ /^0806/'
    Z8,

    /// Z9 (firmware 1.00) - ShotInfo version 0805
    /// ExifTool: line 2628, condition '$$valPt =~ /^0805/'
    Z9,

    /// Z7/Z7II series - ShotInfo versions 0800, 0801, 0802, 0803, 0804, 0807, 0808
    /// ExifTool: line 2609, condition '$$valPt =~ /^080[0123478]/'
    Z7Series,

    /// Unknown model - cannot determine specific processing
    Unknown,
}

/// Model-specific offset scheme configuration
/// ExifTool: NIKON_OFFSETS table position varies by camera model
#[derive(Debug, Clone)]
pub struct ModelOffsetConfig {
    /// Position of offset table in ShotInfo data
    /// ExifTool: $$tagTablePtr{VARS}{NIKON_OFFSETS}
    pub offset_table_position: usize,

    /// Byte order for this model
    /// ExifTool: SubDirectory ByteOrder setting
    pub byte_order: ByteOrder,

    /// Decryption start position
    /// ExifTool: SubDirectory DecryptStart setting  
    pub decrypt_start: usize,
}

impl NikonCameraModel {
    /// Detect camera model from ShotInfo version header
    /// ExifTool: Nikon.pm lines 2507-2636 - Condition patterns
    pub fn detect_from_shotinfo_version(data: &[u8]) -> Self {
        if data.len() < 4 {
            debug!("Insufficient data for Nikon model detection");
            return Self::Unknown;
        }

        // Read version as 4-character string (ExifTool uses regex patterns)
        let version_bytes = &data[0..4];
        let version_str = String::from_utf8_lossy(version_bytes);

        trace!(
            "Detecting Nikon model from ShotInfo version: {}",
            version_str
        );

        // ExifTool: Condition => '$$valPt =~ /^0243/'
        if version_str.starts_with("0243") {
            debug!("Detected Nikon D850 (ShotInfo version 0243)");
            return Self::D850;
        }

        // ExifTool: Condition => '$$valPt =~ /^0806/'
        if version_str.starts_with("0806") {
            debug!("Detected Nikon Z8 (ShotInfo version 0806)");
            return Self::Z8;
        }

        // ExifTool: Condition => '$$valPt =~ /^0805/'
        if version_str.starts_with("0805") {
            debug!("Detected Nikon Z9 (ShotInfo version 0805)");
            return Self::Z9;
        }

        // ExifTool: Condition => '$$valPt =~ /^080[0123478]/'
        if version_str.starts_with("080") {
            let third_char = version_str.chars().nth(3).unwrap_or('?');
            if "0123478".contains(third_char) {
                debug!(
                    "Detected Nikon Z7/Z series (ShotInfo version 080{})",
                    third_char
                );
                return Self::Z7Series;
            }
        }

        debug!("Unknown Nikon model for ShotInfo version: {}", version_str);
        Self::Unknown
    }

    /// Get model-specific offset configuration
    /// ExifTool: NIKON_OFFSETS and SubDirectory settings per model
    pub fn get_offset_config(&self) -> ModelOffsetConfig {
        match self {
            // ExifTool: D850 ShotInfo table lines 8198-8252
            // NIKON_OFFSETS => 0x0c, DecryptStart => 4
            Self::D850 => ModelOffsetConfig {
                offset_table_position: 0x0c,
                byte_order: ByteOrder::LittleEndian, // Inherited from context
                decrypt_start: 4,
            },

            // ExifTool: Z8 ShotInfo table lines 8831-8907
            // NIKON_OFFSETS => 0x24, DecryptStart => 4, ByteOrder => 'LittleEndian'
            Self::Z8 => ModelOffsetConfig {
                offset_table_position: 0x24,
                byte_order: ByteOrder::LittleEndian,
                decrypt_start: 4,
            },

            // ExifTool: Z9 ShotInfo table lines 8910-8977
            // NIKON_OFFSETS => 0x24, DecryptStart => 4, ByteOrder => 'LittleEndian'
            Self::Z9 => ModelOffsetConfig {
                offset_table_position: 0x24,
                byte_order: ByteOrder::LittleEndian,
                decrypt_start: 4,
            },

            // ExifTool: Z7II ShotInfo table lines 8682-8828 (covers Z7)
            // NIKON_OFFSETS => 0x24, DecryptStart => 4, ByteOrder => 'LittleEndian'
            Self::Z7Series => ModelOffsetConfig {
                offset_table_position: 0x24,
                byte_order: ByteOrder::LittleEndian,
                decrypt_start: 4,
            },

            // Unknown model - use conservative defaults
            Self::Unknown => ModelOffsetConfig {
                offset_table_position: 0x24, // Most common for newer cameras
                byte_order: ByteOrder::LittleEndian,
                decrypt_start: 4,
            },
        }
    }

    /// Get human-readable model name for debugging
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::D850 => "NIKON D850",
            Self::Z8 => "NIKON Z8",
            Self::Z9 => "NIKON Z9",
            Self::Z7Series => "NIKON Z7/Z7II Series",
            Self::Unknown => "Unknown Nikon Model",
        }
    }
}

/// Process encrypted ShotInfo section with model-specific offset handling
/// ExifTool: Nikon.pm PrepareNikonOffsets function (lines 13808-13849)
pub fn process_encrypted_shotinfo(
    reader: &mut ExifReader,
    data: &[u8],
    keys: &mut NikonEncryptionKeys,
) -> Result<()> {
    debug!(
        "Processing encrypted ShotInfo section ({} bytes)",
        data.len()
    );

    if data.len() < 8 {
        warn!("ShotInfo data too small for processing");
        return Ok(());
    }

    // Step 1: Detect camera model from version header
    let model = NikonCameraModel::detect_from_shotinfo_version(data);
    let config = model.get_offset_config();

    trace!(
        "Using offset config for {}: table at {:#x}, decrypt start {}",
        model.model_name(),
        config.offset_table_position,
        config.decrypt_start
    );

    // Step 2: Check if we have decryption keys
    let (serial, count) = match (keys.get_serial_key_numeric(), keys.get_count_key()) {
        (Some(serial), Some(count)) => (serial, count),
        _ => {
            warn!("Cannot process encrypted ShotInfo - missing decryption keys");
            let tag_source = reader.create_tag_source_info("Nikon");
            reader.store_tag_with_precedence(
                0x0091, // ShotInfo tag ID
                TagValue::String(format!(
                    "Encrypted ShotInfo detected ({}) - decryption keys required",
                    model.model_name()
                )),
                tag_source,
            );
            return Ok(());
        }
    };

    // Step 3: Decrypt the data using our decryption algorithm
    let decrypted_data = decrypt_nikon_data(
        data,
        0,
        None, // Decrypt all data
        Some(serial),
        Some(count),
        &mut keys.decryption_state,
    )?;

    debug!(
        "ShotInfo decryption completed for {} ({} bytes)",
        model.model_name(),
        decrypted_data.len()
    );

    // Step 4: Process offset table header (ExifTool: PrepareNikonOffsets)
    if config.offset_table_position + 4 > decrypted_data.len() {
        warn!(
            "ShotInfo offset table position {:#x} beyond data bounds",
            config.offset_table_position
        );
        return Ok(());
    }

    // Read number of offsets (ExifTool: Get32u($dataPt, $offset))
    let num_offsets = config
        .byte_order
        .read_u32(&decrypted_data, config.offset_table_position)
        .map_err(|e| ExifError::ParseError(format!("Failed to read offset count: {}", e)))?;

    trace!("ShotInfo contains {} subdirectory offsets", num_offsets);

    // Step 5: Process each offset to extract actual tags using binary data extraction
    // ExifTool: for ($i=0; $i<$numOffsets; ++$i) loop
    crate::implementations::nikon::binary_data_extraction::extract_shotinfo_tags(
        reader,
        &decrypted_data,
        &model,
        &config,
    )?;

    // Store successful processing status
    let tag_source = reader.create_tag_source_info("Nikon");
    reader.store_tag_with_precedence(
        0x0091, // ShotInfo tag ID
        TagValue::String(format!(
            "ShotInfo processed successfully for {} ({} subdirectories)",
            model.model_name(),
            num_offsets
        )),
        tag_source,
    );

    debug!(
        "ShotInfo processing completed for {}: {} subdirectories processed",
        model.model_name(),
        num_offsets
    );

    Ok(())
}

/// Process encrypted LensData section
/// ExifTool: Tag 0x0098 processing
pub fn process_encrypted_lensdata(
    reader: &mut ExifReader,
    data: &[u8],
    keys: &mut NikonEncryptionKeys,
) -> Result<()> {
    debug!(
        "Processing encrypted LensData section ({} bytes)",
        data.len()
    );

    if data.is_empty() {
        warn!("Empty LensData section");
        return Ok(());
    }

    // Check for decryption keys
    let (serial, count) = match (keys.get_serial_key_numeric(), keys.get_count_key()) {
        (Some(serial), Some(count)) => (serial, count),
        _ => {
            warn!("Cannot process encrypted LensData - missing decryption keys");
            let tag_source = reader.create_tag_source_info("Nikon");
            reader.store_tag_with_precedence(
                0x0098, // LensData tag ID
                TagValue::String(
                    "Encrypted LensData detected - decryption keys required".to_string(),
                ),
                tag_source,
            );
            return Ok(());
        }
    };

    // Decrypt the LensData
    let decrypted_data = decrypt_nikon_data(
        data,
        0,
        None,
        Some(serial),
        Some(count),
        &mut keys.decryption_state,
    )?;

    debug!(
        "LensData decryption completed ({} bytes)",
        decrypted_data.len()
    );

    // Extract actual lens information from decrypted data
    let model = NikonCameraModel::detect_from_shotinfo_version(&decrypted_data);
    crate::implementations::nikon::binary_data_extraction::extract_lensdata_tags(
        reader,
        &decrypted_data,
        &model,
    )?;

    // Store processing status
    let tag_source = reader.create_tag_source_info("Nikon");
    reader.store_tag_with_precedence(
        0x0098, // LensData tag ID
        TagValue::String(format!(
            "LensData processed successfully ({} bytes, {} model)",
            decrypted_data.len(),
            model.model_name()
        )),
        tag_source,
    );

    Ok(())
}

/// Process encrypted ColorBalance section
/// ExifTool: Tag 0x0097 processing
pub fn process_encrypted_colorbalance(
    reader: &mut ExifReader,
    data: &[u8],
    keys: &mut NikonEncryptionKeys,
) -> Result<()> {
    debug!(
        "Processing encrypted ColorBalance section ({} bytes)",
        data.len()
    );

    if data.is_empty() {
        warn!("Empty ColorBalance section");
        return Ok(());
    }

    // Check for decryption keys
    let (serial, count) = match (keys.get_serial_key_numeric(), keys.get_count_key()) {
        (Some(serial), Some(count)) => (serial, count),
        _ => {
            warn!("Cannot process encrypted ColorBalance - missing decryption keys");
            let tag_source = reader.create_tag_source_info("Nikon");
            reader.store_tag_with_precedence(
                0x0097, // ColorBalance tag ID
                TagValue::String(
                    "Encrypted ColorBalance detected - decryption keys required".to_string(),
                ),
                tag_source,
            );
            return Ok(());
        }
    };

    // Decrypt the ColorBalance data
    let decrypted_data = decrypt_nikon_data(
        data,
        0,
        None,
        Some(serial),
        Some(count),
        &mut keys.decryption_state,
    )?;

    debug!(
        "ColorBalance decryption completed ({} bytes)",
        decrypted_data.len()
    );

    // Extract actual color balance information from decrypted data
    let model = NikonCameraModel::detect_from_shotinfo_version(&decrypted_data);
    crate::implementations::nikon::binary_data_extraction::extract_colorbalance_tags(
        reader,
        &decrypted_data,
        &model,
    )?;

    // Store processing status
    let tag_source = reader.create_tag_source_info("Nikon");
    reader.store_tag_with_precedence(
        0x0097, // ColorBalance tag ID
        TagValue::String(format!(
            "ColorBalance processed successfully ({} bytes, {} model)",
            decrypted_data.len(),
            model.model_name()
        )),
        tag_source,
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_detection_d850() {
        let shotinfo_data = b"0243test_data_here";
        let model = NikonCameraModel::detect_from_shotinfo_version(shotinfo_data);
        assert_eq!(model, NikonCameraModel::D850);
        assert_eq!(model.model_name(), "NIKON D850");
    }

    #[test]
    fn test_model_detection_z8() {
        let shotinfo_data = b"0806test_data_here";
        let model = NikonCameraModel::detect_from_shotinfo_version(shotinfo_data);
        assert_eq!(model, NikonCameraModel::Z8);
        assert_eq!(model.model_name(), "NIKON Z8");
    }

    #[test]
    fn test_model_detection_z9() {
        let shotinfo_data = b"0805test_data_here";
        let model = NikonCameraModel::detect_from_shotinfo_version(shotinfo_data);
        assert_eq!(model, NikonCameraModel::Z9);
        assert_eq!(model.model_name(), "NIKON Z9");
    }

    #[test]
    fn test_model_detection_z7_series() {
        // Test various Z7 series patterns: 0800, 0801, 0802, 0803, 0804, 0807, 0808
        for pattern in ["0800", "0801", "0802", "0803", "0804", "0807", "0808"] {
            let shotinfo_data = format!("{}test_data_here", pattern);
            let model = NikonCameraModel::detect_from_shotinfo_version(shotinfo_data.as_bytes());
            assert_eq!(model, NikonCameraModel::Z7Series);
            assert_eq!(model.model_name(), "NIKON Z7/Z7II Series");
        }
    }

    #[test]
    fn test_model_detection_unknown() {
        let shotinfo_data = b"9999unknown_model";
        let model = NikonCameraModel::detect_from_shotinfo_version(shotinfo_data);
        assert_eq!(model, NikonCameraModel::Unknown);
        assert_eq!(model.model_name(), "Unknown Nikon Model");
    }

    #[test]
    fn test_model_detection_insufficient_data() {
        let shotinfo_data = b"04"; // Less than 4 bytes
        let model = NikonCameraModel::detect_from_shotinfo_version(shotinfo_data);
        assert_eq!(model, NikonCameraModel::Unknown);
    }

    #[test]
    fn test_offset_config_d850() {
        let model = NikonCameraModel::D850;
        let config = model.get_offset_config();

        assert_eq!(config.offset_table_position, 0x0c);
        assert_eq!(config.byte_order, ByteOrder::LittleEndian);
        assert_eq!(config.decrypt_start, 4);
    }

    #[test]
    fn test_offset_config_z_series() {
        for model in [
            NikonCameraModel::Z8,
            NikonCameraModel::Z9,
            NikonCameraModel::Z7Series,
        ] {
            let config = model.get_offset_config();

            assert_eq!(config.offset_table_position, 0x24);
            assert_eq!(config.byte_order, ByteOrder::LittleEndian);
            assert_eq!(config.decrypt_start, 4);
        }
    }
}
