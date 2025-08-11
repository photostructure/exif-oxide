#!/bin/bash

# Make sure our working directory is the project root:
cd "$(dirname "${BASH_SOURCE[0]}")/.."

for file in $(jq -r '.modules | to_entries | map(.value[]) | .[]' config/exiftool_modules.json); do
  module_name=$(echo "$file" | sed 's|.*/||' | sed 's|\.pm$||')

  # CAREFUL! We're truncating objects and arrays here just so the output is not
  # quite as overwhelming. It's still gigantic: 22k lines of JSON!

  perl ./codegen/scripts/field_extractor.pl "third-party/exiftool/$file" | jq --arg mod "$module_name" '.data[] | if type == "array" then .[0:4][] else . end | select(type == "object" and has("PrintConv")) | {Name: ($mod + "." + .Name), PrintConv: (if (.PrintConv | type) == "object" then (.PrintConv | to_entries | .[0:4] | from_entries) else .PrintConv end)}'
done | jq -s 'group_by(.PrintConv) | map({PrintConv: .[0].PrintConv, Names: [.[].Name], Count: length}) | sort_by(.PrintConv | length)'
