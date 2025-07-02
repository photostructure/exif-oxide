# Milestone 12: Variable ProcessBinaryData

**Duration**: 3 weeks  
**Goal**: Handle variable-length formats with DataMember dependencies

## Overview

Many camera manufacturers use variable-length data structures where the size or format of later fields depends on values extracted from earlier fields. This milestone implements ExifTool's DataMember system for handling these dependencies.

## Background

From ExifTool's ProcessBinaryData:

- Tags can be marked as `DataMember` to store values for later use
- Format expressions like `string[$val{3}]` reference previously extracted values
- Canon AF data is a prime example where `NumAFPoints` determines array sizes

## Key Concepts

### DataMember Tags

```perl
# ExifTool example
2 => {
    Name => 'NumAFPoints',
    DataMember => 'NumAFPoints',  # Store for later use
    Format => 'int16u',
},
8 => {
    Name => 'AFPointsInFocus',
    Format => 'int16u[$val{NumAFPoints}]',  # Variable-length array
},
```

### Variable Format Expressions

- `string[$val{3}]` - String length from tag 3
- `int16u[$val{NumAFPoints}]` - Array size from NumAFPoints
- `var_string` - Null-terminated variable string

## Deliverables

### 1. DataMember Dependency System

```rust
pub struct DataMemberSystem {
    // Values extracted from DataMember tags
    values: HashMap<String, DataMemberValue>,

    // Track dependencies for two-phase extraction
    dependencies: HashMap<TagId, Vec<String>>,
}

impl DataMemberSystem {
    pub fn mark_data_member(&mut self, tag_id: TagId, name: String) {
        self.dependencies.entry(tag_id).or_default().push(name);
    }

    pub fn store_value(&mut self, name: String, value: DataMemberValue) {
        self.values.insert(name, value);
    }

    pub fn get_value(&self, name: &str) -> Option<&DataMemberValue> {
        self.values.get(name)
    }
}
```

### 2. Two-Phase Extraction

```rust
impl ExifReader {
    fn process_binary_data_with_dependencies(
        &mut self,
        data: &[u8],
        table: &BinaryDataTable,
    ) -> Result<()> {
        // Phase 1: Extract DataMember tags
        for (&tag_id, tag_def) in &table.tags {
            if tag_def.data_member.is_some() {
                let value = self.extract_tag_value(data, tag_id, tag_def)?;

                // Store in DataMember system
                if let Some(member_name) = &tag_def.data_member {
                    self.data_members.store_value(
                        member_name.clone(),
                        value.into(),
                    );
                }
            }
        }

        // Phase 2: Extract remaining tags with resolved formats
        for (&tag_id, tag_def) in &table.tags {
            if tag_def.data_member.is_none() {
                let format = self.resolve_format(&tag_def.format)?;
                let value = self.extract_with_format(data, tag_id, &format)?;
                self.add_tag(&tag_def.name, value);
            }
        }

        Ok(())
    }
}
```

### 3. Format Expression Evaluation

```rust
pub struct FormatResolver<'a> {
    data_members: &'a DataMemberSystem,
}

impl<'a> FormatResolver<'a> {
    pub fn resolve(&self, format_expr: &str) -> Result<ResolvedFormat> {
        // Handle expressions like "string[$val{3}]"
        if let Some(caps) = VAL_EXPR_REGEX.captures(format_expr) {
            let base_format = &caps[1];  // "string"
            let val_ref = &caps[2];      // "3" or "NumAFPoints"

            let count = self.resolve_value_ref(val_ref)?;

            match base_format {
                "string" => Ok(ResolvedFormat::String(count)),
                "int16u" => Ok(ResolvedFormat::Array(
                    Box::new(ResolvedFormat::Int16u),
                    count,
                )),
                _ => Err(format!("Unknown format: {}", base_format)),
            }
        } else {
            // Simple format without expression
            ResolvedFormat::parse(format_expr)
        }
    }

    fn resolve_value_ref(&self, val_ref: &str) -> Result<usize> {
        // Try as tag index first
        if let Ok(index) = val_ref.parse::<usize>() {
            // Look up by tag index
            if let Some(value) = self.data_members.get_by_index(index) {
                return value.as_usize()
                    .ok_or("Value is not a valid size");
            }
        }

        // Try as named DataMember
        if let Some(value) = self.data_members.get_value(val_ref) {
            return value.as_usize()
                .ok_or("Value is not a valid size");
        }

        Err(format!("Unknown value reference: {}", val_ref))
    }
}
```

### 4. Variable Format Parsers

```rust
// src/implementations/formats/variable.rs

pub fn parse_string_from_val(
    data: &[u8],
    offset: usize,
    length: usize,
) -> Result<(String, usize)> {
    // Validate bounds
    if offset + length > data.len() {
        return Err(ExifError::format_overflow(offset, length, data.len()));
    }

    // Extract string
    let bytes = &data[offset..offset + length];
    let string = String::from_utf8_lossy(bytes).into_owned();

    Ok((string, length))
}

pub fn parse_var_string(
    data: &[u8],
    offset: usize,
) -> Result<(String, usize)> {
    // Find null terminator
    let null_pos = data[offset..]
        .iter()
        .position(|&b| b == 0)
        .ok_or("Unterminated string")?;

    let bytes = &data[offset..offset + null_pos];
    let string = String::from_utf8_lossy(bytes).into_owned();

    Ok((string, null_pos + 1))  // +1 for null byte
}
```

## Canon AF Data Example

Implementation of Canon's variable AF data:

```rust
// src/implementations/canon.rs

pub fn process_canon_af_info2(
    reader: &mut ExifReader,
    data: &[u8],
) -> Result<()> {
    // ExifTool: Canon.pm:6520 AFInfo2

    let mut offset = 0;

    // Tag 2: NumAFPoints (DataMember)
    let num_points = u16::from_be_bytes([data[offset + 4], data[offset + 5]]);
    reader.data_members.insert("NumAFPoints", num_points as usize);
    reader.add_tag("Canon:NumAFPoints", TagValue::Integer(num_points as i64));

    // Skip to variable arrays
    offset = 8;

    // Tag 8: AFAreaWidths (variable array)
    if num_points > 0 {
        let mut widths = Vec::new();
        for _ in 0..num_points {
            let width = u16::from_be_bytes([data[offset], data[offset + 1]]);
            widths.push(width);
            offset += 2;
        }
        reader.add_tag("Canon:AFAreaWidths", TagValue::Array(
            widths.into_iter()
                .map(|w| TagValue::Integer(w as i64))
                .collect()
        ));
    }

    // Continue with other variable arrays...

    Ok(())
}
```

## Success Criteria

- [ ] Variable-length string formats extract correctly
- [ ] DataMember values properly stored and retrieved
- [ ] Canon AF arrays sized dynamically based on NumAFPoints
- [ ] Two-phase extraction maintains correct order
- [ ] Format expressions evaluated correctly

## Testing Strategy

1. **Unit Tests**: Test format resolution and DataMember storage
2. **Integration Tests**: Test with real Canon files containing AF data
3. **Edge Cases**: Zero-length arrays, missing DataMembers
4. **Compatibility**: Compare with ExifTool output

## Implementation Phases

### Week 1: Core Infrastructure

- DataMember storage system
- Format expression parser
- Two-phase extraction logic

### Week 2: Format Implementations

- Variable string formats
- Array size expressions
- Canon AF data specifics

### Week 3: Testing & Polish

- Comprehensive test coverage
- Performance optimization
- Error handling improvements

## Related Documentation

- [STATE-MANAGEMENT.md](../STATE-MANAGEMENT.md) - DataMember storage design
- [PROCESS_PROC.md](../../third-party/exiftool/doc/concepts/PROCESS_PROC.md) - ProcessBinaryData details
- [Canon.md](../../third-party/exiftool/doc/modules/Canon.md) - Canon AF data structures
