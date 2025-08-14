# PPI Token Processing for Rust Code Generation

This document explains how to process PPI (Perl Parsing Interface) tokens from our field extractor for Rust code generation.

## Overview

Our simplified `field_extractor.pl` uses `PPI::Simple` to parse Perl expressions from ExifTool modules into structured JSON. These JSON structures represent the Abstract Syntax Tree (AST) of Perl expressions, which we then convert to equivalent Rust code.

## JSON Structure

Each PPI token in JSON has this structure:

```json
{
  "class": "PPI::Token::Symbol",
  "content": "$$self{Make}",
  "symbol_type": "scalar",
  "children": [...]
}
```

**Key fields:**
- `class`: The PPI token type (e.g., `PPI::Token::Symbol`, `PPI::Token::Operator`)
- `content`: The raw Perl text (for atomic tokens)
- `children`: Array of child tokens (for container nodes)
- Type-specific fields (e.g., `symbol_type`, `numeric_value`, `string_value`)

## Common PPI Token Types

### Variables: `PPI::Token::Symbol`

**Perl:** `$val`, `$$self{Make}`, `@array`, `%hash`

```json
{
  "class": "PPI::Token::Symbol", 
  "content": "$$self{Make}",
  "symbol_type": "scalar"
}
```

**Rust generation:**
- `$val` → `val`
- `$$self{Make}` → `self_ref.get("Make")?`
- `@array` → `array` (slice)
- `%hash` → `hash` (HashMap)

### Operators: `PPI::Token::Operator`

**Perl:** `+`, `-`, `eq`, `=~`, `&&`, `||`, `?`, `:`

```json
{
  "class": "PPI::Token::Operator",
  "content": "eq"
}
```

**Rust generation:**
- `eq` → `==` (for strings)
- `ne` → `!=` 
- `&&` → `&&`
- `||` → `||`
- `=~` → `.matches()` (regex)
- `!~` → `!.matches()`

### Numbers: `PPI::Token::Number`

**Perl:** `42`, `3.14`, `0x10`, `0777`

```json
{
  "class": "PPI::Token::Number",
  "content": "42",
  "numeric_value": 42
}
```

**Rust generation:**
- `42` → `42`
- `3.14` → `3.14`
- `0x10` → `0x10`

### Strings: `PPI::Token::Quote::*`

**Perl:** `"hello"`, `'world'`, `qq{text}`

```json
{
  "class": "PPI::Token::Quote::Double",
  "content": "\"Canon\"",
  "string_value": "Canon"
}
```

**Rust generation:**
- `"hello"` → `"hello"`
- `'world'` → `"world"` (normalize to double quotes)

### Function Calls: `PPI::Token::Word` + `PPI::Structure::List`

**Perl:** `sprintf("%.1f", $val)`

```json
{
  "class": "PPI::Statement",
  "children": [
    {
      "class": "PPI::Token::Word",
      "content": "sprintf"
    },
    {
      "class": "PPI::Structure::List",
      "structure_bounds": "( ... )",
      "children": [...]
    }
  ]
}
```

**Rust generation:**
- `sprintf("%.1f", $val)` → `format!("{:.1}", val)`
- `int($val)` → `val.trunc() as i32`
- `abs($val)` → `val.abs()`

### Regular Expressions: `PPI::Token::Regexp::Match`

**Perl:** `/pattern/flags`, `=~ /Canon/i`

```json
{
  "class": "PPI::Token::Regexp::Match",
  "content": "/Canon/i"
}
```

**Rust generation:**
- `/pattern/` → `Regex::new(r"pattern")?`
- `/pattern/i` → `RegexBuilder::new(r"pattern").case_insensitive(true).build()?`

## Real ExifTool Examples

### Example 1: Simple Arithmetic

**Perl Expression:** `$val / 100`

**PPI JSON:**
```json
{
  "class": "PPI::Statement",
  "children": [
    {
      "class": "PPI::Token::Symbol",
      "content": "$val",
      "symbol_type": "scalar"
    },
    {
      "class": "PPI::Token::Operator", 
      "content": "/"
    },
    {
      "class": "PPI::Token::Number",
      "content": "100",
      "numeric_value": 100
    }
  ]
}
```

**Generated Rust:**
```rust
val / 100.0
```

### Example 2: Self Reference Comparison

**Perl Expression:** `$$self{Make} eq 'Canon'`

**PPI JSON:**
```json
{
  "class": "PPI::Statement",
  "children": [
    {
      "class": "PPI::Token::Symbol",
      "content": "$$self{Make}",
      "symbol_type": "scalar"
    },
    {
      "class": "PPI::Token::Operator",
      "content": "eq"
    },
    {
      "class": "PPI::Token::Quote::Single",
      "content": "'Canon'",
      "string_value": "Canon"
    }
  ]
}
```

**Generated Rust:**
```rust
self_ref.get("Make")? == "Canon"
```

### Example 3: Ternary Operator

**Perl Expression:** `$val > 0 ? $val : undef`

**PPI JSON:**
```json
{
  "class": "PPI::Statement",
  "children": [
    {
      "class": "PPI::Token::Symbol",
      "content": "$val",
      "symbol_type": "scalar"
    },
    {
      "class": "PPI::Token::Operator",
      "content": ">"
    },
    {
      "class": "PPI::Token::Number", 
      "content": "0",
      "numeric_value": 0
    },
    {
      "class": "PPI::Token::Operator",
      "content": "?"
    },
    {
      "class": "PPI::Token::Symbol",
      "content": "$val",
      "symbol_type": "scalar"
    },
    {
      "class": "PPI::Token::Operator",
      "content": ":"
    },
    {
      "class": "PPI::Token::Word",
      "content": "undef"
    }
  ]
}
```

**Generated Rust:**
```rust
if val > 0 { Some(val) } else { None }
```

### Example 4: sprintf Function Call

**Perl Expression:** `sprintf("%.1f mm", $val)`

**Generated Rust:**
```rust
format!("{:.1} mm", val)
```

## Rust Code Generation Patterns

### 1. Statement Processing
```rust
fn process_statement(stmt: &PpiNode) -> Result<String, Error> {
    match stmt.class.as_str() {
        "PPI::Statement" => process_children(stmt),
        "PPI::Token::Symbol" => process_variable(stmt),
        "PPI::Token::Operator" => process_operator(stmt),
        "PPI::Token::Number" => Ok(stmt.content.clone()),
        _ => Err(Error::UnsupportedToken(stmt.class.clone()))
    }
}
```

### 2. Variable Mapping
```rust
fn process_variable(token: &PpiNode) -> Result<String, Error> {
    match token.content.as_str() {
        "$val" => Ok("val".to_string()),
        "$valPt" => Ok("val_pt".to_string()),
        _ if token.content.starts_with("$$self{") => {
            // Extract field name from $$self{FieldName}
            let field = extract_self_field(&token.content)?;
            Ok(format!("self_ref.get(\"{}\")?", field))
        },
        _ => Ok(token.content.trim_start_matches('$').to_string())
    }
}
```

### 3. Operator Translation
```rust
fn process_operator(token: &PpiNode) -> Result<String, Error> {
    match token.content.as_str() {
        "eq" => Ok("==".to_string()),
        "ne" => Ok("!=".to_string()),
        "&&" => Ok("&&".to_string()),
        "||" => Ok("||".to_string()),
        "=~" => Ok(".matches".to_string()), // Special handling needed
        op => Ok(op.to_string()) // Most operators are the same
    }
}
```

### 4. Function Call Mapping
```rust
fn process_function_call(name: &str, args: &[PpiNode]) -> Result<String, Error> {
    match name {
        "sprintf" => {
            let format_str = extract_format_string(&args[0])?;
            let rust_format = perl_to_rust_format(&format_str)?;
            let rust_args = process_args(&args[1..])?;
            Ok(format!("format!(\"{}\", {})", rust_format, rust_args.join(", ")))
        },
        "int" => Ok(format!("{}.trunc() as i32", process_node(&args[0])?)),
        "abs" => Ok(format!("{}.abs()", process_node(&args[0])?)),
        _ => Err(Error::UnsupportedFunction(name.to_string()))
    }
}
```

## Error Handling

### Fallback Strategies
1. **PPI parsing fails**: Use string-based pattern matching as fallback
2. **Unknown tokens**: Generate comment with original Perl code
3. **Complex expressions**: Extract to helper functions

```rust
fn generate_expression(perl_expr: &str) -> Result<String, Error> {
    // Try PPI AST first
    if let Some(ast) = perl_expr.get("PrintConv_ast") {
        match process_ppi_ast(ast) {
            Ok(rust_code) => return Ok(rust_code),
            Err(e) => log::warn!("PPI AST failed for '{}': {}", perl_expr, e),
        }
    }
    
    // Fallback to string patterns
    generate_expression_fallback(perl_expr)
}
```

### Debugging Support
```rust
fn generate_with_debug(ast: &PpiNode, original: &str) -> String {
    match process_ppi_ast(ast) {
        Ok(rust_code) => rust_code,
        Err(e) => {
            format!("/* TODO: Failed to convert Perl expression: '{}' - {} */\nformat!(\"({})\", \"CONVERSION_ERROR\")", 
                   original, e)
        }
    }
}
```

## Testing Approach

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_simple_arithmetic() {
        let ast = parse_test_json(r#"{"class": "PPI::Statement", ...}"#);
        assert_eq!(process_ppi_ast(&ast).unwrap(), "val / 100.0");
    }
    
    #[test]
    fn test_self_reference() {
        let ast = parse_test_json(r#"{"class": "PPI::Statement", ...}"#);
        assert_eq!(process_ppi_ast(&ast).unwrap(), r#"self_ref.get("Make")? == "Canon""#);
    }
}
```

### Integration Tests
Test with real ExifTool expressions by comparing outputs with reference ExifTool behavior.

## Implementation Notes

1. **Type Safety**: Use strong typing for PPI node variants
2. **Error Recovery**: Always provide fallback for complex expressions
3. **Performance**: Cache compiled regexes and format strings
4. **Maintainability**: Keep Perl→Rust mappings in configuration files where possible
5. **Testing**: Every supported pattern should have unit tests with ExifTool examples

The goal is 80%+ automatic conversion success for common ExifTool expressions, with clean fallbacks for edge cases.