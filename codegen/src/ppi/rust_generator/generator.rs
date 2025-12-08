//! Core Rust code generator from PPI structures
//!
//! Contains the main RustGenerator struct and orchestrates the generation
//! of complete Rust functions from PPI AST nodes.
//!
//! Trust ExifTool: Generated code preserves exact Perl evaluation semantics.

use indoc::formatdoc;
use std::fmt::Write;
use tracing::trace;

use crate::ppi::rust_generator::{
    errors::CodeGenError,
    expressions::{
        wrap_branch_for_owned, wrap_condition_for_bool, BinaryOperationsHandler, ExpressionCombiner,
    },
    functions::FunctionGenerator,
    pattern_matching, signature,
    visitor::PpiVisitor,
};
use crate::ppi::types::{ExpressionType, PpiNode};

/// Generate Rust function from PPI AST node
pub struct RustGenerator {
    pub expression_type: ExpressionType,
    pub function_name: String,
    pub original_expression: String,
}

impl RustGenerator {
    /// Create a new RustGenerator instance
    pub fn new(
        expression_type: ExpressionType,
        function_name: String,
        original_expression: String,
    ) -> Self {
        Self {
            expression_type,
            function_name,
            original_expression,
        }
    }

    /// Format a Perl expression as properly escaped Rust documentation
    pub fn format_perl_expression_doc(expression: &str) -> String {
        // Prefix each line with /// to make it valid Rust doc comment
        // Trim leading whitespace from each line for cleaner formatting
        let doc_expression = expression
            .lines()
            .map(|line| format!("/// {}", line.trim_start()))
            .collect::<Vec<_>>()
            .join("\n");

        formatdoc! {r#"
            /// Original perl expression:
            /// ``` perl
            {}
            /// ```
        "#, doc_expression}
    }

    /// Generate complete function from PPI AST
    pub fn generate_function(&self, ast: &PpiNode) -> Result<String, CodeGenError> {
        let mut code = String::new();

        // Function header with documentation
        let doc_comment = Self::format_perl_expression_doc(&self.original_expression);
        code.push_str(&doc_comment);

        // Function signature
        let signature = signature::generate_signature(&self.expression_type, &self.function_name);
        writeln!(code, "{signature} {{")?;

        // Function body
        let body = self.generate_body(ast)?;
        for line in body.lines() {
            writeln!(code, "    {line}")?;
        }

        writeln!(code, "}}")?;

        Ok(code)
    }

    /// Generate function body from AST using recursive visitor pattern
    fn generate_body(&self, ast: &PpiNode) -> Result<String, CodeGenError> {
        // Enable multi-pass normalizer for multi-token pattern recognition (e.g., join+unpack)
        let normalized_ast = crate::ppi::normalizer::normalize_multi_pass(ast.clone());

        // Check AST structure for standalone literals (rare in ExifTool)
        // Document -> Statement -> Single literal node
        let is_standalone_literal = self.is_standalone_literal(&normalized_ast);

        let code = if is_standalone_literal && self.expression_type == ExpressionType::ValueConv {
            // For standalone literals in ValueConv, wrap them properly
            self.visit_standalone_literal_for_valueconv(&normalized_ast)?
        } else {
            self.visit_node(&normalized_ast)?
        };

        // Wrap the generated expression based on expression type
        match self.expression_type {
            ExpressionType::ValueConv => {
                // Special case: if the expression is just "val", we need to clone it
                // because ValueConv functions return owned TagValue
                if code == "val" {
                    Ok("Ok(val.clone())".to_string())
                } else {
                    // Strip unnecessary outer parentheses before wrapping in Ok()
                    // This avoids clippy warnings about double parentheses: Ok((expr))
                    let unwrapped = Self::strip_outer_parens(&code);
                    Ok(format!("Ok({unwrapped})"))
                }
            }
            ExpressionType::PrintConv => Ok(code),
            ExpressionType::Condition => Ok(code),
        }
    }

    /// Strip unnecessary outer parentheses from an expression.
    /// Only strips if the entire expression is wrapped in matching parens
    /// with nothing after the closing paren (no method calls, etc.)
    fn strip_outer_parens(expr: &str) -> &str {
        let expr = expr.trim();
        if !expr.starts_with('(') || !expr.ends_with(')') {
            return expr;
        }

        // Find where the first '(' closes
        let mut depth = 0;
        for (i, c) in expr.chars().enumerate() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        // If the first ( closes at the last position, we can strip
                        if i == expr.len() - 1 {
                            return &expr[1..expr.len() - 1];
                        }
                        // Otherwise there's something after, keep the parens
                        return expr;
                    }
                }
                _ => {}
            }
        }
        expr
    }

    /// Check if AST represents a standalone literal (no operations)
    fn is_standalone_literal(&self, ast: &PpiNode) -> bool {
        // Pattern: Document -> Statement -> Single literal
        if ast.class != "PPI::Document" || ast.children.len() != 1 {
            return false;
        }

        let statement = &ast.children[0];
        if statement.class != "PPI::Statement" || statement.children.len() != 1 {
            return false;
        }

        let node = &statement.children[0];
        matches!(
            node.class.as_str(),
            "PPI::Token::Number"
                | "PPI::Token::Number::Float"
                | "PPI::Token::Quote::Single"
                | "PPI::Token::Quote::Double"
        )
    }

    /// Special handling for standalone literals in ValueConv context
    fn visit_standalone_literal_for_valueconv(
        &self,
        ast: &PpiNode,
    ) -> Result<String, CodeGenError> {
        // Navigate to the actual literal node
        let literal = &ast.children[0].children[0];

        match literal.class.as_str() {
            "PPI::Token::Number" | "PPI::Token::Number::Float" => {
                if let Some(num) = literal.numeric_value {
                    if num.fract() == 0.0 && num.abs() < i32::MAX as f64 {
                        Ok(format!("TagValue::I32({})", num as i32))
                    } else {
                        Ok(format!("TagValue::F64({num})"))
                    }
                } else {
                    Err(CodeGenError::MissingContent("number".to_string()))
                }
            }
            "PPI::Token::Quote::Single" | "PPI::Token::Quote::Double" => {
                if let Some(ref str_val) = literal.string_value {
                    Ok(format!(
                        "TagValue::String(\"{}\".to_string())",
                        str_val.escape_default()
                    ))
                } else {
                    Err(CodeGenError::MissingContent("string".to_string()))
                }
            }
            _ => {
                // Shouldn't happen based on is_standalone_literal check
                self.visit_node(ast)
            }
        }
    }

    /// Visit document node - handle multi-statement patterns like operation ; return var
    pub fn visit_document(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        if node.children.is_empty() {
            return Ok("".to_string());
        }

        if node.children.len() == 1 {
            // Simple case: single statement
            return self.visit_node(&node.children[0]);
        }

        // For multi-statement documents, check if they contain complex constructs
        // that we cannot reliably translate to Rust
        for child in &node.children {
            if pattern_matching::check_node_complexity(child).is_err() {
                return Err(CodeGenError::UnsupportedStructure(
                    "Multi-statement block contains complex Perl constructs that cannot be translated".to_string()
                ));
            }
        }

        // Handle multiple statements (e.g., "$val=~tr/ /./; $val")
        // For ExifTool compatibility, we need to process all statements
        // and return the result of the last one (Perl's behavior)
        let mut results = Vec::new();
        let mut last_result = String::new();

        for (i, child) in node.children.iter().enumerate() {
            // Skip whitespace and comments that PPI might include
            if child.class == "PPI::Token::Whitespace" || child.class == "PPI::Token::Comment" {
                continue;
            }

            let result = self.visit_node(child)?;

            // Skip empty results
            if result.trim().is_empty() {
                continue;
            }

            // For multiple statements, we need to handle them as a sequence
            if i == node.children.len() - 1 {
                // Last statement becomes the return value
                last_result = result;
            } else {
                // Earlier statements are executed for side effects
                results.push(result);
            }
        }

        if results.is_empty() {
            // Only one meaningful statement
            Ok(last_result)
        } else {
            // Multiple statements: combine them in a block expression
            results.push(last_result);
            let statements = results[..results.len() - 1].join(";\n    ");
            let final_result = &results[results.len() - 1];
            Ok(formatdoc! {r#"
                {{
                    {statements};
                    {final_result}
                }}
            "#})
        }
    }

    /// Visit statement node - processes children and combines them intelligently
    pub fn visit_statement(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        self.process_node_sequence(&node.children)
    }

    /// Visit expression node - handles complex expressions with function composition
    /// PPI::Statement::Expression (4,172 occurrences) - Essential for complex expressions
    pub fn visit_expression(&self, node: &PpiNode) -> Result<String, CodeGenError> {
        self.process_node_sequence(&node.children)
    }

    /// Process a sequence of child nodes with intelligent pattern recognition
    fn process_node_sequence(&self, children: &[PpiNode]) -> Result<String, CodeGenError> {
        if children.is_empty() {
            return Ok("".to_string());
        }

        // Single child - just visit it
        if children.len() == 1 {
            return self.visit_node(&children[0]);
        }

        trace!("process_node_sequence: {} children", children.len());

        // Check for \$val reference pattern - we can't handle references
        // Pattern: Cast(\) + Symbol($val)
        // This is used for binary data handling in ExifTool but not needed for our required tags
        if children.len() >= 2 {
            trace!(
                "Checking for reference pattern: [0] class={}, content={:?}; [1] class={}, content={:?}",
                children[0].class, children[0].content,
                children[1].class, children[1].content
            );

            if children[0].class == "PPI::Token::Cast"
                && children[0].content.as_ref() == Some(&"\\".to_string())
                && children[1].class == "PPI::Token::Symbol"
            {
                return Err(CodeGenError::UnsupportedStructure(
                    "Reference operator (\\$val) requires fallback implementation".to_string(),
                ));
            }
        }

        // Check for join with regex pattern - generates invalid code
        // Pattern: join ".", $val =~ /../g
        let mut has_join = false;
        let mut has_regex_match = false;
        for child in children {
            if child.class == "PPI::Token::Word" && child.content.as_deref() == Some("join") {
                has_join = true;
            }
            if child.class == "PPI::Token::Operator" && child.content.as_deref() == Some("=~") {
                has_regex_match = true;
            }
        }
        if has_join && has_regex_match {
            return Err(CodeGenError::UnsupportedStructure(
                "join with regex pattern requires fallback implementation".to_string(),
            ));
        }

        // Check for string concatenation with unpack - generates invalid code
        // Pattern: "0x" . unpack("H*",$val)
        let mut has_unpack = false;
        let mut has_concat = false;
        for child in children {
            if child.class == "PPI::Token::Word" && child.content.as_deref() == Some("unpack") {
                has_unpack = true;
            }
            if child.class == "PPI::Token::Operator" && child.content.as_deref() == Some(".") {
                has_concat = true;
            }
        }
        if has_unpack && has_concat {
            return Err(CodeGenError::UnsupportedStructure(
                "String concatenation with unpack requires fallback implementation".to_string(),
            ));
        }

        // Check for comma operator at statement level - Perl returns last value
        // Pattern: $val =~ s/.../, $val  (do substitution, return $val)
        // This generates invalid Rust like Ok(expr1, expr2) which doesn't compile
        let has_top_level_comma = children.iter().any(|child| {
            child.class == "PPI::Token::Operator" && child.content.as_deref() == Some(",")
        });
        if has_top_level_comma {
            return Err(CodeGenError::UnsupportedStructure(
                "Comma operator at statement level requires fallback implementation".to_string(),
            ));
        }

        // Check for sprintf with unpack without parentheses
        // Pattern: sprintf "%s", unpack "H4", $val
        // The unpack args get mis-parsed when not in parentheses
        let has_sprintf = children
            .iter()
            .any(|c| c.class == "PPI::Token::Word" && c.content.as_deref() == Some("sprintf"));
        if has_sprintf && has_unpack {
            return Err(CodeGenError::UnsupportedStructure(
                "sprintf with unpack requires fallback implementation".to_string(),
            ));
        }

        // Check for sprintf with binary operations in arguments
        // This pattern doesn't normalize correctly and generates invalid code
        if self.has_sprintf_with_binary_ops(children) {
            return Err(CodeGenError::UnsupportedStructure(
                "sprintf with binary operations in arguments requires fallback implementation"
                    .to_string(),
            ));
        }

        // Check for complex nested function patterns that we can't handle
        // Pattern: unpack "H*", pack "C*", split " ", $val
        if children.len() >= 6 {
            let mut has_unpack = false;
            let mut has_pack = false;
            let mut has_split = false;

            for child in children {
                if child.class == "PPI::Token::Word" {
                    if let Some(content) = &child.content {
                        match content.as_str() {
                            "unpack" => has_unpack = true,
                            "pack" => has_pack = true,
                            "split" => has_split = true,
                            _ => {}
                        }
                    }
                }
            }

            // This is a complex nested function pattern we can't handle
            if has_unpack && has_pack && has_split {
                return Err(CodeGenError::UnsupportedStructure(
                    "Complex nested function pattern (unpack/pack/split) requires fallback implementation".to_string()
                ));
            }
        }

        // First check for ternary pattern: condition ? true_expr : false_expr
        if let Some(ternary_result) = self.try_process_ternary_pattern(children)? {
            return Ok(ternary_result);
        }

        // Check for $$self{field} pattern BEFORE other patterns
        // Pattern: Cast($) + Symbol($self) + Subscript{field}
        if children.len() >= 3 {
            trace!(
                "Checking for $$self pattern: [0] class={}, content={:?}; [1] class={}, content={:?}; [2] class={}, content={:?}",
                children[0].class, children[0].content,
                children[1].class, children[1].content,
                children[2].class, children[2].content
            );
        }

        if children.len() >= 3
            && children[0].class == "PPI::Token::Cast"
            && children[0].content.as_ref() == Some(&"$".to_string())
            && children[1].class == "PPI::Token::Symbol"
            && children[1].content.as_ref() == Some(&"$self".to_string())
            && children[2].class == "PPI::Structure::Subscript"
        {
            if let Some(key) = self.extract_subscript_key(&children[2])? {
                // Generate context access that handles Option<&ExifContext>
                // For TimeScale specifically, default to 1 to avoid division by zero
                let default_value = if key == "TimeScale" {
                    "TagValue::U32(1)"
                } else {
                    "TagValue::String(String::new())"
                };
                trace!("Found $$self{{{}}} pattern, generating context access", key);
                return Ok(format!(
                    "ctx.and_then(|c| c.get_data_member(\"{key}\").cloned()).unwrap_or({default_value})"
                ));
            } else {
                trace!("Pattern matched but couldn't extract key from subscript");
            }
        }

        // Check for array access pattern BEFORE trying to visit individual nodes
        // This prevents the subscript from being visited directly which causes errors
        for i in 0..children.len() {
            if children[i].class == "PPI::Token::Symbol"
                && i + 1 < children.len()
                && children[i + 1].class == "PPI::Structure::Subscript"
            {
                let array_name = children[i].content.as_deref().unwrap_or("$val");
                let rust_array = if array_name == "$val" {
                    "val"
                } else {
                    array_name.trim_start_matches('$')
                };

                if let Some(index) = self.extract_subscript_index(&children[i + 1])? {
                    // Generate array access that handles all array types
                    // This uses the get_array_element helper that handles typed arrays
                    return Ok(format!(
                        "codegen_runtime::get_array_element({rust_array}, {index})"
                    ));
                }
            }
        }

        // Check for binary operation patterns first
        let parts: Vec<String> = children
            .iter()
            .filter(|child| child.class != "PPI::Token::Whitespace")
            .map(|child| self.visit_node(child))
            .collect::<Result<Vec<_>, _>>()?;

        if let Some(binary_result) = self.try_binary_operation_pattern(&parts)? {
            return Ok(binary_result);
        }

        // Look for other patterns in the sequence
        let mut processed = Vec::new();
        let mut i = 0;

        while i < children.len() {
            // Skip whitespace
            if children[i].class == "PPI::Token::Whitespace" {
                i += 1;
                continue;
            }

            // Pattern: method call (Symbol + -> + Word) - detect and fail explicitly
            if children[i].class == "PPI::Token::Symbol"
                && i + 2 < children.len()
                && children[i + 1].class == "PPI::Token::Operator"
                && children[i + 1].content.as_ref() == Some(&"->".to_string())
                && children[i + 2].is_word()
            {
                // Method call pattern detected - this is unsupported in standalone functions
                // Following explicit failure semantics from CODEGEN.md
                let symbol = children[i].content.as_deref().unwrap_or("unknown");
                let method = children[i + 2].content.as_deref().unwrap_or("unknown");
                return Err(CodeGenError::UnsupportedStructure(format!(
                    "Method call '{symbol}->{method}()' is not supported in standalone functions"
                )));
            }

            // Pattern: function call with parentheses (Word + Structure::List)
            if children[i].is_word()
                && i + 1 < children.len()
                && children[i + 1].class == "PPI::Structure::List"
            {
                let func_result = self.process_function_call(&children[i], &children[i + 1])?;
                processed.push(func_result);
                i += 2;
                continue;
            }

            // Pattern: array access (Symbol + Subscript for $val[0])
            if children[i].class == "PPI::Token::Symbol"
                && i + 1 < children.len()
                && children[i + 1].class == "PPI::Structure::Subscript"
            {
                // Handle array subscript access like $val[0], $val[1]
                let array_name = children[i].content.as_deref().unwrap_or("$val");
                let rust_array = if array_name == "$val" {
                    "val"
                } else {
                    array_name.trim_start_matches('$')
                };

                // Extract the index from the subscript structure
                if let Some(index) = self.extract_subscript_index(&children[i + 1])? {
                    // Generate array access that handles all array types
                    let array_access =
                        format!("codegen_runtime::get_array_element({rust_array}, {index})");
                    processed.push(array_access);
                    i += 2;
                    continue;
                }
            }

            // Pattern: hash dereference (Cast + Symbol + Subscript for $$self{key})
            if children[i].class == "PPI::Token::Cast"
                && children[i].content.as_ref() == Some(&"$".to_string())
                && i + 2 < children.len()
                && children[i + 1].class == "PPI::Token::Symbol"
                && children[i + 1].content.as_ref() == Some(&"$self".to_string())
                && children[i + 2].class == "PPI::Structure::Subscript"
            {
                // Extract the key from the subscript structure
                if let Some(key) = self.extract_subscript_key(&children[i + 2])? {
                    // Generate context access that handles Option<&ExifContext>
                    // For TimeScale specifically, default to 1 to avoid division by zero
                    let default_value = if key == "TimeScale" {
                        "TagValue::U32(1)"
                    } else {
                        "TagValue::String(String::new())"
                    };
                    let hash_access = format!(
                        "ctx.and_then(|c| c.get_data_member(\"{key}\").cloned()).unwrap_or({default_value})"
                    );
                    processed.push(hash_access);
                    i += 3;
                    continue;
                }
            }

            // Default: visit the node normally
            processed.push(self.visit_node(&children[i])?);
            i += 1;
        }

        // Combine the processed parts - this handles multi-arg functions without parens
        self.combine_processed_parts(&processed, children)
    }

    /// Try to process ternary pattern: condition ? true_expr : false_expr
    /// Returns Some(result) if ternary pattern found, None otherwise
    fn try_process_ternary_pattern(
        &self,
        children: &[PpiNode],
    ) -> Result<Option<String>, CodeGenError> {
        // Find the ? and : operators to identify ternary structure
        let mut question_idx = None;
        let mut colon_idx = None;

        for (i, child) in children.iter().enumerate() {
            if child.class == "PPI::Token::Operator" {
                if let Some(content) = &child.content {
                    if content == "?" && question_idx.is_none() {
                        question_idx = Some(i);
                    } else if content == ":" && question_idx.is_some() && colon_idx.is_none() {
                        colon_idx = Some(i);
                        break; // Found both operators
                    }
                }
            }
        }

        // If we found both ? and : operators, this is a ternary expression
        if let (Some(q_idx), Some(c_idx)) = (question_idx, colon_idx) {
            if q_idx < c_idx {
                // Extract the three parts: condition, true_expr, false_expr
                let condition_nodes = &children[..q_idx];
                let true_expr_nodes = &children[q_idx + 1..c_idx];
                let false_expr_nodes = &children[c_idx + 1..];

                // Debug output for condition nodes
                trace!(
                    "Ternary condition has {} nodes: {:?}",
                    condition_nodes.len(),
                    condition_nodes
                        .iter()
                        .map(|n| (&n.class, &n.content))
                        .collect::<Vec<_>>()
                );

                // Process each part using the proper expression combiner logic
                // Apply normalization to each branch to handle complex expressions properly
                //
                // CRITICAL: Each branch of a ternary operator must be normalized separately
                // because complex expressions (arithmetic, function calls, etc.) within ternary
                // branches may not be properly parsed without normalization. This ensures:
                // - Operator precedence is correctly handled in all branches
                // - Function calls are properly recognized and structured
                // - String operations are normalized consistently
                // - Complex nested expressions are properly bracketed
                let condition_ast = PpiNode {
                    class: "PPI::Statement".to_string(),
                    content: None,
                    children: condition_nodes.to_vec(),
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                };
                let normalized_condition =
                    crate::ppi::normalizer::normalize_multi_pass(condition_ast);
                let condition = self.process_node_sequence(&normalized_condition.children)?;

                let true_ast = PpiNode {
                    class: "PPI::Statement".to_string(),
                    content: None,
                    children: true_expr_nodes.to_vec(),
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                };
                let normalized_true = crate::ppi::normalizer::normalize_multi_pass(true_ast);
                let true_expr = self.process_node_sequence(&normalized_true.children)?;

                let false_ast = PpiNode {
                    class: "PPI::Statement".to_string(),
                    content: None,
                    children: false_expr_nodes.to_vec(),
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                };
                let normalized_false = crate::ppi::normalizer::normalize_multi_pass(false_ast);
                let false_expr = self.process_node_sequence(&normalized_false.children)?;

                // Wrap condition for bool conversion and branches for ownership
                let condition_wrapped = wrap_condition_for_bool(&condition);
                let true_expr_wrapped = wrap_branch_for_owned(&true_expr);
                let false_expr_wrapped = wrap_branch_for_owned(&false_expr);

                // Generate Rust if-else expression
                let result = format!(
                    "if {condition_wrapped} {{ {true_expr_wrapped} }} else {{ {false_expr_wrapped} }}"
                );
                return Ok(Some(result));
            }
        }

        Ok(None)
    }

    /// Process function call with arguments
    fn process_function_call(
        &self,
        name_node: &PpiNode,
        args_node: &PpiNode,
    ) -> Result<String, CodeGenError> {
        let func_name = self.visit_word(name_node)?;

        // Special handling for sprintf - always convert it
        if func_name == "sprintf" {
            // Create a temporary node structure for the function call
            let mut function_node = name_node.clone();
            function_node.children = args_node.children.clone();
            return FunctionGenerator::generate_function_call_from_parts(self, &function_node);
        }

        // Generate the function call using AST node
        // Create a temporary node structure for the function call
        let mut function_node = name_node.clone();
        function_node.children = args_node.children.clone();
        FunctionGenerator::generate_function_call_from_parts(self, &function_node)
    }

    /// Combine processed parts using expression combiner logic
    fn combine_processed_parts(
        &self,
        parts: &[String],
        original_children: &[PpiNode],
    ) -> Result<String, CodeGenError> {
        // Filter out empty parts
        let parts: Vec<_> = parts.iter().filter(|p| !p.is_empty()).cloned().collect();

        if parts.is_empty() {
            return Ok("".to_string());
        }

        if parts.len() == 1 {
            return Ok(parts[0].clone());
        }

        // Use combine_statement_parts which handles multi-arg functions without parens
        ExpressionCombiner::combine_statement_parts(self, &parts, original_children)
    }

    /// Extract the key from a subscript structure (e.g., {FocalUnits} -> "FocalUnits")
    fn extract_subscript_index(
        &self,
        subscript_node: &PpiNode,
    ) -> Result<Option<String>, CodeGenError> {
        // The subscript should have children that represent the index
        if subscript_node.children.is_empty() {
            return Ok(None);
        }

        // For [0], [1], etc., we expect a single expression child containing the number
        if subscript_node.children.len() == 1
            && subscript_node.children[0].class == "PPI::Statement::Expression"
        {
            let expr_node = &subscript_node.children[0];
            if !expr_node.children.is_empty() && expr_node.children[0].class == "PPI::Token::Number"
            {
                if let Some(index) = &expr_node.children[0].content {
                    return Ok(Some(index.clone()));
                }
            }
        }

        Ok(None)
    }

    fn extract_subscript_key(
        &self,
        subscript_node: &PpiNode,
    ) -> Result<Option<String>, CodeGenError> {
        // The subscript should have children that represent the key
        if subscript_node.children.is_empty() {
            trace!("extract_subscript_key: No children in subscript");
            return Ok(None);
        }

        trace!(
            "extract_subscript_key: {} children, first child class: {}",
            subscript_node.children.len(),
            subscript_node.children[0].class
        );

        // For {FocalUnits}, we expect a single expression child containing the word
        if subscript_node.children.len() == 1
            && subscript_node.children[0].class == "PPI::Statement::Expression"
        {
            let expr_node = &subscript_node.children[0];
            if !expr_node.children.is_empty() && expr_node.children[0].class == "PPI::Token::Word" {
                if let Some(key) = &expr_node.children[0].content {
                    trace!("extract_subscript_key: Found key '{}'", key);
                    return Ok(Some(key.clone()));
                }
            }
        }

        Ok(None)
    }

    /// Check if children contain sprintf with binary operations
    /// This recursively checks for patterns that generate invalid Rust code
    fn has_sprintf_with_binary_ops(&self, children: &[PpiNode]) -> bool {
        let mut has_sprintf = false;
        let mut has_binary_op = false;

        for child in children {
            // Check for sprintf function
            if child.class == "PPI::Token::Word" && child.content.as_deref() == Some("sprintf") {
                has_sprintf = true;
            }

            // Check for binary operators (*, /, +, -)
            if child.class == "PPI::Token::Operator" {
                if let Some(op) = child.content.as_deref() {
                    if matches!(op, "*" | "/" | "+" | "-") {
                        has_binary_op = true;
                    }
                }
            }

            // Recursively check children for nested patterns
            if self.has_sprintf_with_binary_ops(&child.children) {
                return true;
            }
        }

        has_sprintf && has_binary_op
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ppi::{ExpressionType, PpiNode};

    fn create_test_generator() -> RustGenerator {
        RustGenerator::new(
            ExpressionType::PrintConv,
            "test_function".to_string(),
            "test expression".to_string(),
        )
    }

    #[test]
    fn test_has_sprintf_with_binary_ops_basic() {
        let generator = create_test_generator();

        // Test case with sprintf and multiplication
        let children_with_sprintf_and_mult = vec![
            PpiNode {
                class: "PPI::Token::Word".to_string(),
                content: Some("sprintf".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Operator".to_string(),
                content: Some("*".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
        ];

        assert!(generator.has_sprintf_with_binary_ops(&children_with_sprintf_and_mult));
    }

    #[test]
    fn test_has_sprintf_with_binary_ops_all_operators() {
        let generator = create_test_generator();

        let sprintf_node = PpiNode {
            class: "PPI::Token::Word".to_string(),
            content: Some("sprintf".to_string()),
            children: vec![],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        };

        // Test each binary operator
        for op in &["*", "/", "+", "-"] {
            let children = vec![
                sprintf_node.clone(),
                PpiNode {
                    class: "PPI::Token::Operator".to_string(),
                    content: Some(op.to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                },
            ];

            assert!(
                generator.has_sprintf_with_binary_ops(&children),
                "Should detect sprintf with '{}' operator",
                op
            );
        }
    }

    #[test]
    fn test_has_sprintf_with_binary_ops_false_cases() {
        let generator = create_test_generator();

        // Test case with sprintf but no binary operators
        let sprintf_only = vec![
            PpiNode {
                class: "PPI::Token::Word".to_string(),
                content: Some("sprintf".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Quote::Double".to_string(),
                content: Some("\"format\"".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
        ];

        assert!(!generator.has_sprintf_with_binary_ops(&sprintf_only));

        // Test case with binary ops but no sprintf
        let binary_only = vec![
            PpiNode {
                class: "PPI::Token::Symbol".to_string(),
                content: Some("$val".to_string()),
                children: vec![],
                symbol_type: Some("scalar".to_string()),
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Operator".to_string(),
                content: Some("*".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
        ];

        assert!(!generator.has_sprintf_with_binary_ops(&binary_only));

        // Test empty children
        assert!(!generator.has_sprintf_with_binary_ops(&[]));
    }

    #[test]
    fn test_has_sprintf_with_binary_ops_nested() {
        let generator = create_test_generator();

        // Test nested structure where sprintf and operator are in child nodes
        let nested_children = vec![PpiNode {
            class: "PPI::Structure::List".to_string(),
            content: Some("(".to_string()),
            children: vec![
                PpiNode {
                    class: "PPI::Token::Word".to_string(),
                    content: Some("sprintf".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                },
                PpiNode {
                    class: "PPI::Token::Operator".to_string(),
                    content: Some("*".to_string()),
                    children: vec![],
                    symbol_type: None,
                    numeric_value: None,
                    string_value: None,
                    structure_bounds: None,
                },
            ],
            symbol_type: None,
            numeric_value: None,
            string_value: None,
            structure_bounds: None,
        }];

        assert!(generator.has_sprintf_with_binary_ops(&nested_children));
    }

    #[test]
    fn test_has_sprintf_with_binary_ops_ignores_other_operators() {
        let generator = create_test_generator();

        // Test that non-binary operators are ignored
        let children_with_other_ops = vec![
            PpiNode {
                class: "PPI::Token::Word".to_string(),
                content: Some("sprintf".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Operator".to_string(),
                content: Some(",".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
            PpiNode {
                class: "PPI::Token::Operator".to_string(),
                content: Some(".".to_string()),
                children: vec![],
                symbol_type: None,
                numeric_value: None,
                string_value: None,
                structure_bounds: None,
            },
        ];

        assert!(!generator.has_sprintf_with_binary_ops(&children_with_other_ops));
    }
}
