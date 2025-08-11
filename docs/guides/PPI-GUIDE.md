# PPI (Perl Parsing Interface) Guide for New Engineers

## Overview

PPI (Perl Parsing Interface) is a powerful Perl module that parses Perl source code into a structured Abstract Syntax Tree (AST) without executing it. Unlike traditional Perl parsing that requires the interpreter, PPI treats Perl code as a "document" and creates a complete Document Object Model (PDOM) that preserves all formatting, whitespace, and comments.

**Key Principle**: PPI enables "round-trip safe" parsing - the input exactly matches the output when serialized back to source code.

## Why PPI Matters for exif-oxide

In our project, PPI is crucial for translating ExifTool's Perl expressions into Rust code. ExifTool contains thousands of complex Perl expressions for tag conditions, value conversions, and print formatting. Instead of using fragile string-based pattern matching, PPI gives us true syntactic understanding of these expressions, enabling accurate Rust code generation.

## Core Architecture

### PDOM (Perl Document Object Model)

PPI creates a hierarchical tree structure with two main categories:

1. **PPI::Node** - Container elements that can hold other elements
2. **PPI::Token** - Atomic content units representing actual source code bytes

### Class Hierarchy

```
PPI::Element (base class)
├── PPI::Node (containers)
│   ├── PPI::Document (root document)
│   ├── PPI::Statement (code statements) 
│   └── PPI::Structure (braced structures)
└── PPI::Token (content units)
    ├── PPI::Token::Symbol (variables: $var, @array, %hash)
    ├── PPI::Token::Word (barewords, keywords)
    ├── PPI::Token::Operator (operators: +, -, eq, etc.)
    ├── PPI::Token::Number (numeric literals)
    ├── PPI::Token::Quote::* (string literals)
    └── [many other token types]
```

## PPI::Node - Container Base Class

**Purpose**: Abstract base class for elements that can contain other elements.

**Key Characteristics**:
- Inherits from `PPI::Element`
- Maintains a `children` array of contained elements
- Provides tree traversal and manipulation methods

**Essential Methods**:
- `children()` - Returns list of direct child elements
- `add_element($element)` - Adds an element to the end of the node
- `find($class_or_coderef)` - Searches for elements matching criteria
- `prune($class_or_coderef)` - Removes matching elements
- `scope()` - Returns true if node represents a lexical scope boundary

**Example Usage**:
```perl
# Find all barewords in a node
my $barewords = $node->find('PPI::Token::Word');

# Find by complex criteria
my $my_tokens = $node->find(sub { $_[1]->content eq 'my' });

# Remove whitespace
$node->prune('PPI::Token::Whitespace');
```

## PPI::Statement - Perl Statements

**Purpose**: Represents any series of tokens treated as a single statement by Perl.

**Statement Types**:
- **PPI::Statement** - Simple statements (base class)
- **PPI::Statement::Expression** - Boolean conditions, argument lists
- **PPI::Statement::Include** - `use`, `no`, `require` statements
- **PPI::Statement::Package** - Package declarations
- **PPI::Statement::Variable** - `my`, `our`, `local`, `state` declarations
- **PPI::Statement::Compound** - `if`, `unless`, `for`, `while`, `foreach`
- **PPI::Statement::Sub** - Subroutine declarations
- **PPI::Statement::Scheduled** - `BEGIN`, `END`, `CHECK`, `INIT` blocks
- **PPI::Statement::Break** - `return`, `last`, `next`, `redo`, `goto`
- **PPI::Statement::Given/When** - Switch statement constructs (Perl 5.10+)

**Key Methods**:
- `label()` - Returns statement label if present
- `stable()` - Tests if statement remains legal after modification

**ExifTool Context**: Statement parsing is essential for understanding complex conditional logic in tag definitions.

## PPI::Structure - Braced Structures

**Purpose**: Handles all Perl bracing structures: `()`, `[]`, `{}`.

**Unique Behavior**: Structure objects both contain and consist of content. The brace tokens are part of the structure itself, not children.

**Structure Types**:
- **PPI::Structure::List** - Function arguments, literal lists `()`
- **PPI::Structure::Block** - Code blocks for `if`, `sub`, etc. `{}`
- **PPI::Structure::Constructor** - Anonymous array/hash refs `[]`, `{}`
- **PPI::Structure::Subscript** - Array/hash access `$foo[0]`, `$bar{key}`
- **PPI::Structure::Condition** - Boolean conditionals in `if`, `while`
- **PPI::Structure::For** - Three-part C-style for loop expressions
- **PPI::Structure::Given/When** - Switch statement expressions

**Important**: `children()` excludes brace tokens, but `elements()` and `tokens()` include them.

## PPI::Token - Content Units

**Purpose**: Base class for all tokens representing actual source code bytes.

**Major Token Categories**:

### PPI::Token::Symbol
- Represents Perl variables: `$scalar`, `@array`, `%hash`
- Handles complex forms: `$$self{Make}`, `$variable->{key}`
- Critical for ExifTool context references

### PPI::Token::Word  
- Barewords, function names, keywords
- Examples: `if`, `sprintf`, `function_name`
- Essential for identifying ExifTool function calls

### PPI::Token::Operator
- All Perl operators: `+`, `-`, `eq`, `=~`, `&&`, `||`
- Includes assignment operators: `=`, `+=`, etc.

### PPI::Token::Number
- Numeric literals with subtypes:
  - `PPI::Token::Number::Binary` - `0b1010`
  - `PPI::Token::Number::Octal` - `0777`  
  - `PPI::Token::Number::Hex` - `0xFF`
  - `PPI::Token::Number::Float` - `3.14`
  - `PPI::Token::Number::Exp` - `1.23e-4`

### PPI::Token::Quote::*
- String literals with various forms:
  - `PPI::Token::Quote::Single` - `'string'`
  - `PPI::Token::Quote::Double` - `"string"`
  - `PPI::Token::Quote::Literal` - `q{string}`
  - `PPI::Token::Quote::Interpolate` - `qq{string}`

### PPI::Token::Regexp::*
- Regular expressions:
  - `PPI::Token::Regexp::Match` - `m/.../`
  - `PPI::Token::Regexp::Substitute` - `s/.../.../ `
  - `PPI::Token::Regexp::Transliterate` - `tr/.../.../ `

## PPI::Document::Fragment

**Purpose**: Represents a partial Perl document for isolated parsing.

**Key Differences from PPI::Document**:
- Cannot determine line/column positions
- Does not represent a lexical scope
- Useful for parsing expression snippets

**Use Case**: Perfect for parsing individual ExifTool expressions extracted from larger modules.

## Parsing Process

### Tokenization
1. **PPI::Tokenizer** converts source into token stream
2. Handles Perl's complex tokenization rules
3. Preserves all whitespace and comments

### Lexical Analysis  
1. **PPI::Lexer** transforms tokens into hierarchical PDOM
2. Builds proper parent/child relationships
3. Handles Perl's context-sensitive parsing

### Document Creation
```perl
use PPI;

# Parse from string
my $document = PPI::Document->new(\'$var = "value";');

# Parse from file  
my $document = PPI::Document->new('script.pl');

# Parse fragment
my $fragment = PPI::Document::Fragment->new(\'$val > 0');
```

## Working with PPI in exif-oxide

### Our Implementation Pattern

1. **Expression Extraction**: `field_extractor_with_ast.pl` extracts Perl expressions from ExifTool modules
2. **PPI Parsing**: Each expression is parsed into PPI AST structure
3. **JSON Serialization**: AST data is serialized for Rust consumption
4. **Rust Conversion**: `ast/src/ppi_converter.rs` transforms PPI nodes into Rust code

### Key PPI Patterns in ExifTool

**Context Access**:
```perl
# PPI sees this as PPI::Token::Symbol
$$self{Make} eq 'Canon'
```

**Function Calls**:
```perl  
# PPI parses as PPI::Token::Word + PPI::Structure::List
sprintf("%d", $val)
```

**Conditions**:
```perl
# Complex PPI::Statement::Expression structure
$val > 0 && $$self{Model} =~ /EOS/
```

### Performance Considerations

- PPI parsing adds overhead - cache results during codegen
- Use PPI::Document::Fragment for expression snippets
- Implement graceful fallback to string-based parsing for failures

### Error Handling

PPI parsing can fail for:
- Incomplete expressions
- Syntax errors  
- Perl constructs using source filters
- Very complex dynamic code

Always provide fallback mechanisms when PPI parsing fails.

## Best Practices

1. **Trust ExifTool**: Parse exactly what ExifTool does, preserve semantics
2. **Round-trip validation**: Test that generated Rust produces equivalent results  
3. **Cache parsed ASTs**: Avoid re-parsing during codegen
4. **Graceful degradation**: Fall back to simpler approaches when PPI fails
5. **Preserve context**: Maintain ExifTool's variable scoping and context rules

## Debugging PPI

Use `PPI::Dumper` to visualize parse trees:

```perl
use PPI::Dumper;

my $document = PPI::Document->new(\'$var = "value"');
my $dumper = PPI::Dumper->new($document);
print $dumper->string;
```

This shows the complete PDOM structure, helping understand how PPI interprets complex expressions.

## Integration with Rust

Our `ast/src/ppi_types.rs` provides Rust representations of PPI structures:

```rust
pub struct PpiNode {
    pub node_type: String,  // "PPI::Token::Symbol"
    pub content: String,    // "$$self{Make}"  
    pub children: Vec<PpiNode>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}
```

This enables type-safe conversion from Perl AST to Rust code generation, maintaining ExifTool's exact semantics while gaining Rust's performance and safety benefits.