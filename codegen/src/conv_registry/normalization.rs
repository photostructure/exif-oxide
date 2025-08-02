//! Expression normalization using Perl for consistent lookups
//!
//! This module handles normalization of Perl expressions to ensure consistent
//! registry lookups. It uses the Perl interpreter to properly parse and
//! normalize expressions.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::process::Command;

// Cache for normalized expressions to avoid repeated subprocess calls
static NORMALIZATION_CACHE: LazyLock<Mutex<HashMap<String, String>>> = 
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Normalize expression for consistent lookup
/// Uses Perl to normalize Perl expressions
pub fn normalize_expression(expr: &str) -> String {
    // Check cache first
    if let Ok(cache) = NORMALIZATION_CACHE.lock() {
        if let Some(normalized) = cache.get(expr) {
            return normalized.clone();
        }
    }
    
    // Use Perl normalization
    let normalized = match normalize_with_perl(expr) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Warning: Failed to normalize expression '{}': {}", expr, e);
            eprintln!("Using original expression");
            expr.to_string()
        }
    };
    
    // Cache the result
    if let Ok(mut cache) = NORMALIZATION_CACHE.lock() {
        cache.insert(expr.to_string(), normalized.clone());
    }
    
    normalized
}

/// Batch normalize multiple expressions in a single Perl call
/// This is much more efficient than calling normalize_expression repeatedly
pub fn batch_normalize_expressions(expressions: &[String]) -> Result<HashMap<String, String>, String> {
    // Filter out expressions that are already cached
    let uncached: Vec<String> = {
        let cache = NORMALIZATION_CACHE.lock().map_err(|_| "Cache lock failed")?;
        expressions.iter()
            .filter(|expr| !cache.contains_key(*expr))
            .cloned()
            .collect()
    };
    
    if uncached.is_empty() {
        // All expressions are cached, return cached results
        let cache = NORMALIZATION_CACHE.lock().map_err(|_| "Cache lock failed")?;
        return Ok(expressions.iter()
            .filter_map(|expr| cache.get(expr).map(|normalized| (expr.clone(), normalized.clone())))
            .collect());
    }
    
    // Batch normalize uncached expressions
    let batch_results = normalize_batch_with_perl(&uncached)?;
    
    // Update cache with new results
    {
        let mut cache = NORMALIZATION_CACHE.lock().map_err(|_| "Cache lock failed")?;
        for (original, normalized) in &batch_results {
            cache.insert(original.clone(), normalized.clone());
        }
    }
    
    // Return all results (cached + new)
    let cache = NORMALIZATION_CACHE.lock().map_err(|_| "Cache lock failed")?;
    Ok(expressions.iter()
        .filter_map(|expr| cache.get(expr).map(|normalized| (expr.clone(), normalized.clone())))
        .collect())
}

/// Call Perl script to normalize multiple expressions in batch
fn normalize_batch_with_perl(expressions: &[String]) -> Result<HashMap<String, String>, String> {
    use std::io::Write;
    use std::process::Stdio;
    
    // Find the normalize script by searching up from current directory
    let script_path = find_normalize_script()
        .ok_or_else(|| "Could not find normalize_expression.pl script".to_string())?;
    
    // Set up Perl environment for local::lib
    let home_dir = std::env::var("HOME")
        .map_err(|_| "HOME environment variable not set".to_string())?;
    let perl5lib = format!("{}/perl5/lib/perl5", home_dir);
    
    // Call the Perl script with stdin and proper environment
    let mut child = Command::new("perl")
        .arg("-I")
        .arg(&perl5lib)
        .arg("-Mlocal::lib")
        .arg(&script_path)
        .env("PERL5LIB", &perl5lib)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute Perl: {}", e))?;
    
    // Write all expressions to stdin, separated by the delimiter
    if let Some(mut stdin) = child.stdin.take() {
        let batch_input = expressions.join("\n\n\n\n");
        stdin.write_all(batch_input.as_bytes())
            .map_err(|e| format!("Failed to write to Perl stdin: {}", e))?;
    }
    
    // Wait for completion and get output
    let output = child.wait_with_output()
        .map_err(|e| format!("Failed to wait for Perl process: {}", e))?;
    
    if output.status.success() {
        let stdout_str = String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8 in Perl output: {}", e))?;
        
        // Parse the batch response - each normalized expression is separated by the same delimiter
        let normalized_expressions: Vec<&str> = stdout_str.split("\n\n\n\n").collect();
        
        if normalized_expressions.len() != expressions.len() {
            return Err(format!(
                "Batch normalization mismatch: sent {} expressions, got {} results",
                expressions.len(),
                normalized_expressions.len()
            ));
        }
        
        // Create mapping from original to normalized
        let mut results = HashMap::new();
        for (original, normalized) in expressions.iter().zip(normalized_expressions.iter()) {
            results.insert(original.clone(), normalized.trim().to_string());
        }
        
        Ok(results)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Perl script failed: {}", stderr))
    }
}

/// Call Perl script to normalize expression
fn normalize_with_perl(expr: &str) -> Result<String, String> {
    use std::io::Write;
    use std::process::Stdio;
    
    // Find the normalize script by searching up from current directory
    let script_path = find_normalize_script()
        .ok_or_else(|| "Could not find normalize_expression.pl script".to_string())?;
    
    // Set up Perl environment for local::lib
    let home_dir = std::env::var("HOME")
        .map_err(|_| "HOME environment variable not set".to_string())?;
    let perl5lib = format!("{}/perl5/lib/perl5", home_dir);
    
    // Call the Perl script with stdin and proper environment
    let mut child = Command::new("perl")
        .arg("-I")
        .arg(&perl5lib)
        .arg("-Mlocal::lib")
        .arg(&script_path)
        .env("PERL5LIB", &perl5lib)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute Perl: {}", e))?;
    
    // Write expression to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(expr.as_bytes())
            .map_err(|e| format!("Failed to write to Perl stdin: {}", e))?;
    }
    
    // Wait for completion and get output
    let output = child.wait_with_output()
        .map_err(|e| format!("Failed to wait for Perl process: {}", e))?;
    
    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8 in Perl output: {}", e))
            .map(|s| s.trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Perl script failed: {}", stderr))
    }
}

/// Find the normalize_expression.pl script by searching up the directory tree
fn find_normalize_script() -> Option<std::path::PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    
    // Search up to 5 levels
    for _ in 0..5 {
        // Check if we're in the codegen directory
        let script_path = current.join("extractors").join("normalize_expression.pl");
        if script_path.exists() {
            return Some(script_path);
        }
        
        // Check if we're in the project root
        let codegen_script = current.join("codegen").join("extractors").join("normalize_expression.pl");
        if codegen_script.exists() {
            return Some(codegen_script);
        }
        
        // Move up one directory
        current = current.parent()?.to_path_buf();
    }
    
    None
}