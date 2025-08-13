# SubDirectory Processing Gap Analysis

## Critical Finding: We're Not Following ExifTool's Architecture

### ExifTool's SubDirectory Processing

ExifTool's subdirectory system (documented in `SUBDIRECTORY_SYSTEM.md`) is a **recursive metadata architecture** that:

1. **Identifies subdirectory tags** through the `SubDirectory` hash in tag definitions
2. **Evaluates Condition fields** at runtime using Perl's `eval` with `$self` context
3. **Processes subdirectories recursively** via `ProcessDirectory` calls
4. **Applies PrintConv AFTER extraction** as part of the tag value flow

Key insights from ExifTool source (`ExifTool.pm:10701-10728`):
```perl
sub GetTagInfo($$$;$$$) {
    # ...
    foreach $tagInfo (@infoArray) {
        my $condition = $$tagInfo{Condition};
        if ($condition) {
            # Evaluates condition with $self in scope
            #### eval Condition ($self, [$valPt, $format, $count])
            unless ( eval $condition ) {
                next;  # Skip this tag variant if condition fails
            }
        }
        return $tagInfo;  # Return first matching variant
    }
}
```

### Our Current Implementation Issues

Our `subdirectory_processing.rs` module has fundamental architectural mismatches:

1. **Generic Approach Instead of Manufacturer-Specific**
   - We're using generic functions passed as parameters
   - ExifTool has manufacturer-specific modules with their own logic
   
2. **Missing Condition Evaluation**
   - We have `ExpressionEvaluator` but aren't using it for subdirectory selection
   - ExifTool evaluates Conditions to pick the right subdirectory variant (e.g., Canon tag 0xf has 50+ variants based on camera model)

3. **Wrong Processing Flow**
   - We're trying to apply PrintConv during subdirectory processing
   - ExifTool applies PrintConv AFTER tags are extracted, as part of normal tag value flow

4. **Incorrect Tag Table Selection**
   - We're not properly selecting which TagTable to use based on Conditions
   - Example: Canon tag 0xf can be CustomFunctions1D, CustomFunctions5D, CustomFunctions10D, etc., based on Model

## Specific Example: Canon Tag 0xf (CustomFunctions)

### ExifTool Implementation (Canon.pm:1553-1600+)

```perl
0xf => [
    {
        Name => 'CustomFunctions1D',
        Condition => '$$self{Model} =~ /EOS-1D/',
        SubDirectory => {
            Validate => 'Image::ExifTool::Canon::Validate($dirData,$subdirStart,$size)',
            TagTable => 'Image::ExifTool::CanonCustom::Functions1D',
        },
    },
    {
        Name => 'CustomFunctions5D', 
        Condition => '$$self{Model} =~ /EOS 5D/',
        SubDirectory => {
            Validate => 'Image::ExifTool::Canon::Validate($dirData,$subdirStart,$size)',
            TagTable => 'Image::ExifTool::CanonCustom::Functions5D',
        },
    },
    # ... 50+ more variants for different camera models
]
```

### What We're Missing

1. **Model-based dispatch**: Need to check camera model and select appropriate tag table
2. **Condition evaluation**: Must evaluate `$$self{Model} =~ /pattern/` expressions
3. **Table switching**: Need to load different tag tables (Functions1D vs Functions5D)
4. **Validation functions**: Need manufacturer-specific validation

## ProcessBinaryData vs SubDirectory

We're conflating two different ExifTool concepts:

### ProcessBinaryData (ExifTool.pm:11514+)
- Extracts values from binary data at specific offsets
- Uses FORMAT (int8u, int16s, etc.) to interpret bytes
- Creates tags from binary positions
- Example: Canon CameraSettings is binary data

### SubDirectory Processing (Exif.pm:6807-6999)
- Recursively processes nested metadata structures
- Changes tag table context
- Can have different byte order
- Example: ExifIFD, GPS, MakerNotes are subdirectories

## Required Changes

### 1. Separate SubDirectory from BinaryData Processing

- SubDirectory processing should be about **recursive directory processing**
- BinaryData processing should be about **extracting values from binary arrays**
- These are orthogonal concerns that sometimes combine (binary data can contain subdirectories)

### 2. Implement Proper Condition Evaluation

- Use our `ExpressionEvaluator` to evaluate Conditions
- Pass camera model, manufacturer, and other context
- Select the correct tag variant based on conditions

### 3. Follow ExifTool's Processing Flow

```
1. Identify tag with SubDirectory definition
2. Evaluate Conditions to select variant
3. Extract/validate subdirectory data
4. Call ProcessDirectory recursively with new TagTable
5. Store extracted tags (PrintConv happens later in display)
```

### 4. Manufacturer-Specific Implementations

Instead of generic functions, each manufacturer needs:
- `has_subdirectory(tag_id) -> Option<Vec<SubDirectoryDef>>`
- `evaluate_conditions(defs, context) -> Option<SubDirectoryDef>`
- `process_subdirectory(def, data, context) -> Result<Tags>`

## Next Steps

1. **Refactor subdirectory_processing.rs** to match ExifTool's architecture
2. **Implement Condition evaluation** for tag variant selection
3. **Create manufacturer-specific subdirectory modules** 
4. **Separate PrintConv application** from subdirectory processing
5. **Add proper TagTable switching** based on SubDirectory definitions