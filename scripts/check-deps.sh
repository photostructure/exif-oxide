#!/bin/bash

# Check for required external tools
# This script verifies that all necessary dependencies are installed

set -e

# Function to check if a command exists and add to missing list if not
check_tool() {
  local tool="$1"
  local install_cmd="$2"

  if ! command -v "$tool" >/dev/null 2>&1; then
    missing="$missing $tool($install_cmd)"
  fi
}

# Initialize missing tools list
missing=""

# Check each required tool
check_tool "git" "sudo apt-get install git"
check_tool "rg" "sudo apt-get install ripgrep or cargo install ripgrep"
check_tool "sd" "cargo install sd"
check_tool "shfmt" "sudo apt-get install shfmt or go install mvdan.cc/sh/v3/cmd/shfmt@latest"
check_tool "jq" "sudo apt-get install jq"
check_tool "yamllint" "pip install yamllint"
check_tool "cargo-upgrade" "cargo install cargo-edit"
check_tool "cargo-audit" "cargo install cargo-audit --locked"

# Report results
if [ -n "$missing" ]; then
  echo "Missing tools: $missing"
  exit 1
fi
