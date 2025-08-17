# Binary Pattern Integration with AST Pipeline

## Problem
PPI AST parsing fails on expressions with binary data because:
1. `$$valPt=~/\x00\x46\x00/` contains non-UTF-8 sequences
2. JSON corruption breaks binary data round-trips
3. AST parser expects clean Perl syntax

## Solution: Pre-process in Field Extractor

**Handle binary patterns in `field_extractor.pl` BEFORE AST parsing**

### Implementation

#### Step 1: Add Binary Detection to Field Extractor

```perl
# In field_extractor.pl, before try_parse_expression_with_ppi()

sub preprocess_binary_patterns {
    my ($expr_string) = @_;
    
    my @binary_components = ();
    my $clean_expr = $expr_string;
    my $placeholder_counter = 0;
    
    # Extract and replace hex escapes: \x##
    while ($clean_expr =~ s/\\x([0-9a-fA-F]{2})/BINARY_PLACEHOLDER_$placeholder_counter/g) {
        push @binary_components, {
            type => "hex_escape",
            original => "\\x$1",
            raw_bytes => [hex($1)],
            placeholder => "BINARY_PLACEHOLDER_$placeholder_counter"
        };
        $placeholder_counter++;
    }
    
    # Extract and replace null bytes: \0
    while ($clean_expr =~ s/\\0/BINARY_PLACEHOLDER_$placeholder_counter/g) {
        push @binary_components, {
            type => "null_byte", 
            original => "\\0",
            raw_bytes => [0],
            placeholder => "BINARY_PLACEHOLDER_$placeholder_counter"
        };
        $placeholder_counter++;
    }
    
    return {
        clean_expression => $clean_expr,
        binary_components => \@binary_components,
        has_binary => scalar(@binary_components) > 0
    };
}
```

#### Step 2: Integrate with AST Processing

```perl
# Modify try_parse_expression_with_ppi()

sub try_parse_expression_with_ppi {
    my ( $expr_string, $context, $analysis ) = @_;
    
    # PRE-PROCESS: Extract binary patterns
    my $preprocessing = preprocess_binary_patterns($expr_string);
    
    # Skip very long expressions (likely multi-line code blocks)
    return 0 if length($preprocessing->{clean_expression}) > 500;
    
    # Try to parse the CLEAN expression (binary data removed)
    eval {
        my $document = PPI::Document->new(\$preprocessing->{clean_expression});
        
        if ($document) {
            $ast_processed_expressions++;
            
            # Analyze the parsed AST
            my $ast_info = analyze_ppi_document($document, $preprocessing->{clean_expression});
            
            # ENHANCEMENT: Add binary data info to AST result
            if ($preprocessing->{has_binary}) {
                $ast_info->{binary_components} = $preprocessing->{binary_components};
                $ast_info->{original_with_binary} = $expr_string;
                $ast_info->{has_binary_data} = 1;
            }
            
            # Store enhanced AST data
            $analysis->{ppi_ast_data}->{$context} = $ast_info;
            
            return 1;
        }
        # ... rest of function
    };
}
```

#### Step 3: Enhance AstStrategy to Handle Binary Data

```rust
// In ast_strategy.rs

impl AstStrategy {
    fn can_handle_expression_flags(&self, ast_info: &AstInfo) -> bool {
        // Must be parseable by PPI
        if !ast_info.parseable {
            return false;
        }
        
        // NEW: Check if expression has binary components
        if ast_info.has_binary_data.unwrap_or(false) {
            // We can handle binary regex patterns
            if ast_info.has_regex && ast_info.binary_components.is_some() {
                return true;
            }
            
            // For other binary patterns, route to specialized binary strategy
            return false;  
        }
        
        // ... existing logic
    }
    
    fn process_expression_with_binary(&self, ast_info: &AstInfo) -> Result<String> {
        let mut rust_code = String::new();
        
        // Get the clean expression and binary components
        let clean_expr = &ast_info.clean_expression;
        let binary_components = ast_info.binary_components.as_ref().unwrap();
        
        // Generate Rust code that reconstructs the binary pattern
        rust_code.push_str("// Generated from expression with binary data\n");
        
        for component in binary_components {
            let bytes_str = component.raw_bytes.iter()
                .map(|b| format!("0x{:02x}", b))
                .collect::<Vec<_>>()
                .join(", ");
                
            rust_code.push_str(&format!(
                "static {}: &[u8] = &[{}];\n", 
                component.placeholder.to_uppercase(),
                bytes_str
            ));
        }
        
        // Generate the actual matching logic
        // Convert regex pattern to Rust regex with binary data
        // ... implementation details
        
        Ok(rust_code)
    }
}
```

#### Step 4: Update AST Types

```rust
// In ast/src/ppi_types.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstInfo {
    pub original: String,
    pub parseable: bool,
    
    // NEW: Binary data support
    pub has_binary_data: Option<bool>,
    pub binary_components: Option<Vec<BinaryComponent>>,
    pub clean_expression: Option<String>,      // Expression with binary data removed
    pub original_with_binary: Option<String>, // Original expression with binary data
    
    // Existing fields...
    pub has_variables: bool,
    pub has_operators: bool,
    pub has_functions: bool,
    pub has_self_refs: bool,
    pub has_regex: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryComponent {
    pub component_type: String,  // "hex_escape", "null_byte"
    pub original: String,        // "\\x01"
    pub raw_bytes: Vec<u8>,      // [1]
    pub placeholder: String,     // "BINARY_PLACEHOLDER_0"
}
```

## Benefits of This Approach

1. **✅ AST parsing works**: Clean expressions parse successfully with PPI
2. **✅ Binary data preserved**: Raw bytes stored separately as arrays  
3. **✅ Clean integration**: Builds on existing AST pipeline
4. **✅ Flexible routing**: Can route complex binary expressions to specialized strategies
5. **✅ Debuggable**: Clear separation between clean syntax and binary data

## Alternative: Separate Binary Strategy

If binary expressions are too complex for AST handling, create a separate `BinaryExpressionStrategy`:

```rust
pub struct BinaryExpressionStrategy {
    // Handle expressions that contain binary data
    // Route from AstStrategy when has_binary_data = true
}
```

## Recommendation

**Use the preprocessing approach** - it's the cleanest integration that preserves the AST pipeline while handling binary data robustly.