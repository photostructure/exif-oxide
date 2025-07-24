//! AVIF file format processing
//!
//! This module handles AVIF (AV1 Image File Format) file processing,
//! extracting metadata and dimensions from AVIF files following ExifTool's implementation.
//!
//! AVIF uses the ISO Base Media File Format (like MP4/MOV) with specific box structures.
//! Dimensions are stored in the 'ispe' (Image Spatial Extent) box within the 'meta' container.
//!
//! Reference: third-party/exiftool/lib/Image/ExifTool/QuickTime.pm:2946-2959

use crate::types::{Result, TagEntry, TagValue};

/// ISO Base Media File Format box header size (size + type)
const BOX_HEADER_SIZE: usize = 8;

/// Maximum number of boxes to scan for AVIF processing
const MAX_BOXES_TO_SCAN: usize = 100;

/// AVIF image dimensions extracted from 'ispe' box
/// ExifTool reference: QuickTime.pm:2946-2959 (ispe box processing)
#[derive(Debug, Clone)]
pub struct AvifImageProperties {
    pub width: u32,
    pub height: u32,
}

/// Primary item information for HEIC/HEIF files
/// ExifTool reference: QuickTime.pm:3550-3557 (pitm box processing)
#[derive(Debug, Clone)]
pub struct PrimaryItemInfo {
    pub primary_item_id: u32,
}

/// Item information from iinf box
/// ExifTool reference: QuickTime.pm:3730-3740 (iinf box processing)
#[derive(Debug, Clone)]
pub struct ItemInfo {
    pub item_id: u32,
    pub item_type: [u8; 4],
    pub item_name: String,
}

/// Property association from ipma box
/// ExifTool reference: QuickTime.pm:10320-10380 (ipma box processing)
#[derive(Debug, Clone)]
pub struct ItemPropertyAssociation {
    pub item_id: u32,
    pub property_indices: Vec<u16>,
    // Note: essential_flags could be added here if needed for complete ExifTool compatibility
}

/// ISO Base Media File Format box structure
#[derive(Debug, Clone)]
pub struct IsoBox {
    #[allow(dead_code)]
    pub size: u64,
    pub box_type: [u8; 4],
    pub data: Vec<u8>,
}

/// Parse ISO Base Media File Format box header
///
/// Box structure:
/// - Size: 4 bytes big-endian (u32) - if 1, then 8-byte extended size follows
/// - Type: 4 bytes ASCII (e.g., 'ftyp', 'meta', 'iprp', 'ipco', 'ispe')
/// - Data: variable length
///
/// ExifTool reference: QuickTime.pm:3254-3280 (ReadAtom function)
pub fn parse_box_header(data: &[u8], offset: usize) -> Result<(IsoBox, usize)> {
    if data.len() < offset + BOX_HEADER_SIZE {
        return Err(crate::types::ExifError::InvalidFormat(
            "Not enough data for box header".to_string(),
        ));
    }

    // Read box size (4 bytes big-endian)
    let size32 = u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);

    // Read box type (4 bytes ASCII)
    let mut box_type = [0u8; 4];
    box_type.copy_from_slice(&data[offset + 4..offset + 8]);

    let (box_size, header_size) = if size32 == 1 {
        // Extended size: next 8 bytes contain the actual size
        if data.len() < offset + 16 {
            return Err(crate::types::ExifError::InvalidFormat(
                "Not enough data for extended box size".to_string(),
            ));
        }
        let size64 = u64::from_be_bytes([
            data[offset + 8],
            data[offset + 9],
            data[offset + 10],
            data[offset + 11],
            data[offset + 12],
            data[offset + 13],
            data[offset + 14],
            data[offset + 15],
        ]);
        (size64, 16)
    } else {
        (size32 as u64, 8)
    };

    // Calculate data size (total box size minus header size)
    let data_size = if box_size >= header_size as u64 {
        (box_size - header_size as u64) as usize
    } else {
        return Err(crate::types::ExifError::InvalidFormat(
            "Invalid box size".to_string(),
        ));
    };

    // Read box data
    let data_start = offset + header_size;
    let data_end = data_start + data_size;

    if data.len() < data_end {
        return Err(crate::types::ExifError::InvalidFormat(
            "Not enough data for box content".to_string(),
        ));
    }

    let box_data = data[data_start..data_end].to_vec();

    let iso_box = IsoBox {
        size: box_size,
        box_type,
        data: box_data,
    };

    Ok((iso_box, data_end))
}

/// Find a specific box type within a parent box
///
/// Recursively searches through box hierarchy to find target box type.
/// Used to navigate AVIF structure: ftyp → meta → iprp → ipco → ispe
pub fn find_box_by_type(data: &[u8], target_type: &[u8; 4]) -> Result<Option<IsoBox>> {
    let mut offset = 0;
    let mut boxes_scanned = 0;

    while offset < data.len() && boxes_scanned < MAX_BOXES_TO_SCAN {
        match parse_box_header(data, offset) {
            Ok((box_data, next_offset)) => {
                if &box_data.box_type == target_type {
                    return Ok(Some(box_data));
                }

                // If this is a container box, search recursively
                if is_container_box(&box_data.box_type) {
                    if let Ok(Some(nested_box)) = find_box_by_type(&box_data.data, target_type) {
                        return Ok(Some(nested_box));
                    }
                }

                offset = next_offset;
                boxes_scanned += 1;
            }
            Err(_) => break, // Stop on parse error
        }
    }

    Ok(None)
}

/// Check if a box type is a container that can contain other boxes
///
/// Container boxes in AVIF: 'meta', 'iprp', 'ipco'
/// ExifTool reference: QuickTime.pm container box handling
fn is_container_box(box_type: &[u8; 4]) -> bool {
    matches!(
        box_type,
        b"meta" | b"iprp" | b"ipco" | b"moov" | b"trak" | b"mdia"
    )
}

/// Parse AVIF 'ispe' (Image Spatial Extent) box to extract image dimensions
///
/// ispe box structure (ExifTool QuickTime.pm:2946-2959):
/// - Version/Flags: 4 bytes (must be [0,0,0,0])
/// - Width: 4 bytes big-endian (u32)
/// - Height: 4 bytes big-endian (u32)
///
/// ExifTool code reference:
/// ```perl
/// ispe => {
///     Name => 'ImageSpatialExtents',
///     Condition => '$$valPt =~ /^\\0\\0\\0\\0/',
///     SubDirectory => {
///         TagTable => 'Image::ExifTool::QuickTime::ImageSpatialExtents',
///         Start => 4,
///     },
/// },
/// # ImageSpatialExtents table:
/// 0 => { Name => 'ImageWidth',  Format => 'int32u' },
/// 4 => { Name => 'ImageHeight', Format => 'int32u' },
/// ```
pub fn parse_ispe_box(ispe_data: &[u8]) -> Result<AvifImageProperties> {
    if ispe_data.len() < 12 {
        return Err(crate::types::ExifError::InvalidFormat(
            "ispe box too short".to_string(),
        ));
    }

    // Check version/flags (must be [0,0,0,0])
    // ExifTool condition: $$valPt =~ /^\\0\\0\\0\\0/
    if ispe_data[0..4] != [0, 0, 0, 0] {
        return Err(crate::types::ExifError::InvalidFormat(
            "Invalid ispe box version/flags".to_string(),
        ));
    }

    // Extract width and height (skip 4-byte header)
    // ExifTool: Start => 4, Format => 'int32u' (big-endian 32-bit unsigned)
    let width = u32::from_be_bytes([ispe_data[4], ispe_data[5], ispe_data[6], ispe_data[7]]);

    let height = u32::from_be_bytes([ispe_data[8], ispe_data[9], ispe_data[10], ispe_data[11]]);

    Ok(AvifImageProperties { width, height })
}

/// Parse 'pitm' (Primary Item) box to extract primary item ID
///
/// pitm box structure (ExifTool QuickTime.pm:3550-3557):
/// - Version: 1 byte (0 = 16-bit item ID, 1+ = 32-bit item ID)
/// - Flags: 3 bytes
/// - Item ID: 2 or 4 bytes depending on version
///
/// ExifTool code reference:
/// ```perl
/// pitm => [{
///     Name => 'PrimaryItemReference',
///     Condition => '$$valPt =~ /^\0/', # (version 0?)
///     RawConv => '$$self{PrimaryItem} = unpack("x4n",$val)',
/// },{
///     Name => 'PrimaryItemReference',
///     RawConv => '$$self{PrimaryItem} = unpack("x4N",$val)',
/// }],
/// ```
pub fn parse_pitm_box(pitm_data: &[u8]) -> Result<PrimaryItemInfo> {
    if pitm_data.len() < 6 {
        return Err(crate::types::ExifError::InvalidFormat(
            "pitm box too short".to_string(),
        ));
    }

    // Check version (first byte after box header)
    let version = pitm_data[0];

    let primary_item_id = if version == 0 {
        // Version 0: 16-bit item ID at offset 4 (after version/flags)
        // ExifTool: unpack("x4n", $val) - skip 4 bytes, read 16-bit big-endian
        if pitm_data.len() < 6 {
            return Err(crate::types::ExifError::InvalidFormat(
                "pitm box too short for version 0".to_string(),
            ));
        }
        u16::from_be_bytes([pitm_data[4], pitm_data[5]]) as u32
    } else {
        // Version 1+: 32-bit item ID at offset 4 (after version/flags)
        // ExifTool: unpack("x4N", $val) - skip 4 bytes, read 32-bit big-endian
        if pitm_data.len() < 8 {
            return Err(crate::types::ExifError::InvalidFormat(
                "pitm box too short for version 1+".to_string(),
            ));
        }
        u32::from_be_bytes([pitm_data[4], pitm_data[5], pitm_data[6], pitm_data[7]])
    };

    tracing::debug!(
        "Found primary item ID: {} (version: {})",
        primary_item_id,
        version
    );

    Ok(PrimaryItemInfo { primary_item_id })
}

/// Parse 'iinf' (Item Information) box to extract item details
///
/// iinf box structure (ExifTool QuickTime.pm:3730-3740):
/// - Version/Flags: 4 bytes
/// - Entry count: 2 or 4 bytes depending on version
/// - Item entries: variable length infe boxes
///
/// Each infe (Item Information Entry) contains:
/// - Item ID: 2 or 4 bytes depending on version
/// - Protection index: 2 bytes
/// - Item type: 4 bytes ASCII
/// - Item name: null-terminated string
pub fn parse_iinf_box(iinf_data: &[u8]) -> Result<Vec<ItemInfo>> {
    if iinf_data.len() < 6 {
        return Err(crate::types::ExifError::InvalidFormat(
            "iinf box too short".to_string(),
        ));
    }

    let version = iinf_data[0];

    // Get entry count based on version
    let (entry_count, entries_start) = if version == 0 {
        // Version 0: 16-bit entry count at offset 4
        let count = u16::from_be_bytes([iinf_data[4], iinf_data[5]]) as u32;
        (count, 6)
    } else {
        // Version 1+: 32-bit entry count at offset 4
        if iinf_data.len() < 8 {
            return Err(crate::types::ExifError::InvalidFormat(
                "iinf box too short for version 1+ entry count".to_string(),
            ));
        }
        let count = u32::from_be_bytes([iinf_data[4], iinf_data[5], iinf_data[6], iinf_data[7]]);
        (count, 8)
    };

    tracing::debug!("iinf box: version={}, entry_count={}", version, entry_count);

    let mut items = Vec::new();
    let mut offset = entries_start;

    // Parse each infe (Item Information Entry) box
    for i in 0..entry_count {
        if offset >= iinf_data.len() {
            break;
        }

        // Parse infe box header
        match parse_box_header(&iinf_data[offset..], 0) {
            Ok((infe_box, box_end)) => {
                if &infe_box.box_type == b"infe" {
                    match parse_infe_box(&infe_box.data) {
                        Ok(item_info) => {
                            tracing::debug!(
                                "Item {}: ID={}, type={:?}, name='{}'",
                                i,
                                item_info.item_id,
                                String::from_utf8_lossy(&item_info.item_type),
                                item_info.item_name
                            );
                            items.push(item_info);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse infe box {}: {}", i, e);
                        }
                    }
                }
                offset += box_end;
            }
            Err(e) => {
                tracing::warn!("Failed to parse infe box header {}: {}", i, e);
                break;
            }
        }
    }

    Ok(items)
}

/// Parse 'infe' (Item Information Entry) box
///
/// infe box structure:
/// - Version/Flags: 4 bytes
/// - Item ID: 2 or 4 bytes depending on version
/// - Protection index: 2 bytes  
/// - Item type: 4 bytes ASCII
/// - Item name: null-terminated string
fn parse_infe_box(infe_data: &[u8]) -> Result<ItemInfo> {
    if infe_data.len() < 12 {
        return Err(crate::types::ExifError::InvalidFormat(
            "infe box too short".to_string(),
        ));
    }

    let version = infe_data[0];

    let (item_id, item_type_offset) = if version == 0 || version == 1 {
        // Version 0/1: 16-bit item ID at offset 4
        let id = u16::from_be_bytes([infe_data[4], infe_data[5]]) as u32;
        (id, 8) // Skip version/flags (4) + item_id (2) + protection_index (2)
    } else {
        // Version 2+: 32-bit item ID at offset 4
        if infe_data.len() < 14 {
            return Err(crate::types::ExifError::InvalidFormat(
                "infe box too short for version 2+ item ID".to_string(),
            ));
        }
        let id = u32::from_be_bytes([infe_data[4], infe_data[5], infe_data[6], infe_data[7]]);
        (id, 10) // Skip version/flags (4) + item_id (4) + protection_index (2)
    };

    // Extract item type (4 bytes ASCII)
    if infe_data.len() < item_type_offset + 4 {
        return Err(crate::types::ExifError::InvalidFormat(
            "infe box too short for item type".to_string(),
        ));
    }

    let mut item_type = [0u8; 4];
    item_type.copy_from_slice(&infe_data[item_type_offset..item_type_offset + 4]);

    // Extract item name (null-terminated string after item type)
    let name_start = item_type_offset + 4;
    let item_name = if name_start < infe_data.len() {
        let name_bytes = &infe_data[name_start..];
        let null_pos = name_bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(name_bytes.len());
        String::from_utf8_lossy(&name_bytes[..null_pos]).to_string()
    } else {
        String::new()
    };

    Ok(ItemInfo {
        item_id,
        item_type,
        item_name,
    })
}

/// Parse 'ipma' (Item Property Association) box to link items with properties
///
/// ipma box structure (ExifTool QuickTime.pm:10320-10380):
/// - Version/Flags: 4 bytes
/// - Entry count: 4 bytes
/// - For each entry:
///   - Item ID: 2 or 4 bytes depending on version
///   - Association count: 1 byte
///   - Property associations: variable length
///
/// ExifTool reference: ParseItemPropAssoc function
pub fn parse_ipma_box(ipma_data: &[u8]) -> Result<Vec<ItemPropertyAssociation>> {
    if ipma_data.len() < 8 {
        return Err(crate::types::ExifError::InvalidFormat(
            "ipma box too short".to_string(),
        ));
    }

    let version = ipma_data[0];
    let flags = [ipma_data[1], ipma_data[2], ipma_data[3]];

    // Entry count is always 32-bit
    let entry_count = u32::from_be_bytes([ipma_data[4], ipma_data[5], ipma_data[6], ipma_data[7]]);

    tracing::debug!(
        "ipma box: version={}, flags={:?}, entry_count={}",
        version,
        flags,
        entry_count
    );

    let mut associations = Vec::new();
    let mut offset = 8;

    for i in 0..entry_count {
        if offset >= ipma_data.len() {
            break;
        }

        // Parse item ID based on version
        let (item_id, id_size) = if version == 0 {
            // Version 0: 16-bit item ID
            if offset + 2 > ipma_data.len() {
                break;
            }
            let id = u16::from_be_bytes([ipma_data[offset], ipma_data[offset + 1]]) as u32;
            (id, 2)
        } else {
            // Version 1+: 32-bit item ID
            if offset + 4 > ipma_data.len() {
                break;
            }
            let id = u32::from_be_bytes([
                ipma_data[offset],
                ipma_data[offset + 1],
                ipma_data[offset + 2],
                ipma_data[offset + 3],
            ]);
            (id, 4)
        };

        offset += id_size;

        // Parse association count
        if offset >= ipma_data.len() {
            break;
        }
        let association_count = ipma_data[offset];
        offset += 1;

        tracing::debug!(
            "Item {}: ID={}, association_count={}",
            i,
            item_id,
            association_count
        );

        // Parse property associations
        let mut property_indices = Vec::new();

        for j in 0..association_count {
            if offset >= ipma_data.len() {
                break;
            }

            // Property index and essential flag
            // If flags[0] & 1, property index is 15 bits + 1 essential bit
            // Otherwise, property index is 7 bits + 1 essential bit
            let (property_index, essential) = if flags[0] & 1 != 0 {
                // 15-bit property index + 1 essential bit (2 bytes total)
                if offset + 2 > ipma_data.len() {
                    break;
                }
                let val = u16::from_be_bytes([ipma_data[offset], ipma_data[offset + 1]]);
                let essential = (val & 0x8000) != 0; // Top bit is essential flag
                let index = val & 0x7FFF; // Bottom 15 bits are property index
                offset += 2;
                (index, essential)
            } else {
                // 7-bit property index + 1 essential bit (1 byte total)
                let val = ipma_data[offset];
                let essential = (val & 0x80) != 0; // Top bit is essential flag
                let index = (val & 0x7F) as u16; // Bottom 7 bits are property index
                offset += 1;
                (index, essential)
            };

            tracing::debug!(
                "  Association {}: property_index={}, essential={}",
                j,
                property_index,
                essential
            );

            property_indices.push(property_index);
            // Note: essential flag is parsed but not stored - could be added if needed
        }

        associations.push(ItemPropertyAssociation {
            item_id,
            property_indices,
        });
    }

    Ok(associations)
}

/// Extract HEIC/HEIF image dimensions using ExifTool's primary item detection
///
/// This function implements ExifTool's complete primary item detection logic:
/// 1. Parse pitm box to get primary item ID (QuickTime.pm:3550-3557)
/// 2. Parse iinf box to build item information map (QuickTime.pm:3730-3740)
/// 3. Parse ipma box to associate items with properties (QuickTime.pm:10320-10380)
/// 4. Find ispe boxes and determine which belong to primary item (QuickTime.pm:6450-6460)
/// 5. Extract dimensions only from primary item's ispe box (DOC_NUM logic)
///
/// ExifTool reference: QuickTime.pm:2946-2959 (ispe processing with DOC_NUM check)
pub fn extract_heic_dimensions_primary_item(data: &[u8]) -> Result<AvifImageProperties> {
    // Find meta box
    let meta_box = find_box_by_type(data, b"meta")?.ok_or_else(|| {
        crate::types::ExifError::InvalidFormat("No meta box found in HEIC file".to_string())
    })?;

    // Note: meta box has a 4-byte version/flags header, so actual content starts at offset 4
    let meta_content = if meta_box.data.len() >= 4 {
        &meta_box.data[4..]
    } else {
        &meta_box.data[..]
    };

    // Step 1: Parse pitm box to get primary item ID
    let pitm_box = find_box_by_type(meta_content, b"pitm")?.ok_or_else(|| {
        crate::types::ExifError::InvalidFormat("No pitm box found in meta box".to_string())
    })?;

    let primary_item_info = parse_pitm_box(&pitm_box.data)?;
    let primary_item_id = primary_item_info.primary_item_id;

    tracing::debug!("Primary item ID: {}", primary_item_id);

    // Step 2: Parse iinf box to build item information map
    let iinf_box = find_box_by_type(meta_content, b"iinf")?.ok_or_else(|| {
        crate::types::ExifError::InvalidFormat("No iinf box found in meta box".to_string())
    })?;

    let items = parse_iinf_box(&iinf_box.data)?;
    tracing::debug!("Found {} items in iinf box", items.len());

    // Step 3: Parse ipma box to associate items with properties
    let ipma_box = find_box_by_type(meta_content, b"ipma")?.ok_or_else(|| {
        crate::types::ExifError::InvalidFormat("No ipma box found in meta box".to_string())
    })?;

    let associations = parse_ipma_box(&ipma_box.data)?;
    tracing::debug!(
        "Found {} property associations in ipma box",
        associations.len()
    );

    // Step 4: Find iprp/ipco container with properties
    let iprp_box = find_box_by_type(meta_content, b"iprp")?.ok_or_else(|| {
        crate::types::ExifError::InvalidFormat("No iprp box found in meta box".to_string())
    })?;

    let ipco_box = find_box_by_type(&iprp_box.data, b"ipco")?.ok_or_else(|| {
        crate::types::ExifError::InvalidFormat("No ipco box found in iprp box".to_string())
    })?;

    // Step 5: Find all ispe boxes in ipco and map them to property indices
    let mut ispe_boxes_with_indices = Vec::new();
    let mut offset = 0;
    let mut property_index = 1; // Properties are 1-indexed

    while offset < ipco_box.data.len() {
        match parse_box_header(&ipco_box.data, offset) {
            Ok((property_box, next_offset)) => {
                if &property_box.box_type == b"ispe" {
                    tracing::debug!(
                        "Found ispe box at property index {}, size: {} bytes",
                        property_index,
                        property_box.data.len()
                    );
                    ispe_boxes_with_indices.push((property_index, property_box));
                }
                property_index += 1;
                offset = next_offset;
            }
            Err(_) => break,
        }
    }

    if ispe_boxes_with_indices.is_empty() {
        return Err(crate::types::ExifError::InvalidFormat(
            "No ispe boxes found in ipco container".to_string(),
        ));
    }

    // Step 6: Apply ExifTool's DOC_NUM logic to find primary item's ispe box
    // Find the association for the primary item
    let primary_association = associations
        .iter()
        .find(|assoc| assoc.item_id == primary_item_id)
        .ok_or_else(|| {
            crate::types::ExifError::InvalidFormat(format!(
                "No property association found for primary item ID {}",
                primary_item_id
            ))
        })?;

    tracing::debug!(
        "Primary item {} has {} associated properties: {:?}",
        primary_item_id,
        primary_association.property_indices.len(),
        primary_association.property_indices
    );

    // Find the ispe box associated with the primary item
    // ExifTool logic: only extract dimensions unless DOC_NUM is set (i.e., for primary document)
    for &property_idx in &primary_association.property_indices {
        if let Some((_, ispe_box)) = ispe_boxes_with_indices
            .iter()
            .find(|(idx, _)| *idx == property_idx as u32)
        {
            tracing::debug!(
                "Found primary item's ispe box at property index {}",
                property_idx
            );

            // Parse the primary item's ispe box for dimensions
            match parse_ispe_box(&ispe_box.data) {
                Ok(props) => {
                    tracing::debug!(
                        "Extracted primary image dimensions: {}x{} from property index {}",
                        props.width,
                        props.height,
                        property_idx
                    );
                    return Ok(props);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse primary item's ispe box at index {}: {}",
                        property_idx,
                        e
                    );
                }
            }
        }
    }

    // Fallback: if we can't find primary item's ispe box, try the first valid ispe box
    // This matches ExifTool's behavior when primary item detection fails
    tracing::warn!(
        "Could not find ispe box for primary item {}, falling back to first ispe box",
        primary_item_id
    );

    for (idx, ispe_box) in &ispe_boxes_with_indices {
        match parse_ispe_box(&ispe_box.data) {
            Ok(props) => {
                tracing::debug!(
                    "Fallback: extracted dimensions {}x{} from property index {}",
                    props.width,
                    props.height,
                    idx
                );
                return Ok(props);
            }
            Err(e) => {
                tracing::warn!("Failed to parse fallback ispe box at index {}: {}", idx, e);
            }
        }
    }

    Err(crate::types::ExifError::InvalidFormat(
        "No valid ispe boxes found for dimension extraction".to_string(),
    ))
}

/// Extract AVIF image dimensions from file data
///
/// AVIF dimension extraction follows this box hierarchy:
/// 1. Find 'meta' box (metadata container)
/// 2. Within meta, find 'iprp' box (item properties container)
/// 3. Within iprp, find 'ipco' box (item property container)  
/// 4. Within ipco, find 'ispe' box (image spatial extents)
/// 5. Parse ispe box for width/height
///
/// ExifTool reference: QuickTime.pm:2946-2959
pub fn extract_avif_dimensions(data: &[u8]) -> Result<AvifImageProperties> {
    // Navigate the box hierarchy: meta → iprp → ipco → ispe

    // Find meta box
    let meta_box = find_box_by_type(data, b"meta")?.ok_or_else(|| {
        crate::types::ExifError::InvalidFormat("No meta box found in AVIF file".to_string())
    })?;

    // Find iprp box within meta
    // Note: meta box has a 4-byte version/flags header, so actual content starts at offset 4
    let meta_content = if meta_box.data.len() >= 4 {
        &meta_box.data[4..]
    } else {
        &meta_box.data[..]
    };

    let iprp_box = find_box_by_type(meta_content, b"iprp")?.ok_or_else(|| {
        // Debug: list all boxes in meta to see what's actually there
        let mut offset = 0;
        let mut found_boxes = Vec::new();
        while offset < meta_content.len() {
            if let Ok((box_data, next_offset)) = parse_box_header(meta_content, offset) {
                found_boxes.push(String::from_utf8_lossy(&box_data.box_type).to_string());
                offset = next_offset;
            } else {
                break;
            }
        }
        crate::types::ExifError::InvalidFormat(format!(
            "No iprp box found in meta box. Found boxes: {:?}",
            found_boxes
        ))
    })?;

    // Find ipco box within iprp
    let ipco_box = find_box_by_type(&iprp_box.data, b"ipco")?.ok_or_else(|| {
        crate::types::ExifError::InvalidFormat("No ipco box found in iprp box".to_string())
    })?;

    // Find ispe box within ipco
    let ispe_box = find_box_by_type(&ipco_box.data, b"ispe")?.ok_or_else(|| {
        crate::types::ExifError::InvalidFormat("No ispe box found in ipco box".to_string())
    })?;

    // Parse ispe box for dimensions
    parse_ispe_box(&ispe_box.data)
}

/// Create AVIF TagEntry objects from image properties
///
/// Following ExifTool's AVIF group assignment and tag naming conventions.
/// AVIF image dimensions are assigned to "File" group in ExifTool.
///
/// ExifTool creates File:ImageWidth and File:ImageHeight for AVIF files
pub fn create_avif_tag_entries(props: &AvifImageProperties) -> Vec<TagEntry> {
    vec![
        // File:ImageWidth - ExifTool creates this for AVIF files
        TagEntry {
            group: "File".to_string(),
            group1: "File".to_string(),
            name: "ImageWidth".to_string(),
            value: TagValue::U32(props.width),
            print: TagValue::U32(props.width),
        },
        // File:ImageHeight - ExifTool creates this for AVIF files
        TagEntry {
            group: "File".to_string(),
            group1: "File".to_string(),
            name: "ImageHeight".to_string(),
            value: TagValue::U32(props.height),
            print: TagValue::U32(props.height),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_header_parsing() {
        // Create test box: size=16, type='test', empty data
        let test_data = [
            0x00, 0x00, 0x00, 0x10, // size: 16 bytes
            b't', b'e', b's', b't', // type: 'test'
            0x01, 0x02, 0x03, 0x04, // 4 bytes of data
            0x05, 0x06, 0x07, 0x08, // 4 more bytes of data
        ];

        let (box_data, next_offset) = parse_box_header(&test_data, 0).unwrap();

        assert_eq!(box_data.size, 16);
        assert_eq!(&box_data.box_type, b"test");
        assert_eq!(box_data.data.len(), 8); // 16 - 8 (header) = 8
        assert_eq!(next_offset, 16);
    }

    #[test]
    fn test_ispe_box_parsing() {
        // Create test ispe box data
        let ispe_data = [
            0x00, 0x00, 0x00, 0x00, // version/flags (must be 0)
            0x00, 0x00, 0x04, 0x00, // width: 1024
            0x00, 0x00, 0x03, 0x00, // height: 768
        ];

        let props = parse_ispe_box(&ispe_data).unwrap();

        assert_eq!(props.width, 1024);
        assert_eq!(props.height, 768);
    }

    #[test]
    fn test_ispe_box_invalid_version() {
        // Create ispe box with invalid version/flags
        let ispe_data = [
            0x00, 0x01, 0x00, 0x00, // invalid version/flags
            0x00, 0x00, 0x04, 0x00, // width: 1024
            0x00, 0x00, 0x03, 0x00, // height: 768
        ];

        let result = parse_ispe_box(&ispe_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_avif_tag_entries() {
        let props = AvifImageProperties {
            width: 1920,
            height: 1080,
        };

        let entries = create_avif_tag_entries(&props);

        assert_eq!(entries.len(), 2);

        // Check ImageWidth entry
        let width_entry = entries.iter().find(|e| e.name == "ImageWidth").unwrap();
        assert_eq!(width_entry.group, "File");
        assert_eq!(width_entry.value, TagValue::U32(1920));

        // Check ImageHeight entry
        let height_entry = entries.iter().find(|e| e.name == "ImageHeight").unwrap();
        assert_eq!(height_entry.group, "File");
        assert_eq!(height_entry.value, TagValue::U32(1080));
    }
}
