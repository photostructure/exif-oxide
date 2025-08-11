#!/bin/bash

# Make sure our working directory is the project root:
cd "$(dirname "${BASH_SOURCE[0]}")/.."

# Careful! This emits 4,000 lines of JSON!

for file in $(jq -r '.modules | to_entries | map(.value[]) | .[]' config/exiftool_modules.json); do
  module_name=$(echo "$file" | sed 's|.*/||' | sed 's|\.pm$||')
  perl ./codegen/scripts/field_extractor.pl "third-party/exiftool/$file" | jq --arg mod "$module_name" '.data[] | if type == "array" then .[] else . end | select(type == "object" and has("Condition")) | {Name: ($mod + "." + .Name), Condition: .Condition}'
done | jq -s 'group_by(.Condition) | map({Condition: .[0].Condition, Names: [.[].Name]}) | sort_by(.Condition | length)'
