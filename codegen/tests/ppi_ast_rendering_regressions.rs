//! Regressions for preserving PPI token boundaries through Rust generation.
//!
//! Direct statement/expression visitor entry points can receive a flat infix
//! sequence. The generator must normalize that sequence as PPI nodes before
//! rendering it; reparsing rendered strings confuses operator words inside
//! literals with Perl syntax and independently gets operator associativity wrong.

use codegen::ppi::rust_generator::generator::RustGenerator;
use codegen::ppi::shared_pipeline::call_ppi_ast_script;
use codegen::ppi::{
    normalizer::normalize_multi_pass, parse_ppi_json, ExpressionContext, ExpressionType, PpiNode,
};
use serde_json::json;
use std::process::Command;

fn node(class: &str, content: Option<&str>, string_value: Option<&str>) -> PpiNode {
    PpiNode {
        class: class.to_string(),
        content: content.map(str::to_string),
        children: Vec::new(),
        symbol_type: None,
        numeric_value: None,
        string_value: string_value.map(str::to_string),
        structure_bounds: None,
    }
}

fn symbol(content: &str) -> PpiNode {
    node("PPI::Token::Symbol", Some(content), None)
}

fn operator(content: &str) -> PpiNode {
    node("PPI::Token::Operator", Some(content), None)
}

fn number(content: &str) -> PpiNode {
    node("PPI::Token::Number", Some(content), None)
}

fn string(value: &str) -> PpiNode {
    node(
        "PPI::Token::Quote::Double",
        Some(&format!("\"{value}\"")),
        Some(value),
    )
}

fn statement(children: Vec<PpiNode>) -> PpiNode {
    PpiNode {
        class: "PPI::Statement".to_string(),
        content: None,
        children,
        symbol_type: None,
        numeric_value: None,
        string_value: None,
        structure_bounds: None,
    }
}

fn document(children: Vec<PpiNode>) -> PpiNode {
    PpiNode {
        class: "PPI::Document".to_string(),
        content: None,
        children,
        symbol_type: None,
        numeric_value: None,
        string_value: None,
        structure_bounds: None,
    }
}

fn generate_flat_statement(expression: &str, children: Vec<PpiNode>) -> String {
    let statement = statement(children);
    let generator = RustGenerator::new(
        ExpressionType::ValueConv,
        "test_rendering_regression".to_string(),
        expression.to_string(),
    );
    generator
        .visit_statement(&statement)
        .expect("flat statement should generate through the canonical AST pipeline")
}

fn assert_binary(node: &PpiNode, operator: &str) {
    assert_eq!(node.class, "BinaryOperation");
    assert_eq!(node.content.as_deref(), Some(operator));
    assert_eq!(node.children.len(), 2);
}

/// Mirror the tag-function pipeline used by `PpiFunctionRegistry`: it stores the
/// raw PPI AST and hands it to `generate_function`, which normalizes exactly once.
/// Reproducing that call shape matters — `shared_pipeline` normalizes before
/// calling the same entry point, so a defect that only survives a single
/// normalization pass is invisible to composite/debug-tool style tests.
fn generate_tag_function(
    expression: &str,
    expression_type: ExpressionType,
    function_name: &str,
) -> String {
    let raw_json = call_ppi_ast_script(expression).expect("ppi_ast.pl should parse the expression");
    let raw_ast: PpiNode =
        serde_json::from_str(&raw_json).expect("ppi_ast.pl output should deserialize");

    RustGenerator::new(
        expression_type,
        function_name.to_string(),
        expression.to_string(),
    )
    .generate_function(&raw_ast)
    .expect("registry pipeline should generate this expression")
}

/// Mirror the composite-tag pipeline (`process_perl_expression_with_context`):
/// the AST is normalized once by the shared pipeline and once more inside
/// `generate_function`.
fn try_generate_composite_function(
    expression: &str,
    expression_type: ExpressionType,
    function_name: &str,
) -> Result<String, String> {
    let raw_json = call_ppi_ast_script(expression).expect("ppi_ast.pl should parse the expression");
    let raw_ast: PpiNode =
        serde_json::from_str(&raw_json).expect("ppi_ast.pl output should deserialize");

    RustGenerator::with_context(
        expression_type,
        ExpressionContext::Composite,
        function_name.to_string(),
        expression.to_string(),
    )
    .generate_function(&normalize_multi_pass(raw_ast))
    .map_err(|error| error.to_string())
}

fn generate_composite_function(
    expression: &str,
    expression_type: ExpressionType,
    function_name: &str,
) -> String {
    try_generate_composite_function(expression, expression_type, function_name)
        .expect("composite pipeline should generate this expression")
}

fn assert_generated_function_type_checks(generated_function: &str) {
    let temp_dir = tempfile::tempdir().expect("temporary rustc fixture directory");
    let source_path = temp_dir.path().join("generated_function.rs");
    let output_path = temp_dir.path().join("libgenerated_function.rlib");
    let source = format!(
        r#"
#![allow(dead_code, unused_variables)]

// Minimal stand-ins for the runtime contract the generated code compiles
// against. Signatures mirror `src/core` exactly so a mismatch here is a real
// mismatch in the workspace build.
mod core {{
    use super::TagValue;

    pub mod types {{
        #[derive(Debug)]
        pub struct ExifError;
    }}

    pub mod string {{
        use super::super::TagValue;

        pub fn concat(left: &TagValue, right: &TagValue) -> TagValue {{
            TagValue::String(format!("{{left}}{{right}}"))
        }}
    }}

    pub fn join_vec(separator: &str, values: &[TagValue]) -> TagValue {{
        let _ = (separator, values);
        TagValue::Empty
    }}

    pub fn join_unpack_binary(separator: &str, format: &str, val: &TagValue) -> TagValue {{
        let _ = (separator, format, val);
        TagValue::Empty
    }}

    pub fn unpack_binary(spec: &str, val: &TagValue) -> Vec<TagValue> {{
        let _ = (spec, val);
        Vec::new()
    }}

    pub fn sprintf_perl(format: &str, args: &[TagValue]) -> String {{
        let _ = (format, args);
        String::new()
    }}

    pub fn length_i32<T: Into<TagValue>>(val: T) -> TagValue {{
        let _ = val.into();
        TagValue::I32(0)
    }}

    pub fn get_array_element(val: &TagValue, index: usize) -> TagValue {{
        let _ = (val, index);
        TagValue::Empty
    }}
}}

#[derive(Clone)]
pub enum TagValue {{
    Bool(bool),
    I32(i32),
    String(String),
    Array(Vec<TagValue>),
    Empty,
}}

impl TagValue {{
    pub fn is_truthy(&self) -> bool {{
        match self {{
            Self::Bool(value) => *value,
            Self::I32(value) => *value != 0,
            Self::String(value) => !value.is_empty() && value != "0",
            Self::Array(values) => !values.is_empty(),
            Self::Empty => false,
        }}
    }}

    pub fn string(value: &str) -> Self {{
        Self::String(value.to_string())
    }}
}}

impl std::fmt::Display for TagValue {{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
        match self {{
            Self::Bool(value) => value.fmt(formatter),
            Self::I32(value) => value.fmt(formatter),
            Self::String(value) => value.fmt(formatter),
            Self::Array(_) => "array".fmt(formatter),
            Self::Empty => "".fmt(formatter),
        }}
    }}
}}

impl From<i32> for TagValue {{
    fn from(value: i32) -> Self {{
        Self::I32(value)
    }}
}}

impl From<&str> for TagValue {{
    fn from(value: &str) -> Self {{
        Self::String(value.to_string())
    }}
}}

impl From<String> for TagValue {{
    fn from(value: String) -> Self {{
        Self::String(value)
    }}
}}

impl From<&TagValue> for TagValue {{
    fn from(value: &TagValue) -> Self {{
        value.clone()
    }}
}}

impl PartialEq<i32> for TagValue {{
    fn eq(&self, other: &i32) -> bool {{
        matches!(self, Self::I32(value) if value == other)
    }}
}}

impl PartialEq<i32> for &TagValue {{
    fn eq(&self, other: &i32) -> bool {{
        matches!(self, TagValue::I32(value) if value == other)
    }}
}}

pub struct ExifContext;

{generated_function}
"#
    );
    std::fs::write(&source_path, source).expect("write rustc fixture");

    let output = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .arg("--edition=2021")
        .arg("--crate-type=lib")
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("run rustc over generated function");

    assert!(
        output.status.success(),
        "generated function failed to type-check:\n{}\nfunction:\n{generated_function}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn operator_word_inside_string_literal_is_not_reparsed() {
    let rust = generate_flat_statement(
        "$val eq \"a or b\"",
        vec![symbol("$val"), operator("eq"), string("a or b")],
    );

    assert!(
        rust.contains("val.to_string() == \"a or b\""),
        "string literal token boundary was lost; got:\n{rust}"
    );
    assert!(
        !rust.contains("\"a || b\""),
        "literal contents were parsed as a logical operator; got:\n{rust}"
    );
}

#[test]
fn concat_literal_with_operator_word_stays_whole() {
    let rust = generate_flat_statement(
        "\"5.00 s\" . \" or longer\"",
        vec![string("5.00 s"), operator("."), string(" or longer")],
    );

    assert!(
        rust.contains(
            "crate::core::string::concat(&TagValue::string(\"5.00 s\"), &TagValue::string(\" or longer\"))"
        ),
        "concat operand was not preserved as one string literal; got:\n{rust}"
    );
    assert!(
        !rust.contains("|| longer"),
        "literal contents were parsed as a logical operator; got:\n{rust}"
    );
}

#[test]
fn exponentiation_is_right_associative() {
    let rust = generate_flat_statement(
        "2 ** 3 ** 2",
        vec![
            number("2"),
            operator("**"),
            number("3"),
            operator("**"),
            number("2"),
        ],
    );

    assert!(
        rust.contains(
            "crate::core::power(Into::<TagValue>::into(2i32), crate::core::power(Into::<TagValue>::into(3i32), Into::<TagValue>::into(2i32)))"
        ),
        "Perl exponentiation must group from the right; got:\n{rust}"
    );
    assert!(
        !rust.contains("power(crate::core::power"),
        "exponentiation was rendered left-associatively; got:\n{rust}"
    );
}

#[test]
fn perl_word_or_preserves_comparison_precedence_and_short_circuiting() {
    // PPI::Token::Operator v1.283 classifies word operators as operators.
    // ExifTool BMP.pm:201 uses the same `eq ... or ... eq` shape.
    let children = vec![
        symbol("$val"),
        operator("eq"),
        string("a"),
        operator("or"),
        symbol("$val"),
        operator("eq"),
        string("b"),
    ];

    let normalized = normalize_multi_pass(statement(children.clone()));
    assert_binary(&normalized, "or");
    assert_binary(&normalized.children[0], "eq");
    assert_binary(&normalized.children[1], "eq");

    let rust = RustGenerator::new(
        ExpressionType::Condition,
        "test_word_or".to_string(),
        "$val eq \"a\" or $val eq \"b\"".to_string(),
    )
    .generate_function(&document(vec![statement(children)]))
    .expect("typed Perl word-or expression should generate");

    assert!(
        rust.contains("val.to_string() == \"a\" || val.to_string() == \"b\""),
        "word `or` must preserve comparison precedence and short-circuit; got:\n{rust}"
    );
}

#[test]
fn perl_word_and_preserves_comparison_precedence_and_short_circuiting() {
    // ExifTool Sigma.pm:555 uses `>= ... and ... eq ...`; `and` binds below
    // both comparison forms and short-circuits its right operand.
    let children = vec![
        symbol("$val"),
        operator("eq"),
        string("a"),
        operator("and"),
        symbol("$val"),
        operator("ne"),
        string("b"),
    ];

    let normalized = normalize_multi_pass(statement(children.clone()));
    assert_binary(&normalized, "and");
    assert_binary(&normalized.children[0], "eq");
    assert_binary(&normalized.children[1], "ne");

    let rust = RustGenerator::new(
        ExpressionType::Condition,
        "test_word_and".to_string(),
        "$val eq \"a\" and $val ne \"b\"".to_string(),
    )
    .generate_function(&document(vec![statement(children)]))
    .expect("typed Perl word-and expression should generate");

    assert!(
        rust.contains("val.to_string() == \"a\" && val.to_string() != \"b\""),
        "word `and` must preserve comparison precedence and short-circuit; got:\n{rust}"
    );
}

#[test]
fn condition_word_or_coerces_value_operands_to_bool() {
    let rust = RustGenerator::new(
        ExpressionType::Condition,
        "test_condition_value_or".to_string(),
        "$val or $val".to_string(),
    )
    .generate_function(&document(vec![statement(vec![
        symbol("$val"),
        operator("or"),
        symbol("$val"),
    ])]))
    .expect("condition word-or over values should generate");

    assert!(
        rust.contains("pub fn test_condition_value_or(") && rust.contains(") -> bool"),
        "condition function must retain its bool return type; got:\n{rust}"
    );
    assert!(
        rust.contains("val.is_truthy() || val.is_truthy()"),
        "each value operand must be coerced to Perl truthiness; got:\n{rust}"
    );
    assert_generated_function_type_checks(&rust);
}

#[test]
fn condition_word_and_coerces_only_the_value_operand() {
    let rust = RustGenerator::new(
        ExpressionType::Condition,
        "test_condition_mixed_and".to_string(),
        "$val eq \"a\" and $val".to_string(),
    )
    .generate_function(&document(vec![statement(vec![
        symbol("$val"),
        operator("eq"),
        string("a"),
        operator("and"),
        symbol("$val"),
    ])]))
    .expect("condition word-and over mixed comparison/value operands should generate");

    assert!(
        rust.contains("val.to_string() == \"a\" && val.is_truthy()"),
        "comparison is already bool while the value operand needs truthiness coercion; got:\n{rust}"
    );
    assert_generated_function_type_checks(&rust);
}

#[test]
fn valueconv_word_or_returns_one_owned_tagvalue_operand() {
    let rust = RustGenerator::new(
        ExpressionType::ValueConv,
        "test_valueconv_comparison_or".to_string(),
        "$val eq \"a\" or $val eq \"b\"".to_string(),
    )
    .generate_function(&document(vec![statement(vec![
        symbol("$val"),
        operator("eq"),
        string("a"),
        operator("or"),
        symbol("$val"),
        operator("eq"),
        string("b"),
    ])]))
    .expect("ValueConv word-or over comparisons should generate");

    assert!(
        rust.contains("-> Result<TagValue, crate::core::types::ExifError>"),
        "ValueConv function must retain its TagValue result type; got:\n{rust}"
    );
    assert_eq!(
        rust.matches("TagValue::Bool(").count(),
        2,
        "both Perl boolean operand values must become the one branch type; got:\n{rust}"
    );
    assert!(
        rust.contains("let logical_left = TagValue::Bool(")
            && rust.contains("if logical_left.is_truthy() { logical_left } else"),
        "word-or must evaluate the left operand once and return an owned TagValue; got:\n{rust}"
    );
    assert_generated_function_type_checks(&rust);
}

#[test]
fn public_generate_function_preserves_concat_literal_ppi_fixture() {
    // Real PPI shape for the literal-sensitive core of ExifTool's documented
    // sprintf/concat case: Quote, Operator(.), Quote under a Statement.
    let ast = parse_ppi_json(&json!({
        "class": "PPI::Document",
        "children": [{
            "class": "PPI::Statement",
            "children": [{
                "class": "PPI::Token::Quote::Double",
                "content": "\"5.00 s\"",
                "string_value": "5.00 s"
            }, {
                "class": "PPI::Token::Operator",
                "content": "."
            }, {
                "class": "PPI::Token::Quote::Double",
                "content": "\" or longer\"",
                "string_value": " or longer"
            }]
        }]
    }))
    .expect("PPI JSON fixture should parse");

    let rust = RustGenerator::new(
        ExpressionType::ValueConv,
        "test_public_concat_fixture".to_string(),
        "\"5.00 s\" . \" or longer\"".to_string(),
    )
    .generate_function(&ast)
    .expect("public generation pipeline should preserve the PPI literal node");

    assert!(
        rust.contains(
            "crate::core::string::concat(&TagValue::string(\"5.00 s\"), &TagValue::string(\" or longer\"))"
        ),
        "public pipeline corrupted the concat literal; got:\n{rust}"
    );
}

#[test]
fn repeated_structural_normalization_is_idempotent() {
    let once = normalize_multi_pass(statement(vec![
        symbol("$val"),
        operator("eq"),
        string("a"),
        operator("or"),
        symbol("$val"),
        operator("eq"),
        string("b"),
    ]));
    let twice = normalize_multi_pass(once.clone());

    assert_eq!(
        serde_json::to_value(once).unwrap(),
        serde_json::to_value(twice).unwrap(),
        "normalization boundaries rely on structural idempotence"
    );
}

// ---------------------------------------------------------------------------
// Condition position: a Perl logical operator inside a ternary condition is
// evaluated in boolean context, so it must render a Rust `bool` even inside a
// ValueConv/PrintConv function whose logical *values* are owned `TagValue`s.
// ---------------------------------------------------------------------------

#[test]
fn ternary_condition_over_word_or_renders_bool() {
    // Canon.pm RedEyeReduction composite ValueConv.
    let rust = generate_composite_function(
        "($val[0]==3 or $val[0]==4 or $val[0]==6) ? 1 : 0",
        ExpressionType::ValueConv,
        "composite_valueconv_canon_redeyereduction",
    );

    assert!(
        rust.contains(
            "if (vals.first().cloned().unwrap_or(TagValue::Empty) == 3i32 \
             || vals.first().cloned().unwrap_or(TagValue::Empty) == 4i32) \
             || vals.first().cloned().unwrap_or(TagValue::Empty) == 6i32 {"
        ),
        "chained `or` in a ternary condition must render short-circuiting bools; got:\n{rust}"
    );
    assert!(
        !rust.contains("logical_left"),
        "a TagValue-producing logical block cannot sit in `if` condition position; got:\n{rust}"
    );
    assert_generated_function_type_checks(&rust);
}

#[test]
fn nested_ternary_condition_over_word_and_renders_bool() {
    // Canon.pm ShootingMode composite ValueConv: the `and` sits in the
    // condition of a ternary that is itself a ternary branch.
    let rust = generate_composite_function(
        "$val[0] ? (($val[0] eq \"4\" and $val[2]) ? 7 : $val[0]) : $val[1] + 10",
        ExpressionType::ValueConv,
        "composite_valueconv_canon_shootingmode",
    );

    assert!(
        rust.contains(
            "if vals.first().cloned().unwrap_or(TagValue::Empty).to_string() == \"4\" \
             && (vals.get(2).cloned().unwrap_or(TagValue::Empty)).is_truthy() {"
        ),
        "`and` in a ternary condition must render a bool with Perl truthiness on the \
         value operand; got:\n{rust}"
    );
    assert!(
        !rust.contains("logical_left"),
        "a TagValue-producing logical block cannot sit in `if` condition position; got:\n{rust}"
    );
    assert_generated_function_type_checks(&rust);
}

#[test]
fn ternary_condition_over_regex_and_comparison_renders_bool() {
    // Nikon.pm ContrastDetectAF composite ValueConv. Not type-checked here:
    // the regex branch needs the `regex` crate, which the rustc fixture omits.
    let rust = generate_composite_function(
        "(($val[0] !~ /^Manual/i) and ($val[1] == 1)) ? 1 : 0",
        ExpressionType::ValueConv,
        "composite_valueconv_nikon_contrastdetectaf",
    );

    assert!(
        rust.contains("} && vals.get(1).cloned().unwrap_or(TagValue::Empty) == 1i32 {"),
        "`and` over two boolean operands must render `&&`; got:\n{rust}"
    );
    assert!(
        !rust.contains("logical_left"),
        "a TagValue-producing logical block cannot sit in `if` condition position; got:\n{rust}"
    );
}

// ---------------------------------------------------------------------------
// Nested function-call arguments: normalization must produce the typed
// `FunctionCall` node for an argument that is itself a call, or the caller's
// structural handling (join+unpack, sprintf+unpack) never fires and the
// rendered `TagValue::Array(...)` lands where a `&[TagValue]` is required.
// ---------------------------------------------------------------------------

#[test]
fn join_over_unpack_argument_renders_a_slice() {
    // Photoshop.pm CopyrightFlag / QuickTime.pm VideoFieldOrder ValueConv.
    let rust = generate_tag_function(
        "join(\" \",unpack(\"C*\", $val))",
        ExpressionType::ValueConv,
        "ast_value_346603e685b2cac",
    );

    assert!(
        rust.contains("crate::core::join_unpack_binary(\" \", \"C*\", &val)"),
        "join over unpack must use the slice-aware helper; got:\n{rust}"
    );
    assert!(
        !rust.contains("TagValue::Array("),
        "join_vec takes &[TagValue]; the TagValue::Array wrapper does not type-check; got:\n{rust}"
    );
    assert_generated_function_type_checks(&rust);
}

#[test]
fn join_over_unpack_with_repeated_format_borrows_the_spec() {
    // Nikon.pm AFPointsUsed ValueConv: the unpack spec is `"H2" x 29`, a
    // computed String, while unpack_binary/join_unpack_binary take `&str`.
    let rust = generate_tag_function(
        "join(\" \", unpack(\"H2\"x29, $val))",
        ExpressionType::ValueConv,
        "ast_value_8cf424424a0749dd",
    );

    assert!(
        rust.contains(
            "crate::core::join_unpack_binary(\" \", &\"H2\".repeat(29i32 as usize), &val)"
        ),
        "a computed unpack spec must be borrowed as &str; got:\n{rust}"
    );
    assert_generated_function_type_checks(&rust);
}

#[test]
fn sprintf_over_unpack_argument_splats_the_unpacked_values() {
    // Pentax.pm Date ValueConv. Perl's sprintf consumes the unpacked list as
    // separate arguments, so the generated call must pass the slice itself.
    let rust = generate_tag_function(
        "length($val)==4 ? sprintf(\"%.4d:%.2d:%.2d\",unpack(\"nC2\",$val)) : \"Unknown ($val)\"",
        ExpressionType::ValueConv,
        "ast_value_676f07102a6401b3",
    );

    assert!(
        rust.contains(
            "crate::core::sprintf_perl(\"%.4d:%.2d:%.2d\", \
             &crate::core::unpack_binary(\"nC2\", &val))"
        ),
        "unpacked values must reach sprintf as separate arguments; got:\n{rust}"
    );
    assert!(
        rust.contains("Into::<TagValue>::into(format!(\"Unknown ({})\", val))"),
        "an interpolated ValueConv branch must be converted to TagValue; got:\n{rust}"
    );
    assert_generated_function_type_checks(&rust);
}

// ---------------------------------------------------------------------------
// Statement-level rendering: `return EXPR;` and postfix `unless` must go
// through the structural pipeline instead of joining rendered children.
// ---------------------------------------------------------------------------

#[test]
fn postfix_unless_condition_keeps_its_operand() {
    // Canon.pm SelfTimer PrintConv, first statement.
    let rust = generate_tag_function(
        "return 'Off' unless $val;",
        ExpressionType::PrintConv,
        "test_postfix_unless",
    );

    assert!(
        rust.contains("if !val.is_truthy() { return Into::<TagValue>::into(\"Off\") }"),
        "`unless $val` must negate Perl truthiness of the operand; got:\n{rust}"
    );
    assert!(
        !rust.contains("if ! {"),
        "the negated operand was dropped, leaving an empty condition; got:\n{rust}"
    );
}

#[test]
fn return_statement_expression_is_rendered_structurally() {
    // Canon.pm SelfTimer PrintConv in full: the concatenation after `return`
    // used to be emitted by joining rendered children, which leaked Perl's `.`
    // operator into the Rust source.
    let rust = generate_tag_function(
        "return 'Off' unless $val;\nreturn (($val&0xfff) / 10) . ' s' . ($val & 0x4000 ? ', Custom' : '');",
        ExpressionType::PrintConv,
        "ast_print_76cfbd458e230bac",
    );

    assert!(
        rust.contains("crate::core::string::concat("),
        "the return expression must render through the concat helper; got:\n{rust}"
    );
    assert!(
        !rust.contains("10i32 . "),
        "Perl's concatenation operator leaked into the generated Rust; got:\n{rust}"
    );
}

#[test]
fn returned_literal_is_converted_to_the_functions_value_type() {
    // Sony.pm FocusDistance2 composite ValueConv. Every path returns, so the
    // body is a statement sequence rather than a wrapped expression.
    let rust = generate_composite_function(
        "return undef unless $val;\nreturn 'inf' if $val >= 255;\nreturn (2**($val/16-5) + 1) * $val[1] / 1000;",
        ExpressionType::ValueConv,
        "composite_valueconv_sony_focusdistance2",
    );

    assert!(
        rust.contains("return Ok(Into::<TagValue>::into(\"inf\"))"),
        "a returned literal must be converted to the function's value type; got:\n{rust}"
    );
    assert!(
        !rust.contains("Ok({"),
        "a body that returns on every path must not be wrapped in an unreachable \
         Ok(...); got:\n{rust}"
    );
}

#[test]
fn sprintf_argument_with_bit_shift_renders_the_shift() {
    // Sony.pm LensFirmwareVersion PrintConv. Normalizing the comma-delimited
    // arguments turns `$val>>8` into a typed BinaryOperation, so the visitor
    // has to render the shift instead of rejecting the operator.
    let rust = generate_tag_function(
        "sprintf(\"Ver.%.2x.%.3d\",$val>>8,$val&0xff)",
        ExpressionType::PrintConv,
        "ast_print_c23821edccdd4b5c",
    );

    assert!(
        rust.contains("crate::core::sprintf_perl(\"Ver.%.2x.%.3d\", &[val >> 8i32, "),
        "a right shift argument must render as a Rust shift; got:\n{rust}"
    );
    assert!(
        !rust.contains("missing_print_conv"),
        "the expression must not fall back to a placeholder; got:\n{rust}"
    );
}

#[test]
fn perl_defined_falls_back_instead_of_emitting_a_bare_identifier() {
    // Exif.pm LensID composite ValueConv. `defined` has no translation, so the
    // expression has to reach the placeholder fallback rather than render the
    // Perl keyword as a Rust identifier.
    let error = try_generate_composite_function(
        "return $val[0] if defined $val[0] and $val[0] =~ /(mm|\\d\\/F)/;\nreturn $val[1];",
        ExpressionType::ValueConv,
        "composite_valueconv_exif_lensid_2",
    )
    .expect_err("`defined` must fail closed");

    assert!(
        error.contains("defined"),
        "the failure should name the unsupported construct; got: {error}"
    );
}
