#!/bin/bash
# Redirect stdout and stderr to temp files and echo the paths
# Usage: ./scripts/capture.sh command args...
# If output is ≤20 lines, prints directly; otherwise saves to files

# Use standard temp directory environment variables with fallback
TEMP_DIR="${TMPDIR:-${TMP:-${TEMP:-/tmp}}}"

# Generate unique temp file names based on timestamp and process ID
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
STDOUT_FILE="${TEMP_DIR}/stdout_${TIMESTAMP}_$$.txt"
STDERR_FILE="${TEMP_DIR}/stderr_${TIMESTAMP}_$$.txt"

# Run the command with timing, redirecting outputs
time "$@" > "$STDOUT_FILE" 2> "$STDERR_FILE";
EXIT_CODE=$?

# Function to handle output display
handle_output() {
    local stream_name=$1
    local file_path=$2
    local line_count=$(wc -l < "$file_path")
    
    if [ "$line_count" -le 20 ]; then
        if [ "$line_count" -gt 0 ]; then
            echo "=== $stream_name ($line_count lines) ==="
            cat "$file_path"
        else
            echo "=== $stream_name (empty) ==="
        fi
        rm -f "$file_path"
    else
        echo "$stream_name: $file_path ($line_count lines)"
    fi
}

echo "EXIT_CODE: $EXIT_CODE"

# Handle both outputs
handle_output "STDOUT" "$STDOUT_FILE"
handle_output "STDERR" "$STDERR_FILE"

exit $EXIT_CODE