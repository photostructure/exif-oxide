# Array Validation Scripts

## validate_arrays.pl

**Purpose**: Validates that generated Rust static arrays exactly match the original ExifTool perl arrays.

This is crucial for cryptographic accuracy - a single wrong byte can break decryption algorithms.

### Usage

```bash
# Validate all arrays in a config
perl scripts/validate_arrays.pl config/Nikon_pm/simple_array.json

# Validate with verbose output
perl scripts/validate_arrays.pl --verbose config/Nikon_pm/simple_array.json

# Validate only a specific array
perl scripts/validate_arrays.pl --array "xlat[0]" config/Nikon_pm/simple_array.json
```

### What it does

1. **Loads ExifTool**: Patches the original Nikon.pm module to access `my` variables as `our` variables
2. **Extracts perl arrays**: Uses the same patching system as our codegen to access arrays like `xlat[0]`, `xlat[1]`
3. **Parses Rust arrays**: Extracts values from generated `.rs` files like `xlat_0.rs`, `xlat_1.rs`
4. **Compares element-by-element**: Validates every single byte matches between perl and Rust
5. **Reports results**: Shows pass/fail for each array with detailed error messages

### Exit codes
- `0`: All arrays validated successfully
- `1`: One or more arrays failed validation

### Example output

```
🔍 Validating arrays from third-party/exiftool/lib/Image/ExifTool/Nikon.pm
📋 Found 2 array(s) to validate

🔧 Validating xlat[0] -> XLAT_0
✅ xlat[0] passed validation (all 256 elements match)

🔧 Validating xlat[1] -> XLAT_1  
✅ xlat[1] passed validation (all 256 elements match)

📋 Validation Summary:
   Total arrays: 2
   ✅ Passed: 2
   ❌ Failed: 0

🎉 ALL ARRAYS VALIDATED SUCCESSFULLY!
```

### Requirements
- Perl 5.x with JSON module
- ExifTool source at `../../third-party/exiftool/lib/`  
- Generated Rust files at `../../src/generated/Nikon_pm/`

## Integration Tests

The simple array extraction pipeline also includes comprehensive Rust integration tests:

```bash
# Run all simple array integration tests
cargo test --test simple_array_integration

# Run just the ExifTool validation test
cargo test --test simple_array_integration test_nikon_xlat_arrays_match_exiftool
```

These tests validate:
- **ExifTool accuracy**: Arrays exactly match original perl arrays using the validation script
- **Key values**: First, middle, and last bytes match expected values (e.g., XLAT_0[0]=193, XLAT_0[127]=47, XLAT_0[255]=199)
- **Runtime access**: Arrays are accessible with both bounds-checked and direct access patterns
- **Cryptographic properties**: Arrays have good distribution and span the full u8 range (important for decryption)