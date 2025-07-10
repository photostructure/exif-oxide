use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Test the patching functionality
    let test_file = Path::new("test_patch.pl");
    
    // Read original content
    let original = fs::read_to_string(test_file)?;
    println!("Original content:");
    println!("{}", original);
    
    // Apply patches
    codegen::patching::patch_module(test_file, &["testHash".to_string(), "anotherHash".to_string()])?;
    
    // Read patched content
    let patched = fs::read_to_string(test_file)?;
    println!("\nPatched content:");
    println!("{}", patched);
    
    // Verify patches were applied
    assert!(patched.contains("our %testHash"));
    assert!(patched.contains("our %anotherHash"));
    assert!(!patched.contains("my %testHash"));
    assert!(!patched.contains("my %anotherHash"));
    
    println!("\nTest passed!");
    
    // Restore original
    fs::write(test_file, original)?;
    
    Ok(())
}