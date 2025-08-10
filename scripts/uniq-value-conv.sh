#!/bin/bash

# Make sure our working directory is the project root:
cd "$(dirname "${BASH_SOURCE[0]}")/.."

# Careful! This emits 4,000 lines of JSON!

for file in $(jq -r '.modules | to_entries | map(.value[]) | .[]' config/exiftool_modules.json); do
  module_name=$(echo "$file" | sed 's|.*/||' | sed 's|\.pm$||')
  perl ./codegen/scripts/field_extractor.pl "third-party/exiftool/$file" | jq --arg mod "$module_name" '.data[] | if type == "array" then .[] else . end | select(type == "object" and has("ValueConv")) | {Name: ($mod + "." + .Name), ValueConv: .ValueConv}'
done | jq -s 'group_by(.ValueConv) | map({ValueConv: .[0].ValueConv, Names: [.[].Name]}) | sort_by(.ValueConv | length)'
