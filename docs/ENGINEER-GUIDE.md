# Engineer's Guide to exif-oxide

This guide helps new engineers understand the exif-oxide project and start contributing effectively. Read this after understanding the high-level ARCHITECTURE.md.

## Essential Background

### What is ExifTool?

ExifTool is a 25-year-old Perl library that reads/writes metadata from image, audio, and video files. It's the de facto standard because it handles thousands of proprietary formats and manufacturer quirks that have accumulated over decades of digital photography.

### Why Translation, Not Innovation?

Every line of ExifTool code exists for a reason - usually to work around a specific camera's bug or non-standard behavior. We must resist the temptation to "improve" or "simplify" the logic. If ExifTool checks for value 0x41 before 0x42, there's a camera somewhere that depends on that order.

## Critical ExifTool Concepts

Before diving into code, read these essential ExifTool documentation files:

- [PROCESS_PROC.md](../third-party/exiftool/doc/concepts/PROCESS_PROC.md) - Processing procedures explained
- [VALUE_CONV.md](../third-party/exiftool/doc/concepts/VALUE_CONV.md) - Value conversion system
- [PRINT_CONV.md](../third-party/exiftool/doc/concepts/PRINT_CONV.md) - Human-readable conversions
- [BINARY_TAGS.md](../third-party/exiftool/doc/concepts/BINARY_TAGS.md) - Binary data extraction
- [MAKERNOTE.md](../third-party/exiftool/doc/concepts/MAKERNOTE.md) - Manufacturer-specific data

### 1. PROCESS_PROC - The Heart of Everything

```perl
# In ExifTool tables
PROCESS_PROC => \&ProcessCanon,  # Function reference
PROCESS_PROC => 'ProcessBinaryData',  # String name
```

PROCESS_PROC tells ExifTool how to parse a block of data. The most common is `ProcessBinaryData` (used 121+ times), which extracts fixed-offset binary structures. Understanding this is crucial.

### 2. Tag Tables - Not Just Data

ExifTool tag tables look like simple hashes but contain code. See [MODULE_OVERVIEW.md](../third-party/exiftool/doc/concepts/MODULE_OVERVIEW.md) for the big picture:

```perl
0x0112 => {
    Name => 'Orientation',
    PrintConv => {
        1 => 'Horizontal (normal)',
        2 => 'Mirror horizontal',
        # ...
    },
    ValueConv => '$val > 8 ? undef : $val',  # Perl code!
}
```

### 3. The Conversion Pipeline

```
Raw Bytes → Format Parsing → ValueConv → PrintConv → Display
         ↑                 ↑           ↑
    (binary)        (logical value) (human readable)
```

- **ValueConv**: Converts raw data to logical values (e.g., APEX to f-stop)
- **PrintConv**: Converts logical values to human-readable strings

### 4. MakerNotes - The Wild West

Each camera manufacturer has proprietary data formats in MakerNotes. These often:

- Use different byte orders than the main file
- Have encrypted sections
- Calculate offsets differently
- Change format between firmware versions

### 5. Offset Calculations - The Biggest Gotcha

ExifTool uses this formula everywhere:

```
absolute_position = base + data_pos + relative_offset
```

But each manufacturer interprets this differently. Canon might use the TIFF header as base, Nikon might use the MakerNote start, Sony might use something else entirely.

For deep understanding of offsets, see:

- [OFFSET-BASE-MANAGEMENT.md](OFFSET-BASE-MANAGEMENT.md) - Our offset management strategy
- [READING_AND_PARSING.md](../third-party/exiftool/doc/concepts/READING_AND_PARSING.md) - ExifTool's approach

## Reading ExifTool Source Code

### Essential Files to Understand

1. **lib/Image/ExifTool.pm** - Core API and state management
2. **lib/Image/ExifTool/Exif.pm** - EXIF/TIFF parsing (study ProcessExif) - see [Exif.md](../third-party/exiftool/doc/modules/Exif.md)
3. **lib/Image/ExifTool/Canon.pm** - Good example of manufacturer complexity - see [Canon.md](../third-party/exiftool/doc/modules/Canon.md)
4. **lib/Image/ExifTool/README** - Documents all special table keys

Also study the module documentation:

- [Nikon.md](../third-party/exiftool/doc/modules/Nikon.md) - Complex encryption and versions
- [Sony.md](../third-party/exiftool/doc/modules/Sony.md) - Another encryption approach
- [MakerNotes.md](../third-party/exiftool/doc/modules/MakerNotes.md) - Central dispatcher

### How to Read Tag Tables

```perl
%Image::ExifTool::Canon::CameraSettings = (
    PROCESS_PROC => \&ProcessBinaryData,
    FIRST_ENTRY => 1,
    FORMAT => 'int16s',
    GROUPS => { 0 => 'MakerNotes', 2 => 'Camera' },
    1 => {
        Name => 'MacroMode',
        PrintConv => {
            1 => 'Macro',
            2 => 'Normal',
        },
    },
    # ... hundreds more
);
```

Key points:

- `PROCESS_PROC` determines how to parse
- `FORMAT` sets default for all tags
- `FIRST_ENTRY` means array is 1-indexed (not 0)
- `GROUPS` affects how tags are categorized

### Common Perl Patterns to Recognize

```perl
# Conditional value
ValueConv => '$val =~ /^\\d+$/ ? $val : undef',

# Bit extraction
ValueConv => '($val >> 4) & 0x0f',

# Multiple values
ValueConv => '[ split " ", $val ]',

# Reference to previously extracted value
Format => 'string[$val{3}]',  # Length from tag 3

# DataMember - tag needed by other tags
DataMember => 'CameraType',
RawConv => '$$self{CameraType} = $val',
```

## Getting Started on Milestone 0

First read [MILESTONES.md](MILESTONES.md) for the full roadmap, then [STATE-MANAGEMENT.md](STATE-MANAGEMENT.md) for stateful processing details.

### Step 1: Set Up the Perl Extractor

The Perl extractor is minimal - it just dumps ExifTool's tables to JSON:

```perl
# codegen/extract_tables.pl
use Image::ExifTool;
use JSON;

# Load tables
require Image::ExifTool::Exif;

# Extract and filter mainstream tags
my $metadata = load_json('TagMetadata.json');
my @tags;

foreach my $id (keys %Image::ExifTool::Exif::Main) {
    my $tag = $Image::ExifTool::Exif::Main{$id};
    next unless is_mainstream($tag, $metadata);

    push @tags, {
        id => $id,
        name => $$tag{Name},
        format => $$tag{Format},
        print_conv => ref($$tag{PrintConv}) ? undef : $$tag{PrintConv},
        # ... other fields
    };
}

print encode_json(\@tags);
```

### Step 2: Understand the Registry Pattern

Instead of generating thousands of stub functions, we use runtime lookup. This approach is explained in [ARCHITECTURE.md](../ARCHITECTURE.md#todo-tracking-system):

```rust
// implementations/registry.rs
lazy_static! {
    static ref PRINT_CONV: HashMap<&'static str, PrintConvFn> = HashMap::new();
}

pub fn register_print_conv(name: &'static str, func: PrintConvFn) {
    PRINT_CONV.insert(name, func);
}

pub fn get_print_conv(name: &str) -> Option<PrintConvFn> {
    PRINT_CONV.get(name).copied()
}
```

### Step 3: Implement Your First PrintConv

Start with Orientation - it's simple and very common:

```rust
// implementations/print_conv/orientation.rs

/// EXIF Orientation
/// ExifTool: lib/Image/ExifTool/Exif.pm:2719
pub fn orientation(val: &TagValue) -> String {
    match val.as_u16() {
        Some(1) => "Horizontal (normal)",
        Some(2) => "Mirror horizontal",
        Some(3) => "Rotate 180",
        Some(4) => "Mirror vertical",
        Some(5) => "Mirror horizontal and rotate 270 CW",
        Some(6) => "Rotate 90 CW",
        Some(7) => "Mirror horizontal and rotate 90 CW",
        Some(8) => "Rotate 270 CW",
        _ => return format!("Unknown ({})", val),
    }.to_string()
}

// Register it
pub fn register() {
    register_print_conv("exif_orientation", orientation);
}
```

## Common Pitfalls and Solutions

For additional context on these pitfalls, see [PATTERNS.md](../third-party/exiftool/doc/concepts/PATTERNS.md).

### 1. Endianness Confusion

ExifTool tracks byte order per directory. A Canon file might be little-endian overall but have big-endian values in certain maker note sections.

**Solution**: Always use the ExifReader's current byte order, not the file's global order.

### 2. Offset Base Confusion

When you see `SubDirectory => { Start => '$val' }`, the base for that subdirectory is NOT obvious. It could be relative to:

- The TIFF header
- The current directory start
- The tag's position
- Something manufacturer-specific

**Solution**: Study ProcessExif carefully. When in doubt, add debug logging to track actual vs expected positions.

### 3. Format Strings

ExifTool format strings can be dynamic:

```perl
Format => 'string[$val{3}]'  # Length from tag 3
Format => 'var_string'        # Variable null-terminated
```

**Solution**: Start with fixed formats only. Add variable format support incrementally.

### 4. PrintConv Complexity

Some PrintConv functions are simple lookups, others are complex:

```perl
PrintConv => 'sprintf("%.1f", $val / 10)',  # Simple
PrintConv => \&CanonEv,  # Complex function with special cases
```

**Solution**: Implement only what you need. Use --show-missing to prioritize.

For more on conversions, see:

- [VALUE_CONV.md](../third-party/exiftool/doc/concepts/VALUE_CONV.md) - Value conversion details
- [PRINT_CONV.md](../third-party/exiftool/doc/concepts/PRINT_CONV.md) - Print conversion patterns

## Debugging Tips

### 1. Use ExifTool for Ground Truth

```bash
# See exactly what ExifTool extracts
exiftool -v3 image.jpg > exiftool_verbose.txt

# Get JSON output for comparison
exiftool -j image.jpg > expected.json

# See hex dump of specific tag
exiftool -htmlDump image.jpg > dump.html
```

### 2. Add Trace Logging

```rust
use log::trace;

trace!("ProcessExif: offset={:#x}, entries={}", offset, count);
trace!("Tag {:04x}: raw_value={:?}", tag_id, raw_value);
```

### 3. Verify Offsets

When implementing a new format, always verify your offset calculations:

```rust
assert_eq!(
    calculated_offset,
    expected_offset,
    "Offset mismatch for tag {}: calculated {:#x} != expected {:#x}",
    tag_name, calculated_offset, expected_offset
);
```

### 4. Test Incrementally

Don't try to parse an entire Canon maker note at once. Start with:

1. Detecting it exists
2. Reading the header
3. Parsing one simple tag
4. Adding more tags
5. Handling special cases

## Tribal Knowledge

### "Magic" Constants

When you see unexplained constants in ExifTool:

```perl
$offset += 0x1a;  # What is 0x1a?
```

These are usually manufacturer-specific quirks discovered through reverse engineering. Document them but don't change them:

```rust
// Add 0x1a to offset for Canon 5D
// ExifTool: Canon.pm:1234 (no explanation given)
offset += 0x1a;
```

### The GE Camera Mystery

ExifTool has this comment:

```perl
# GE cameras need a 210 byte offset (why?)
```

Nobody knows why. Just implement it as-is.

### Double-UTF8 Encoding

Some cameras encode UTF-8 strings twice. ExifTool silently fixes this. You should too.

For character encoding complexities, see [CHARSETS.md](../third-party/exiftool/doc/concepts/CHARSETS.md):

```rust
// Some Sony cameras double-encode UTF-8
// ExifTool: XMP.pm:4567
if looks_like_double_utf8(&string) {
    string = decode_utf8_twice(string);
}
```

### Test Image Gold Mine

The ExifTool test suite (t/images/) contains problematic files from real cameras. When implementing a manufacturer's support, always test against their files in t/images/.

## Git Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Common types:**

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code restructuring without behavior change
- `test:` - Adding/updating tests
- `chore:` - Maintenance tasks

**Examples:**

```
feat(parser): add Canon MakerNote support
fix(exif): handle invalid orientation values
docs: update MILESTONES.md for v0.2
```

Use the most impacted filename as scope. Be concise - summarize the change's purpose, not every diff line.

## Generated Code Policy

**Generated Rust code is committed to git** while intermediate files are ignored:

- **Commit**: Final Rust code in `src/generated/` (tags.rs, conversion_refs.rs, etc.)
- **Ignore**: Intermediate files in `codegen/generated/` (tag_tables.json, etc.)

**Rationale**: This ensures developers can build without requiring Perl + ExifTool while keeping the repository manageable. Generated code is relatively stable and benefits from code review visibility.

**When to regenerate**:
- After modifying extraction scripts (`codegen/extract_tables.pl`)
- After updating ExifTool version  
- When adding new tag implementations to MILESTONE_COMPLETIONS
- Run: `make codegen` then commit the updated `src/generated/` files

## Getting Help

1. **Read the ExifTool source** - The answer is usually there
2. **Check module docs** - See [third-party/exiftool/doc/modules/](../third-party/exiftool/doc/modules/) for specific formats
3. **Review concepts** - [third-party/exiftool/doc/concepts/](../third-party/exiftool/doc/concepts/) explains patterns
4. **Check ExifTool forums** - Many quirks are discussed there
5. **Use --show-missing** - Let it guide your implementation priority
6. **Start small** - One tag at a time, one format at a time

Key documentation files:

- [FILE_TYPES.md](../third-party/exiftool/doc/concepts/FILE_TYPES.md) - File format detection
- [WRITE_PROC.md](../third-party/exiftool/doc/concepts/WRITE_PROC.md) - Writing (future milestone)
- [PROCESSOR-PROC-DISPATCH.md](PROCESSOR-PROC-DISPATCH.md) - Our dispatch strategy

## Remember

- ExifTool compatibility is the #1 priority
- Don't innovate, translate
- Every quirk has a reason
- Test against real images
- Document ExifTool source references

Happy coding! Remember: if it seems weird, it's probably correct. Cameras are weird.
