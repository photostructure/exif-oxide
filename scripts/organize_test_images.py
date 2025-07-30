#!/usr/bin/env python3
"""
Organize test images from tmp/ into test-images/ directory structure.
Format: test-images/{make}/{model}.{ext}
"""

import os
import shutil
import subprocess
import hashlib
from pathlib import Path
from collections import defaultdict

def get_file_hash(filepath):
    """Calculate MD5 hash of a file."""
    hash_md5 = hashlib.md5()
    with open(filepath, "rb") as f:
        for chunk in iter(lambda: f.read(4096), b""):
            hash_md5.update(chunk)
    return hash_md5.hexdigest()

def get_exif_info(filepath):
    """Get Make and Model from EXIF data using exiftool."""
    try:
        result = subprocess.run(
            ["exiftool", "-s", "-Make", "-Model", str(filepath)],
            capture_output=True,
            text=True
        )
        
        make = None
        model = None
        
        for line in result.stdout.strip().split('\n'):
            if line.startswith('Make'):
                make = line.split(':', 1)[1].strip()
            elif line.startswith('Model'):
                model = line.split(':', 1)[1].strip()
        
        return make, model
    except Exception as e:
        print(f"Error reading {filepath}: {e}")
        return None, None

def sanitize_filename(name):
    """Sanitize make/model names for filesystem use."""
    if not name:
        return None
    
    # Remove/replace problematic characters
    name = name.lower()
    name = name.replace(' ', '_')
    name = name.replace('/', '_')
    name = name.replace(',', '')
    name = name.replace('.', '')
    name = name.replace('corporation', '')
    name = name.replace('computer_co_ltd', '')
    name = name.replace('imaging_company_ltd', '')
    name = name.strip('_')
    
    return name

def main():
    tmp_dir = Path("tmp")
    test_images_dir = Path("test-images")
    
    if not tmp_dir.exists():
        print("tmp/ directory not found!")
        return
    
    # Build hash map of existing files
    existing_hashes = {}
    print("Building hash map of existing test-images...")
    for existing_file in test_images_dir.rglob("*"):
        if existing_file.is_file() and existing_file.suffix != '.md':
            file_hash = get_file_hash(existing_file)
            existing_hashes[file_hash] = existing_file
    
    # Process files in tmp/
    processed = 0
    duplicates = 0
    errors = 0
    
    for filepath in sorted(tmp_dir.glob("*")):
        if not filepath.is_file():
            continue
            
        print(f"\nProcessing: {filepath.name}")
        
        # Check for duplicates
        file_hash = get_file_hash(filepath)
        if file_hash in existing_hashes:
            print(f"  DUPLICATE of {existing_hashes[file_hash]}")
            duplicates += 1
            continue
        
        # Get EXIF info
        make, model = get_exif_info(filepath)
        
        if not make or not model:
            print(f"  ERROR: Could not read Make/Model")
            errors += 1
            continue
        
        print(f"  Make: {make}")
        print(f"  Model: {model}")
        
        # Sanitize names
        make_dir = sanitize_filename(make)
        model_name = sanitize_filename(model)
        
        if not make_dir or not model_name:
            print(f"  ERROR: Invalid make/model after sanitization")
            errors += 1
            continue
        
        # Create destination path
        dest_dir = test_images_dir / make_dir
        dest_dir.mkdir(exist_ok=True)
        
        # Check if this model already exists
        ext = filepath.suffix.lower()
        dest_file = dest_dir / f"{model_name}{ext}"
        
        if dest_file.exists():
            # Find a numbered variant
            counter = 1
            while True:
                dest_file = dest_dir / f"{model_name}_{counter:02d}{ext}"
                if not dest_file.exists():
                    break
                counter += 1
        
        # Copy the file
        print(f"  Copying to: {dest_file}")
        shutil.copy2(filepath, dest_file)
        processed += 1
    
    print(f"\n=== Summary ===")
    print(f"Processed: {processed}")
    print(f"Duplicates: {duplicates}")
    print(f"Errors: {errors}")

if __name__ == "__main__":
    main()