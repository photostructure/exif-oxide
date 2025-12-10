//! Generate Rust Test Files from JSON Expression Configs
//!
//! This tool processes JSON expression test configurations through the complete PPI pipeline:
//! 1. Parse JSON configs to get expressions and test cases
//! 2. Process each expression through ppi_ast.pl → normalizer → fn_registry
//! 3. Generate deduplicated function files in tests/generated/functions/
//! 4. Generate test files that import and execute these functions with assertions
//!
//! Supports two modes:
//! - Full generation (--dir): Process all JSON files, update all mod.rs files
//! - Single-file debug (--file): Process one JSON file, skip mod.rs updates
//!
//! Usage:
//!   cargo run --bin generate-expression-tests -- --dir tests/config/ --output tests/generated/
//!   cargo run --bin generate-expression-tests -- --file test.json --output tests/generated/

use anyhow::{Context, Result};
use clap::Parser;
use glob::glob;
use indoc::formatdoc;
use jsonschema::Validator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use codegen::ppi::fn_registry::{FunctionSpec, PpiFunctionRegistry};
use codegen::ppi::normalizer::normalize_multi_pass;
use codegen::ppi::shared_pipeline::call_ppi_ast_script;
use codegen::ppi::{ExpressionType, PpiNode};

#[derive(Parser)]
#[command(
    name = "generate-expression-tests",
    about = "Generate Rust test files from JSON expression configs",
    long_about = "This tool processes JSON expression test configurations through the complete PPI pipeline.
Generates deduplicated functions via fn_registry and test files that import and execute them.

Supports two modes:
- Full generation (--dir): Process all JSON files, regenerate everything including mod.rs
- Single-file debug (--file): Process one JSON file, skip mod.rs updates for rapid iteration"
)]
struct Args {
    /// Process single JSON file (debug mode - skips mod.rs updates)
    #[arg(long, conflicts_with = "dir")]
    file: Option<PathBuf>,

    /// Process directory of JSON files recursively (full generation mode)
    #[arg(long, conflicts_with = "file")]
    dir: Option<PathBuf>,

    /// Output directory for generated files
    #[arg(long)]
    output: PathBuf,

    /// JSON schema file for validation
    #[arg(long, default_value = "tests/config/schema.json")]
    schema: PathBuf,

    /// Skip schema validation (faster but less safe)
    #[arg(long)]
    skip_validation: bool,

    /// Verbose output for debugging
    #[arg(long, short)]
    verbose: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct ExpressionTestFile {
    description: Option<String>,
    expression: String,
    #[serde(rename = "type")]
    expr_type: ExpressionType,
    exiftool_reference: Option<String>,
    placeholder_expected: Option<bool>,
    usage_examples: Option<Vec<String>>,
    test_cases: Option<Vec<TestCase>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TestCase {
    description: Option<String>,
    input: TaggedTagValue,
    expected: TaggedTagValue,
}

/// Wrapper for TagValue that can deserialize from tagged JSON format like {"U32": 50}
#[derive(Debug, Clone, Deserialize, Serialize)]
enum TaggedTagValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I16(i16),
    I32(i32),
    F64(f64),
    String(String),
    Bool(bool),
    Array(Vec<TaggedTagValue>),
    Binary(Vec<u8>),
    Rational(u32, u32),
    SRational(i32, i32),
    U8Array(Vec<u8>),
    U16Array(Vec<u16>),
    U32Array(Vec<u32>),
    F64Array(Vec<f64>),
    RationalArray(Vec<(u32, u32)>),
    SRationalArray(Vec<(i32, i32)>),
    Empty,
}

impl From<TaggedTagValue> for TagValue {
    fn from(tagged: TaggedTagValue) -> Self {
        match tagged {
            TaggedTagValue::U8(v) => TagValue::U8(v),
            TaggedTagValue::U16(v) => TagValue::U16(v),
            TaggedTagValue::U32(v) => TagValue::U32(v),
            TaggedTagValue::U64(v) => TagValue::U64(v),
            TaggedTagValue::I16(v) => TagValue::I16(v),
            TaggedTagValue::I32(v) => TagValue::I32(v),
            TaggedTagValue::F64(v) => TagValue::F64(v),
            TaggedTagValue::String(v) => TagValue::String(v),
            TaggedTagValue::Bool(v) => TagValue::Bool(v),
            TaggedTagValue::Array(v) => TagValue::Array(v.into_iter().map(|t| t.into()).collect()),
            TaggedTagValue::Binary(v) => TagValue::Binary(v),
            TaggedTagValue::Rational(n, d) => TagValue::Rational(n, d),
            TaggedTagValue::SRational(n, d) => TagValue::SRational(n, d),
            TaggedTagValue::U8Array(v) => TagValue::U8Array(v),
            TaggedTagValue::U16Array(v) => TagValue::U16Array(v),
            TaggedTagValue::U32Array(v) => TagValue::U32Array(v),
            TaggedTagValue::F64Array(v) => TagValue::F64Array(v),
            TaggedTagValue::RationalArray(v) => TagValue::RationalArray(v),
            TaggedTagValue::SRationalArray(v) => TagValue::SRationalArray(v),
            TaggedTagValue::Empty => TagValue::Empty,
        }
    }
}

/// Generate the Rust code for a TagValue constructor
fn generate_tag_value_constructor(tagged: &TaggedTagValue) -> String {
    match tagged {
        TaggedTagValue::U8(v) => format!("TagValue::U8({})", v),
        TaggedTagValue::U16(v) => format!("TagValue::U16({})", v),
        TaggedTagValue::U32(v) => format!("TagValue::U32({})", v),
        TaggedTagValue::U64(v) => format!("TagValue::U64({})", v),
        TaggedTagValue::I16(v) => format!("TagValue::I16({})", v),
        TaggedTagValue::I32(v) => format!("TagValue::I32({})", v),
        TaggedTagValue::F64(v) => format!("TagValue::F64({}f64)", v),
        TaggedTagValue::String(v) => {
            format!("TagValue::String(\"{}\".to_string())", v.escape_default())
        }
        TaggedTagValue::Bool(v) => format!("TagValue::Bool({})", v),
        TaggedTagValue::Array(v) => {
            let elements: Vec<String> = v.iter().map(generate_tag_value_constructor).collect();
            format!("TagValue::Array(vec![{}])", elements.join(", "))
        }
        TaggedTagValue::Binary(v) => format!("TagValue::Binary(vec!{:?})", v),
        TaggedTagValue::Rational(n, d) => format!("TagValue::Rational({}, {})", n, d),
        TaggedTagValue::SRational(n, d) => format!("TagValue::SRational({}, {})", n, d),
        TaggedTagValue::U8Array(v) => format!("TagValue::U8Array(vec!{:?})", v),
        TaggedTagValue::U16Array(v) => format!("TagValue::U16Array(vec!{:?})", v),
        TaggedTagValue::U32Array(v) => format!("TagValue::U32Array(vec!{:?})", v),
        TaggedTagValue::F64Array(v) => format!("TagValue::F64Array(vec!{:?})", v),
        TaggedTagValue::RationalArray(v) => {
            let pairs: Vec<String> = v.iter().map(|(n, d)| format!("({}, {})", n, d)).collect();
            format!("TagValue::RationalArray(vec![{}])", pairs.join(", "))
        }
        TaggedTagValue::SRationalArray(v) => {
            let pairs: Vec<String> = v.iter().map(|(n, d)| format!("({}, {})", n, d)).collect();
            format!("TagValue::SRationalArray(vec![{}])", pairs.join(", "))
        }
        TaggedTagValue::Empty => "TagValue::Empty".to_string(),
    }
}

/// Information about a processed test file for generating tests
struct ProcessedTestFile {
    json_path: PathBuf,
    config: ExpressionTestFile,
    function_spec: FunctionSpec,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing - quiet by default, errors always shown
    // Use verbose flag to show info-level messages
    let log_level = if args.verbose {
        tracing::Level::INFO
    } else {
        tracing::Level::WARN
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive(log_level.into()),
        )
        .init();

    // Determine if we're in debug mode first
    let is_debug_mode = args.file.is_some();

    // Determine which files to process
    let json_files = if let Some(file) = args.file {
        // Single-file debug mode
        info!("🔍 Debug mode: Processing single file {}", file.display());
        vec![file]
    } else if let Some(dir) = args.dir {
        // Full generation mode - process all files
        info!(
            "📦 Full generation mode: Processing directory {}",
            dir.display()
        );
        collect_json_files(&dir)?
    } else {
        return Err(anyhow::anyhow!("Must specify either --file or --dir"));
    };

    // Load schema validator if needed
    let validator = if args.skip_validation {
        None
    } else {
        Some(load_schema_validator(&args.schema)?)
    };

    // Phase 1: Process all JSON files through the pipeline and register with fn_registry
    info!("📋 Phase 1: Processing expressions through PPI pipeline...");
    let mut registry = PpiFunctionRegistry::new();
    let mut processed_files = Vec::new();
    let mut expression_to_files: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for json_file in &json_files {
        match process_json_file_phase1(json_file, &mut registry, &validator, args.verbose) {
            Ok(processed) => {
                // Track which files use which expressions for duplicate detection
                expression_to_files
                    .entry(processed.config.expression.clone())
                    .or_default()
                    .push(json_file.clone());

                processed_files.push(processed);
                debug!("✅ Processed: {}", json_file.display());
            }
            Err(e) => {
                warn!("❌ Failed: {} - {}", json_file.display(), e);
                if is_debug_mode {
                    return Err(e);
                }
            }
        }
    }

    // Warn about duplicate expressions
    for (expr, files) in &expression_to_files {
        if files.len() > 1 {
            warn!(
                "⚠️  Expression '{}' appears in multiple files: {:?}",
                expr, files
            );
        }
    }

    // Phase 2: Generate function files
    info!("🔧 Phase 2: Generating function files...");
    let mut generated_files = registry.generate_function_files_with_imports(
        "crate::core::{TagValue, ExifContext, abs, atan2, cos, exp, int, log, sin, sqrt, length_string, length_i32}",
    )?;

    // In debug mode, filter out mod.rs files to avoid breaking other tests
    if is_debug_mode {
        debug!("📝 Debug mode: Skipping mod.rs updates");
        generated_files.retain(|f| !f.path.ends_with("mod.rs"));
    }

    // Write function files to disk
    let functions_dir = args.output.join("functions");
    fs::create_dir_all(&functions_dir)?;

    for file in &generated_files {
        let file_path = args.output.join(&file.path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, &file.content)?;
        debug!("📄 Generated: {}", file_path.display());
    }

    // Phase 3: Generate test files
    info!("🧪 Phase 3: Generating test files...");
    let mut test_modules = HashMap::new();

    for processed in &processed_files {
        let test_file_path = generate_test_file_phase3(&processed, &args.output, is_debug_mode)?;

        // Track modules for mod.rs generation
        if let Some(parent) = test_file_path.parent() {
            if let Some(module_name) = parent.file_name() {
                let module_name = module_name.to_string_lossy().to_string();
                if module_name != "generated" {
                    test_modules
                        .entry(module_name.clone())
                        .or_insert_with(Vec::new)
                        .push(
                            test_file_path
                                .file_stem()
                                .unwrap()
                                .to_string_lossy()
                                .to_string(),
                        );
                }
            }
        }

        debug!("🧪 Generated: {}", test_file_path.display());
    }

    // Generate mod.rs files (skip in debug mode)
    if !is_debug_mode {
        generate_mod_files(&args.output, &test_modules)?;
    }

    info!("🎉 Success! Generated {} test files", processed_files.len());
    if is_debug_mode {
        info!("📝 Debug mode: mod.rs files were preserved");
    }
    info!("🚀 Run tests with: cargo test -p codegen");

    Ok(())
}

/// Process a JSON file through phase 1: parse, generate AST, normalize, register
fn process_json_file_phase1(
    json_file: &Path,
    registry: &mut PpiFunctionRegistry,
    validator: &Option<Validator>,
    verbose: bool,
) -> Result<ProcessedTestFile> {
    info!("Processing JSON file: {:?}", json_file);

    // Read and parse JSON
    let json_content = fs::read_to_string(json_file)
        .with_context(|| format!("Failed to read JSON file: {:?}", json_file))?;

    let json_value: serde_json::Value =
        serde_json::from_str(&json_content).context("Failed to parse JSON")?;

    // Validate against schema if provided
    if let Some(validator) = validator {
        if let Err(validation_error) = validator.validate(&json_value) {
            return Err(anyhow::anyhow!(
                "JSON schema validation failed: {}",
                validation_error
            ));
        }
        debug!("✅ Schema validation passed");
    }

    // Deserialize to typed struct
    let config: ExpressionTestFile = serde_json::from_value(json_value)
        .context("Failed to deserialize JSON to ExpressionTestFile")?;

    if verbose {
        debug!("📄 Processing: {}", json_file.display());
        debug!("   Expression: {}", config.expression);
        debug!("   Type: {:?}", config.expr_type);
    }

    // Step 1: Call ppi_ast.pl to get raw AST
    let raw_ast_json = call_ppi_ast_script(&config.expression)
        .with_context(|| format!("Failed to parse expression: {}", config.expression))?;

    // Step 2: Parse JSON into PpiNode
    let raw_ast: PpiNode =
        serde_json::from_str(&raw_ast_json).context("Failed to parse PPI AST JSON")?;

    if verbose {
        debug!("     ✅ Generated AST");
    }

    // Step 3: Apply normalization
    let normalized_ast = normalize_multi_pass(raw_ast);

    if verbose {
        debug!("     ✅ Normalized AST");
    }

    // Step 4: Register with fn_registry
    let function_spec = registry.register_ast(
        &normalized_ast,
        config.expr_type,
        &config.expression,
        None, // No usage context for test expressions
    )?;

    if verbose {
        debug!("     ✅ Registered as: {}", function_spec.function_name);
    }

    Ok(ProcessedTestFile {
        json_path: json_file.to_path_buf(),
        config,
        function_spec,
    })
}

/// Generate test file in phase 3
fn generate_test_file_phase3(
    processed: &ProcessedTestFile,
    output_dir: &Path,
    _is_debug_mode: bool,
) -> Result<PathBuf> {
    // Determine output path based on expression type
    let type_dir = match processed.config.expr_type {
        ExpressionType::ValueConv => "value_conv",
        ExpressionType::PrintConv => "print_conv",
        ExpressionType::Condition => "conditions",
    };

    let output_subdir = output_dir.join(type_dir);
    fs::create_dir_all(&output_subdir)?;

    // Generate output filename from input filename
    let output_filename = processed
        .json_path
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("Invalid filename: {:?}", processed.json_path))?
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Non-UTF8 filename: {:?}", processed.json_path))?;

    let output_file = output_subdir.join(format!("{}.rs", output_filename));

    // Generate the test file content
    let test_content = generate_test_file_content(processed)?;

    // Write to output file
    fs::write(&output_file, test_content)
        .with_context(|| format!("Failed to write output file: {:?}", output_file))?;

    Ok(output_file)
}

/// Generate the complete Rust test file content
fn generate_test_file_content(processed: &ProcessedTestFile) -> Result<String> {
    let config = &processed.config;
    let func_spec = &processed.function_spec;

    // Generate file header
    let description_line = if let Some(ref description) = config.description {
        format!("//! Description: {}\n", description)
    } else {
        String::new()
    };

    let reference_line = if let Some(ref reference) = config.exiftool_reference {
        format!("//! ExifTool reference: {}\n", reference)
    } else {
        String::new()
    };

    let usage_examples_line = if let Some(ref examples) = config.usage_examples {
        format!("//! Usage examples: {}\n", examples.join(", "))
    } else {
        String::new()
    };

    // Import the generated function
    let hash_prefix = func_spec
        .function_name
        .split('_')
        .last()
        .and_then(|h| h.get(0..2))
        .unwrap_or("00");

    let is_placeholder = config.placeholder_expected.unwrap_or(false);
    let imports = if is_placeholder {
        "use anyhow::Result;\nuse crate::core::{TagValue, ExifContext, missing};\nuse codegen::ppi::ExpressionType;"
    } else {
        "use anyhow::Result;\nuse crate::core::{TagValue, ExifContext};"
    };

    // Properly format multiline expressions in comments
    let expression_comment = config
        .expression
        .lines()
        .map(|line| format!("//! {}", line))
        .collect::<Vec<_>>()
        .join("\n");

    let mut content = formatdoc! {r#"
        //! Generated expression tests for:
        {}
        //! Source: {}
        {}{}{}//! DO NOT EDIT - Regenerate with: make generate-expression-tests

        #![allow(unused_imports)]

        {}
        use super::super::functions::hash_{}::{};

    "#, 
        expression_comment,
        processed.json_path.display(),
        description_line,
        reference_line,
        usage_examples_line,
        imports,
        hash_prefix,
        func_spec.function_name
    };

    let module_name =
        sanitize_identifier(&processed.json_path.file_stem().unwrap().to_string_lossy());

    // Generate test cases - either provided ones or automatic placeholder tests
    let test_cases = if let Some(ref cases) = config.test_cases {
        cases.clone()
    } else if is_placeholder {
        generate_automatic_placeholder_test_cases(&config.expr_type)
    } else {
        return Err(anyhow::anyhow!(
            "No test cases provided and placeholder_expected is false"
        ));
    };

    // Generate test for each test case
    for (i, test_case) in test_cases.iter().enumerate() {
        let test_num = i + 1;

        // Test description
        let test_desc = if let Some(ref desc) = test_case.description {
            format!("/// Test case {}: {}", test_num, desc)
        } else {
            format!("/// Test case {}", test_num)
        };

        let test_type_suffix = match config.expr_type {
            ExpressionType::ValueConv => "valueconv",
            ExpressionType::PrintConv => "printconv",
            ExpressionType::Condition => "condition",
        };

        // Format expression for single-line comment (escape multiline)
        let expression_for_comment = config
            .expression
            .replace('\n', " ")
            .chars()
            .take(100)
            .collect::<String>();
        let expression_for_comment = if expression_for_comment.len() < config.expression.len() {
            format!("{}...", expression_for_comment)
        } else {
            expression_for_comment
        };

        content.push_str(&formatdoc! {r#"
            {}
            /// Expression: {}
            #[test]
            fn test_{}_{}_case_{}() -> Result<()> {{
                // Test inputs
                let input = {};
                let expected = {};

        "#,
            test_desc,
            expression_for_comment,
            module_name,
            test_type_suffix,
            test_num,
            generate_tag_value_constructor(&test_case.input),
            generate_tag_value_constructor(&test_case.expected)
        });

        // Call the function and assert based on type
        if is_placeholder {
            // For placeholder functions, test that they return input unchanged and track missing conversions
            match config.expr_type {
                ExpressionType::ValueConv => {
                    content.push_str(&formatdoc! {r#"
                            // Clear any previous conversions for clean test
                            missing::clear_missing_conversions();

                            // Execute the generated function (ValueConv returns Result)
                            let result = {}(&input, None).unwrap_or_else(|_| input.clone());

                            // For ValueConv placeholders, should return input unchanged
                            assert_eq!(
                                result, expected,
                                "ValueConv placeholder should return input unchanged. Got {{:?}}, expected {{:?}}",
                                result, expected
                            );

                            // Verify the missing conversion was tracked
                            let missing_conversions = missing::get_missing_conversions();
                            assert!(
                                !missing_conversions.is_empty(),
                                "ValueConv placeholder should track missing conversion"
                            );
                    "#,
                        func_spec.function_name
                    });
                }
                ExpressionType::PrintConv => {
                    content.push_str(&formatdoc! {r#"
                            // Clear any previous conversions for clean test
                            missing::clear_missing_conversions();

                            // Execute the generated function (PrintConv returns TagValue)
                            let result = {}(&input, None);

                            // For PrintConv placeholders, should return input unchanged
                            assert_eq!(
                                result, expected,
                                "PrintConv placeholder should return input unchanged. Got {{:?}}, expected {{:?}}",
                                result, expected
                            );

                            // Verify the missing conversion was tracked
                            let missing_conversions = missing::get_missing_conversions();
                            assert!(
                                !missing_conversions.is_empty(),
                                "PrintConv placeholder should track missing conversion"
                            );
                    "#,
                        func_spec.function_name
                    });
                }
                ExpressionType::Condition => {
                    content.push_str(&formatdoc! {r#"
                            // Clear any previous conversions for clean test
                            missing::clear_missing_conversions();

                            // Create context for condition evaluation
                            let ctx = ExifContext::default();

                            // Execute the generated function (Condition returns bool)
                            let result = {}(&input, Some(&ctx));

                            // For condition placeholders, should return false
                            assert_eq!(
                                result, false,
                                "Condition placeholder should return false. Got {{:?}}",
                                result
                            );

                            // Verify the missing conversion was tracked
                            let missing_conversions = missing::get_missing_conversions();
                            assert!(
                                !missing_conversions.is_empty(),
                                "Condition placeholder should track missing conversion"
                            );
                    "#,
                        func_spec.function_name
                    });
                }
            }
        } else {
            match config.expr_type {
                ExpressionType::ValueConv => {
                    content.push_str(&formatdoc! {r#"
                            // Execute the generated function (with None for context)
                            let result = {}(&input, None)?;

                            // Validate the result
                            assert_eq!(
                                result, expected,
                                "ValueConv failed for input {{:?}}. Got {{:?}}, expected {{:?}}",
                                input, result, expected
                            );
                    "#,
                        func_spec.function_name
                    });
                }
                ExpressionType::PrintConv => {
                    content.push_str(&formatdoc! {r#"
                            // Execute the generated function (with None for context)
                            let result = {}(&input, None);

                            // Validate the result
                            assert_eq!(
                                result, expected,
                                "PrintConv failed for input {{:?}}. Got {{:?}}, expected {{:?}}",
                                input, result, expected
                            );
                    "#,
                        func_spec.function_name
                    });
                }
                ExpressionType::Condition => {
                    content.push_str(&formatdoc! {r#"
                            // Create context for condition evaluation
                            let ctx = ExifContext::default();

                            // Execute the generated function
                            let result = {}(&input, Some(&ctx));

                            // For conditions, expected should be a boolean
                            let expected_bool = match expected {{
                                TagValue::Bool(b) => b,
                                TagValue::Empty | TagValue::U32(0) => false,
                                _ => true, // Non-zero/non-empty values are truthy
                            }};
                            assert_eq!(
                                result, expected_bool,
                                "Condition failed for input {{:?}}. Got {{}}, expected {{}}",
                                input, result, expected_bool
                            );
                    "#,
                        func_spec.function_name
                    });
                }
            }
        }

        content.push_str(&formatdoc! {r#"

                Ok(())
            }}
        "#});

        if i < test_cases.len() - 1 {
            content.push_str("\n");
        }
    }

    Ok(content)
}

/// Collect all JSON files recursively from a directory
fn collect_json_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let pattern = dir.join("**/*.json");
    let pattern_str = pattern
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid directory path: {:?}", dir))?;

    let mut files = Vec::new();
    for entry in glob(pattern_str)? {
        let path = entry?;
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip schema files and any other non-test files
        if filename == "schema.json" {
            continue;
        }

        // Skip files with SKIP_ prefix (for temporarily disabling broken tests)
        if filename.starts_with("SKIP_") {
            info!("⏭️  Skipping disabled test: {}", path.display());
            continue;
        }

        files.push(path);
    }

    files.sort();
    Ok(files)
}

/// Load and compile JSON schema validator
fn load_schema_validator(schema_path: &Path) -> Result<Validator> {
    let schema_content = fs::read_to_string(schema_path)
        .with_context(|| format!("Failed to read schema file: {:?}", schema_path))?;

    let schema: serde_json::Value =
        serde_json::from_str(&schema_content).context("Failed to parse schema JSON")?;

    jsonschema::validator_for(&schema).context("Failed to compile JSON schema validator")
}

/// Generate mod.rs files for test organization
fn generate_mod_files(
    output_dir: &Path,
    test_modules: &HashMap<String, Vec<String>>,
) -> Result<()> {
    // Generate main mod.rs
    let mut module_names: Vec<_> = test_modules.keys().collect();
    module_names.sort();

    let module_declarations = module_names
        .iter()
        .map(|name| format!("pub mod {};", name))
        .collect::<Vec<_>>()
        .join("\n");

    let main_mod_content = formatdoc! {r#"
        //! Generated expression test modules
        //! DO NOT EDIT - Regenerate with: make generate-expression-tests

        pub mod functions;

        {}
    "#, module_declarations};

    fs::write(output_dir.join("mod.rs"), main_mod_content)?;

    // Generate mod.rs for each test type directory
    for (module_name, test_files) in test_modules {
        let module_dir = output_dir.join(module_name);

        let mut sorted_files = test_files.clone();
        sorted_files.sort();

        let file_declarations = sorted_files
            .iter()
            .map(|name| format!("pub mod {};", name))
            .collect::<Vec<_>>()
            .join("\n");

        let mod_content = formatdoc! {r#"
            //! Generated expression tests
            //! DO NOT EDIT - Regenerate with: make generate-expression-tests

            {}
        "#, file_declarations};

        fs::write(module_dir.join("mod.rs"), mod_content)?;
    }

    debug!("📄 Generated mod.rs files");

    Ok(())
}

/// Generate automatic test cases for placeholder functions
fn generate_automatic_placeholder_test_cases(expr_type: &ExpressionType) -> Vec<TestCase> {
    match expr_type {
        ExpressionType::ValueConv => vec![
            TestCase {
                description: Some("String input should pass through unchanged".to_string()),
                input: TaggedTagValue::String("test string".to_string()),
                expected: TaggedTagValue::String("test string".to_string()),
            },
            TestCase {
                description: Some("Numeric input should pass through unchanged".to_string()),
                input: TaggedTagValue::U32(42),
                expected: TaggedTagValue::U32(42),
            },
        ],
        ExpressionType::PrintConv => vec![
            TestCase {
                description: Some("String input should pass through unchanged".to_string()),
                input: TaggedTagValue::String("test value".to_string()),
                expected: TaggedTagValue::String("test value".to_string()),
            },
            TestCase {
                description: Some("Numeric input should pass through unchanged".to_string()),
                input: TaggedTagValue::F64(123.45),
                expected: TaggedTagValue::F64(123.45),
            },
        ],
        ExpressionType::Condition => vec![TestCase {
            description: Some("Condition placeholder should return false".to_string()),
            input: TaggedTagValue::String("any value".to_string()),
            expected: TaggedTagValue::Bool(false),
        }],
    }
}

/// Sanitize a string to be a valid Rust identifier
fn sanitize_identifier(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .to_lowercase()
}
