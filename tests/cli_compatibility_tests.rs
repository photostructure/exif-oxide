//! Integration tests for CLI ExifTool compatibility features
//! 
//! These tests verify that our CLI correctly handles ExifTool-compatible flags
//! and edge cases in argument parsing.

use std::process::Command;

/// Helper function to run the exif-oxide binary with given arguments
fn run_exif_oxide(args: &[&str]) -> std::process::Output {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--"]);
    cmd.args(args);
    cmd.output().expect("Failed to execute exif-oxide")
}

/// Helper function to run exif-oxide and capture just stdout
fn run_exif_oxide_stdout(args: &[&str]) -> String {
    let output = run_exif_oxide(args);
    String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout")
}


#[test]
fn test_version_flag_exiftool_style() {
    // Test -ver flag (ExifTool compatibility)
    let output = run_exif_oxide_stdout(&["-ver"]);
    
    // Should just print the version number and nothing else
    assert!(output.trim().ends_with("-dev") || output.trim().contains("."));
    assert!(!output.contains("exif-oxide")); // Should not include app name
    assert_eq!(output.trim().lines().count(), 1); // Should be just one line
}

#[test]
fn test_version_flag_standard_style() {
    // Test --version flag (standard Unix style)
    let output = run_exif_oxide_stdout(&["--version"]);
    
    // Should include app name and version
    assert!(output.contains("exif-oxide"));
    assert!(output.trim().contains(".") || output.trim().contains("-dev"));
    assert_eq!(output.trim().lines().count(), 1); // Should be just one line
}

#[test]
fn test_help_contains_compatibility_info() {
    // Test that --help shows ExifTool compatibility information
    let output = run_exif_oxide_stdout(&["--help"]);
    
    // Should contain ExifTool compatibility section
    assert!(output.contains("EXIFTOOL COMPATIBILITY"));
    assert!(output.contains("-ver"));
    assert!(output.contains("-j, -struct, -G"));
    assert!(output.contains("Ignored"));
}

#[test]
fn test_compatibility_flags_ignored() {
    // Test that -j, -struct, -G flags are ignored and don't cause errors
    // We need a test image file for this, so let's create a minimal test
    let test_file = "/tmp/test_nonexistent.jpg";
    
    let output = run_exif_oxide(&[test_file, "-j", "-struct", "-G", "-FNumber"]);
    
    // Should not fail due to the compatibility flags
    // The error should be about the missing file, not invalid flags
    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");
    assert!(stderr.contains("File not found") || stderr.contains("No files specified"));
    assert!(!stderr.contains("invalid") || !stderr.contains("unknown"));
}

#[test] 
fn test_short_invalid_filters_error() {
    // Test that short invalid filters like -a, -xy cause errors
    let test_file = "/tmp/test_nonexistent.jpg";
    
    let output = run_exif_oxide(&[test_file, "-a"]);
    
    // Should fail due to the short invalid flag
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");
    assert!(stderr.contains("Unknown option -a"));
}

#[test]
fn test_mixed_compatibility_and_valid_flags() {
    // Test mixing compatibility flags with valid tag filters
    let test_file = "/tmp/test_nonexistent.jpg";
    
    let output = run_exif_oxide(&[
        test_file, 
        "-j",           // compatibility flag (ignored)
        "-FNumber",     // valid tag filter
        "-struct",      // compatibility flag (ignored)
        "-G",           // compatibility flag (ignored)
        "-Orientation#" // valid numeric tag filter
    ]);
    
    // Should not fail due to the compatibility flags
    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");
    assert!(stderr.contains("File not found") || stderr.contains("No files specified"));
    assert!(!stderr.contains("invalid") || !stderr.contains("unknown"));
}

#[test]
fn test_debug_logging_for_compatibility_flags() {
    // Test that debug logging shows ignored flags when RUST_LOG=debug
    let test_file = "/tmp/test_nonexistent.jpg";
    
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--"]);
    cmd.args([test_file, "-j", "-struct", "-G"]);
    cmd.env("RUST_LOG", "debug");
    
    let output = cmd.output().expect("Failed to execute exif-oxide");
    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");
    
    // Should contain debug messages about ignoring flags
    assert!(stderr.contains("Ignoring ExifTool compatibility flag: -j") ||
            stderr.contains("compatibility flag"));
}

#[test]
fn test_short_invalid_filters_exit_with_error() {
    // Test that short invalid filters cause immediate error exit
    let test_file = "/tmp/test_nonexistent.jpg";
    
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--"]);
    cmd.args([test_file, "-xy"]);
    
    let output = cmd.output().expect("Failed to execute exif-oxide");
    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");
    
    // Should contain error message about unknown option
    assert!(stderr.contains("Unknown option -xy"));
    assert!(!output.status.success());
}

#[test]
fn test_no_args_shows_error() {
    // Test that running with no arguments shows an appropriate error
    let output = run_exif_oxide(&[]);
    
    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");
    assert!(stderr.contains("required") || stderr.contains("usage") || stderr.contains("help"));
    assert!(!output.status.success());
}

#[test]
fn test_only_compatibility_flags_shows_error() {
    // Test that running with only compatibility flags shows error about no files
    let output = run_exif_oxide(&["-j", "-struct", "-G"]);
    
    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");
    assert!(stderr.contains("No files specified") || stderr.contains("required"));
    assert!(!output.status.success());
}

#[test]
fn test_invalid_short_flags_show_unknown_option_error() {
    // Test that running with invalid short flags shows "Unknown option" error
    let output = run_exif_oxide(&["-y"]);
    
    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");
    assert!(stderr.contains("Unknown option -y"));
    assert!(!output.status.success());
}

#[test]
fn test_version_flags_exit_successfully() {
    // Test that version flags exit with success code
    let output_ver = run_exif_oxide(&["-ver"]);
    assert!(output_ver.status.success());
    
    let output_version = run_exif_oxide(&["--version"]);
    assert!(output_version.status.success());
}

#[test]
fn test_help_flag_exits_successfully() {
    // Test that --help exits with success code
    let output = run_exif_oxide(&["--help"]);
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Options:"));
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_version_flag_with_other_args() {
        // Test that -ver flag works even when other args are present
        // Version should take precedence and exit immediately
        let output = run_exif_oxide_stdout(&["-ver", "image.jpg", "-FNumber"]);
        
        // Should just show version, not process other args
        assert!(output.trim().ends_with("-dev") || output.trim().contains("."));
        assert!(!output.contains("Error"));
        assert!(!output.contains("File not found"));
    }

    #[test]
    fn test_compatibility_flags_case_sensitivity() {
        // Test that compatibility flags are case-sensitive
        let test_file = "/tmp/test_nonexistent.jpg";
        
        // Test that -STRUCT is NOT treated as a compatibility flag (case sensitive)
        let output = run_exif_oxide(&[test_file, "-STRUCT"]);
        let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");
        
        // Should work fine (STRUCT would be treated as a regular filter)
        assert!(stderr.contains("File not found"));
        
        // Test that short uppercase versions cause errors (not compatibility flags)
        let output2 = run_exif_oxide(&[test_file, "-J"]);
        let stderr2 = String::from_utf8(output2.stderr).expect("Invalid UTF-8");
        assert!(stderr2.contains("Unknown option -J"));
    }

    #[test]
    fn test_exactly_three_char_filter_valid() {
        // Test that exactly 3-character filters are valid (not rejected as short)
        let test_file = "/tmp/test_nonexistent.jpg";
        
        let output = run_exif_oxide(&[test_file, "-GPS"]);
        let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");
        
        // Should work fine - GPS is a valid 3-character filter
        assert!(stderr.contains("File not found"));
        assert!(!stderr.contains("invalid"));
    }

    #[test]
    fn test_two_char_filter_rejected() {
        // Test that exactly 2-character filters cause errors
        let test_file = "/tmp/test_nonexistent.jpg";
        
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "--quiet", "--"]);
        cmd.args([test_file, "-AB"]);
        
        let output = cmd.output().expect("Failed to execute exif-oxide");
        let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");
        
        // Should contain error message about unknown option
        assert!(stderr.contains("Unknown option -AB"));
        assert!(!output.status.success());
    }
}