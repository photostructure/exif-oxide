# State Management Design for exif-oxide

## Research Summary

After extensive analysis of ExifTool's source code, I've identified the critical state management patterns and can now provide architectural recommendations for the Rust implementation.

## Key State Components in ExifTool (Research Findings)

### 1. PROCESSED Hash - Infinite Loop Prevention

- **Location**: `lib/Image/ExifTool.pm:4279, 8964-8970`
- **Purpose**: Prevents infinite loops in circular directory references
- **Implementation**:

  ```perl
  $$self{PROCESSED} = { };  # init
  my $addr = $$dirInfo{DirStart} + $$dirInfo{DataPos} + ($$dirInfo{Base}||0) + $$self{BASE};
  $$self{PROCESSED}{$addr} = $dirName unless $$tagTablePtr{VARS}{ALLOW_REPROCESS};
  ```

- **Address calculation**: Combines DirStart + DataPos + Base + global BASE offset
- **Critical for**: Maker notes that reference other sections

### 2. VALUE Hash - Extracted Tag Storage

- **Location**: `lib/Image/ExifTool.pm:4273, 9356, 9527-9530`
- **Purpose**: Stores all extracted tag values indexed by tag key
- **Implementation**:

  ```perl
  $$self{VALUE} = { };  # initialization
  $$self{VALUE}{$vtag} = $$self{VALUE}{$tag};  # duplicate handling
  ```

- **Features**: Supports duplicate tag handling, deletion, and metadata association

### 3. DataMember Dependencies - Complex Interdependencies

- **Location**: Multiple locations, key processing in ProcessBinaryData and ProcessSerialData
- **Purpose**: Earlier tags determine format/count/behavior of later tags
- **Examples**:
  - Canon AF data: `NumAFPoints` determines array sizes for AF positions
  - Format expressions: `int16s[$val{0}]` where `$val{0}` is from tag 0
  - Conditional extraction: Tags only extracted if dependency conditions met
- **Resolution Strategy**: Sequential processing with `%val` hash accumulating values

### 4. Directory Processing Context - Nested State Management

- **Location**: `lib/Image/ExifTool.pm:4287, 8977, 8985, 7159-7161`
- **Components**:
  - **PATH stack**: `$$self{PATH}` tracks current directory hierarchy
  - **Directory Info**: Hash with Base, DataPos, DirStart, DirLen
  - **Base calculations**: Complex offset arithmetic for nested structures
- **State transitions**: Push/pop on directory entry/exit

## Architectural Recommendations for Rust Implementation

### 1. Stateful Reader Object Pattern ✅

**Recommendation**: Use a stateful `ExifReader` object similar to ExifTool's `$self`

```rust
pub struct ExifReader {
    // Core state - equivalent to ExifTool's member variables
    processed: HashMap<u64, String>,                    // PROCESSED hash
    values: HashMap<String, TagValue>,                  // VALUE hash
    data_members: HashMap<String, DataMemberValue>,     // DataMember storage

    // Processing context
    path: Vec<String>,                                  // PATH stack
    base: u64,                                         // Current base offset
    byte_order: ByteOrder,                             // Current byte order

    // File-level state
    file_type: String,                                 // FILE_TYPE
    options: ReadOptions,                              // Processing options
}
```

**Rationale**:

- ExifTool's stateful approach is proven for complex nested metadata
- Rust's ownership system provides memory safety without sacrificing the pattern
- Natural fit for the complex interdependencies in metadata processing

### 2. DataMember Resolution Strategy

**Recommendation**: Sequential Processing with Dependency Tracking

```rust
impl ExifReader {
    fn process_binary_data(&mut self, data: &[u8], table: &TagTable) -> Result<()> {
        // Phase 1: Extract DataMember tags first if needed
        if let Some(data_members) = &table.data_members {
            for &tag_id in data_members {
                if let Some(tag_info) = table.tags.get(&tag_id) {
                    let value = self.extract_tag_value(data, tag_id, tag_info)?;
                    if let Some(member_name) = &tag_info.data_member {
                        self.data_members.insert(member_name.clone(), value.clone());
                    }
                }
            }
        }

        // Phase 2: Process remaining tags with DataMember context available
        for (&tag_id, tag_info) in &table.tags {
            let format = self.resolve_format(&tag_info.format)?;
            let value = self.extract_with_format(data, tag_id, &format)?;
            self.values.insert(tag_info.name.clone(), value);
        }

        Ok(())
    }

    fn resolve_format(&self, format_expr: &str) -> Result<ResolvedFormat> {
        // Handle expressions like "int16s[$val{0}]"
        if format_expr.contains("$val{") {
            // Parse and substitute from data_members
            self.substitute_data_member_refs(format_expr)
        } else {
            Ok(ResolvedFormat::parse(format_expr)?)
        }
    }
}
```

**Key Features**:

- **Two-phase processing**: DataMembers first, then dependent tags
- **Expression evaluation**: Safe parsing of `$val{N}` expressions
- **Forward compatibility**: Can handle ExifTool's dependency patterns

### 3. State Isolation Strategy

**Recommendation**: Shared Mutable State with Controlled Access

```rust
impl ExifReader {
    fn process_subdirectory(&mut self, dir_info: &DirectoryInfo) -> Result<()> {
        // Calculate unique address for recursion prevention
        let addr = self.calculate_directory_address(dir_info);

        // Check for infinite loops
        if let Some(prev_dir) = self.processed.get(&addr) {
            return Err(ExifError::circular_reference(prev_dir, &dir_info.name));
        }

        // Enter subdirectory context
        self.path.push(dir_info.name.clone());
        self.processed.insert(addr, dir_info.name.clone());

        // Process with current context
        let result = self.process_directory_contents(dir_info);

        // Exit subdirectory context
        self.path.pop();
        if !dir_info.allow_reprocess {
            // Keep PROCESSED entry for recursion prevention
        }

        result
    }

    fn calculate_directory_address(&self, dir_info: &DirectoryInfo) -> u64 {
        // Equivalent to ExifTool's addr calculation
        dir_info.dir_start + dir_info.data_pos + dir_info.base + self.base
    }
}
```

**Benefits**:

- **Compatible behavior**: Matches ExifTool's recursion prevention
- **Context preservation**: PATH and offset state properly managed
- **Memory safety**: Rust ownership prevents dangling references

### 4. Thread Safety Approach

**Recommendation**: Single-threaded per Reader, Thread-safe for Multiple Readers

```rust
// ExifReader is NOT Send/Sync - single-threaded use only
impl ExifReader {
    pub fn new() -> Self { /* ... */ }

    pub fn read_metadata(&mut self, data: &[u8]) -> Result<Metadata> {
        self.reset_state();  // Clear previous file's state
        self.process_file(data)
    }
}

// For multi-threading, create separate readers
pub fn process_files_parallel(files: &[PathBuf]) -> Vec<Result<Metadata>> {
    files.par_iter().map(|path| {
        let mut reader = ExifReader::new();  // Each thread gets own reader
        let data = std::fs::read(path)?;
        reader.read_metadata(&data)
    }).collect()
}
```

**Design Rationale**:

- **Simplicity**: Avoids complex synchronization around mutable state
- **Performance**: No locking overhead within single reader
- **Scalability**: Multiple readers can run in parallel on different threads
- **Compatibility**: Matches ExifTool's single-threaded processing model

## Implementation Priority

1. **Phase 1**: Implement basic stateful reader with PROCESSED hash
2. **Phase 2**: Add VALUE hash and directory context management
3. **Phase 3**: Implement DataMember dependency resolution
4. **Phase 4**: Add comprehensive error handling and edge cases

## Compatibility Benefits

This architecture maintains behavioral compatibility with ExifTool while leveraging Rust's safety features:

- **Memory safety**: No risk of dangling pointers or use-after-free
- **Error handling**: Explicit Result types vs Perl's undefined behavior
- **Performance**: Zero-cost abstractions vs Perl's runtime overhead
- **Maintainability**: Strong typing vs Perl's dynamic typing
