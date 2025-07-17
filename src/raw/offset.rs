//! RAW offset processing infrastructure
//!
//! This module provides offset calculation systems for different RAW format types.
//! Some manufacturers like Panasonic use entry-based offsets where the offset
//! to data is stored in IFD entry values rather than fixed positions.
//!
//! ExifTool Reference: Various RAW modules use different offset strategies

use crate::types::{ExifError, Result};
use std::collections::HashMap;

/// Entry-based offset processor for manufacturers like Panasonic
/// ExifTool: PanasonicRaw.pm uses IFD entry values as offsets to actual data
/// This is different from fixed-offset processors used by simpler formats
pub struct EntryBasedOffsetProcessor {
    /// Map of tag ID to offset extraction rules
    /// ExifTool: Different tags have different offset calculation methods
    offset_rules: HashMap<u16, OffsetExtractionRule>,
}

/// Rule for extracting offset from an IFD entry
/// ExifTool: Different tags store offsets in different ways
#[derive(Debug, Clone)]
pub struct OffsetExtractionRule {
    /// Tag ID this rule applies to
    /// ExifTool: Tag number in IFD
    pub tag_id: u16,

    /// How to extract the offset from the IFD entry
    /// ExifTool: Can be from value offset field or actual value
    pub offset_field: OffsetField,

    /// Base offset for calculations
    /// ExifTool: Different manufacturers use different base offsets
    pub base: OffsetBase,

    /// Optional additional offset to add
    /// ExifTool: Some formats require additional offset adjustments
    pub additional_offset: i32,
}

/// How to extract offset from IFD entry
/// ExifTool: Different extraction methods for different manufacturers
#[derive(Debug, Clone, Copy)]
pub enum OffsetField {
    /// Use the value offset field from IFD entry
    /// ExifTool: Standard TIFF offset where value_offset points to data
    ValueOffset,

    /// Use the actual value as offset
    /// ExifTool: The value itself contains the offset (Panasonic pattern)
    ActualValue,
}

/// Base offset for offset calculations
/// ExifTool: Different manufacturers use different reference points
#[derive(Debug, Clone, Copy)]
pub enum OffsetBase {
    /// Offset from start of file
    /// ExifTool: Most common base
    FileStart,

    /// Offset from start of IFD
    /// ExifTool: Some formats use IFD-relative offsets
    IfdStart,

    /// Offset from start of maker note data
    /// ExifTool: Common for maker note subdirectories
    MakerNoteStart,

    /// Offset from current data position
    /// ExifTool: Relative to current read position
    DataPosition,
}

/// IFD entry structure (simplified)
/// ExifTool: Standard TIFF IFD entry format
#[derive(Debug, Clone)]
pub struct IfdEntry {
    /// Tag ID
    /// ExifTool: 2-byte tag identifier
    pub tag: u16,

    /// Data type
    /// ExifTool: 2-byte type identifier (BYTE, LONG, etc.)
    pub data_type: u16,

    /// Number of values
    /// ExifTool: 4-byte count of values
    pub count: u32,

    /// Value or offset to value
    /// ExifTool: 4-byte value or offset depending on size
    pub value_offset: u32,

    /// Actual file position of this entry
    /// Used for relative calculations
    pub entry_position: u64,
}

impl EntryBasedOffsetProcessor {
    /// Create new entry-based offset processor
    /// ExifTool: Initialize with manufacturer-specific rules
    pub fn new(offset_rules: HashMap<u16, OffsetExtractionRule>) -> Self {
        Self { offset_rules }
    }

    /// Get offset extraction rule for a tag
    /// ExifTool: Look up how to process this tag's offset
    pub fn get_rule(&self, tag_id: u16) -> Option<&OffsetExtractionRule> {
        self.offset_rules.get(&tag_id)
    }

    /// Calculate actual offset from IFD entry and rule
    /// ExifTool: Apply manufacturer-specific offset calculation
    pub fn calculate_offset(
        &self,
        entry: &IfdEntry,
        rule: &OffsetExtractionRule,
        context: &OffsetContext,
    ) -> Result<u64> {
        // Extract raw offset based on field type
        // ExifTool: Different extraction methods for different tags
        let raw_offset = match rule.offset_field {
            OffsetField::ValueOffset => entry.value_offset as i64,
            OffsetField::ActualValue => {
                // For actual value, we need to read the value if it's stored elsewhere
                // For now, assume it's directly in value_offset field
                entry.value_offset as i64
            }
        };

        // Apply base offset calculation
        // ExifTool: Different base calculations for different manufacturers
        let base_offset = match rule.base {
            OffsetBase::FileStart => 0i64,
            OffsetBase::IfdStart => context.ifd_start as i64,
            OffsetBase::MakerNoteStart => context.maker_note_start as i64,
            OffsetBase::DataPosition => context.current_position as i64,
        };

        // Calculate final offset
        // ExifTool: Base + raw offset + additional adjustment
        let final_offset = base_offset + raw_offset + rule.additional_offset as i64;

        if final_offset < 0 {
            return Err(ExifError::ParseError(format!(
                "Calculated negative offset for tag 0x{:04x}: {}",
                entry.tag, final_offset
            )));
        }

        Ok(final_offset as u64)
    }

    /// Add new offset extraction rule
    /// ExifTool: Extend processor with additional rules
    pub fn add_rule(&mut self, rule: OffsetExtractionRule) {
        self.offset_rules.insert(rule.tag_id, rule);
    }

    /// Remove offset extraction rule
    /// ExifTool: Remove rule for specific tag
    pub fn remove_rule(&mut self, tag_id: u16) -> Option<OffsetExtractionRule> {
        self.offset_rules.remove(&tag_id)
    }

    /// Get all configured tag IDs
    /// ExifTool: List all tags that have offset rules
    pub fn get_configured_tags(&self) -> Vec<u16> {
        self.offset_rules.keys().copied().collect()
    }
}

/// Context for offset calculations
/// ExifTool: Provides reference points for relative offset calculations
#[derive(Debug, Clone)]
pub struct OffsetContext {
    /// Start of current IFD in file
    /// ExifTool: IFD base position
    pub ifd_start: u64,

    /// Start of maker note data in file
    /// ExifTool: Maker note base position
    pub maker_note_start: u64,

    /// Current read position in file
    /// ExifTool: Current file position
    pub current_position: u64,

    /// Total file size (for bounds checking)
    /// ExifTool: File size for validation
    pub file_size: u64,
}

impl OffsetContext {
    /// Create new offset context
    pub fn new(
        ifd_start: u64,
        maker_note_start: u64,
        current_position: u64,
        file_size: u64,
    ) -> Self {
        Self {
            ifd_start,
            maker_note_start,
            current_position,
            file_size,
        }
    }

    /// Validate that offset is within file bounds
    /// ExifTool: Bounds checking for calculated offsets
    pub fn validate_offset(&self, offset: u64, data_size: u64) -> Result<()> {
        if offset >= self.file_size {
            return Err(ExifError::ParseError(format!(
                "Offset {} exceeds file size {}",
                offset, self.file_size
            )));
        }

        if offset + data_size > self.file_size {
            return Err(ExifError::ParseError(format!(
                "Data at offset {offset} with size {data_size} exceeds file bounds"
            )));
        }

        Ok(())
    }
}

/// Simple offset processor for fixed-offset formats like Kyocera/Minolta
/// ExifTool: Simple formats use fixed offsets from known base positions
pub struct SimpleOffsetProcessor {
    /// Base offset for all calculations
    /// ExifTool: Fixed base position (usually start of data block)
    base_offset: u64,
}

impl SimpleOffsetProcessor {
    /// Create new simple offset processor
    /// ExifTool: Initialize with base offset
    pub fn new(base_offset: u64) -> Self {
        Self { base_offset }
    }

    /// Calculate offset by adding relative offset to base
    /// ExifTool: Simple addition for fixed-offset formats
    pub fn calculate_offset(&self, relative_offset: u64) -> u64 {
        self.base_offset + relative_offset
    }

    /// Update base offset
    /// ExifTool: Change base position if needed
    pub fn set_base_offset(&mut self, base_offset: u64) {
        self.base_offset = base_offset;
    }

    /// Get current base offset
    /// ExifTool: Get current base position
    pub fn get_base_offset(&self) -> u64 {
        self.base_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_based_offset_processor() {
        // Create test rule for Panasonic tag 0x002e
        let rule = OffsetExtractionRule {
            tag_id: 0x002e,
            offset_field: OffsetField::ActualValue,
            base: OffsetBase::MakerNoteStart,
            additional_offset: 0,
        };

        let mut rules = HashMap::new();
        rules.insert(0x002e, rule);

        let processor = EntryBasedOffsetProcessor::new(rules);

        // Test rule retrieval
        assert!(processor.get_rule(0x002e).is_some());
        assert!(processor.get_rule(0x0000).is_none());

        // Test configured tags
        let tags = processor.get_configured_tags();
        assert_eq!(tags.len(), 1);
        assert!(tags.contains(&0x002e));
    }

    #[test]
    fn test_offset_calculation() {
        let rule = OffsetExtractionRule {
            tag_id: 0x002e,
            offset_field: OffsetField::ActualValue,
            base: OffsetBase::MakerNoteStart,
            additional_offset: 10,
        };

        let mut rules = HashMap::new();
        rules.insert(0x002e, rule);

        let processor = EntryBasedOffsetProcessor::new(rules);

        let entry = IfdEntry {
            tag: 0x002e,
            data_type: 4, // LONG
            count: 1,
            value_offset: 0x1000,
            entry_position: 0x500,
        };

        let context = OffsetContext::new(0x400, 0x800, 0x500, 0x10000);

        let offset = processor
            .calculate_offset(&entry, processor.get_rule(0x002e).unwrap(), &context)
            .unwrap();

        // Expected: maker_note_start (0x800) + value_offset (0x1000) + additional (10) = 0x180A
        assert_eq!(offset, 0x180A);
    }

    #[test]
    fn test_offset_context_validation() {
        let context = OffsetContext::new(0x400, 0x800, 0x500, 0x10000);

        // Valid offset and size
        assert!(context.validate_offset(0x1000, 0x100).is_ok());

        // Offset exceeds file size
        assert!(context.validate_offset(0x20000, 0x100).is_err());

        // Data extends beyond file
        assert!(context.validate_offset(0xFF00, 0x200).is_err());
    }

    #[test]
    fn test_simple_offset_processor() {
        let mut processor = SimpleOffsetProcessor::new(0x1000);

        assert_eq!(processor.calculate_offset(0x100), 0x1100);
        assert_eq!(processor.get_base_offset(), 0x1000);

        processor.set_base_offset(0x2000);
        assert_eq!(processor.calculate_offset(0x100), 0x2100);
        assert_eq!(processor.get_base_offset(), 0x2000);
    }

    #[test]
    fn test_offset_field_types() {
        let rule_value_offset = OffsetExtractionRule {
            tag_id: 0x0001,
            offset_field: OffsetField::ValueOffset,
            base: OffsetBase::FileStart,
            additional_offset: 0,
        };

        let rule_actual_value = OffsetExtractionRule {
            tag_id: 0x0002,
            offset_field: OffsetField::ActualValue,
            base: OffsetBase::FileStart,
            additional_offset: 0,
        };

        // Both should use value_offset field for now (simplified implementation)
        let entry = IfdEntry {
            tag: 0x0001,
            data_type: 4,
            count: 1,
            value_offset: 0x1234,
            entry_position: 0x500,
        };

        let context = OffsetContext::new(0, 0, 0, 0x10000);

        let mut rules = HashMap::new();
        rules.insert(0x0001, rule_value_offset);
        rules.insert(0x0002, rule_actual_value);

        let processor = EntryBasedOffsetProcessor::new(rules);

        let offset1 = processor
            .calculate_offset(&entry, processor.get_rule(0x0001).unwrap(), &context)
            .unwrap();
        let offset2 = processor
            .calculate_offset(&entry, processor.get_rule(0x0002).unwrap(), &context)
            .unwrap();

        // Both should return the same result in current implementation
        assert_eq!(offset1, 0x1234);
        assert_eq!(offset2, 0x1234);
    }
}
