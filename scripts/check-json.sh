#!/bin/bash

# Validate JSON files against their schemas
# This script validates JSON files in codegen/config/ using their $schema field references

set -e

if ! command -v python3 >/dev/null 2>&1; then
  echo "❌ Error: python3 not found"
  exit 1
fi

if ! python3 -c "import jsonschema" >/dev/null 2>&1; then
  echo "⚠️  Warning: Python jsonschema package not found, skipping JSON schema validation"
  echo "   Install with: pip install jsonschema"
  exit 0
fi

# Function to validate a single JSON file
validate_json_file() {
  local file="$1"
  local schema_ref
  local schema_path
  local file_dir

  # Extract $schema field from the JSON file
  schema_ref=$(jq -r '."$schema" // empty' "$file" 2>/dev/null)

  if [[ -z "$schema_ref" ]]; then
    echo "⚠️  Warning: No \$schema field found in $file"
    return 0
  fi

  # Convert relative schema path to absolute path
  file_dir=$(dirname "$file")
  schema_path=$(realpath "$file_dir/$schema_ref" 2>/dev/null)

  if [[ ! -f "$schema_path" ]]; then
    echo "❌ Error: Schema file not found: $schema_path (referenced from $file)"
    return 1
  fi

  # Validate the JSON file against its schema using Python jsonschema
  validation_result=$(python3 -c "
import json
import sys
import shlex
from jsonschema import validate, ValidationError, Draft7Validator

try:
    file_path = sys.argv[1]
    schema_path = sys.argv[2]
    
    with open(file_path, 'r') as f:
        data = json.load(f)
    with open(schema_path, 'r') as f:
        schema = json.load(f)
    
    validator = Draft7Validator(schema)
    errors = list(validator.iter_errors(data))
    
    if errors:
        print('INVALID')
        for error in errors:
            print(f'Error: {error.message}')
            if error.absolute_path:
                print(f'Path: {\".\".join(str(p) for p in error.absolute_path)}')
    else:
        print('VALID')
        
except Exception as e:
    print(f'ERROR: {e}')
    sys.exit(1)
" "$file" "$schema_path" 2>&1)

  if echo "$validation_result" | head -1 | grep -q "VALID"; then
    # Silent on success - only show warnings and errors
    return 0
  else
    echo "❌ Invalid: $file"
    echo "   Schema: $schema_path"
    # Show detailed validation errors
    echo "$validation_result" | tail -n +2 | sed 's/^/   /'
    return 1
  fi
}

# Export function so xargs can use it
export -f validate_json_file

# Find JSON files in codegen/config and validate them in parallel
# Use -mindepth 2 to skip top-level config files like supported_tags.json.
find ./codegen/config/ -mindepth 2 -name "*.json" -type f -print0 |
  sort -z |
  xargs -0 -P 8 -I {} bash -c 'validate_json_file "$@"' _ {}
