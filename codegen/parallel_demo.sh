#!/bin/bash
#
# Demonstration of parallel codegen execution
# This shows how the modular architecture enables faster builds

set -e

echo "🚀 Parallel Codegen Demonstration"
echo "================================="
echo ""

# Function to time execution
time_execution() {
    local name=$1
    shift
    echo -n "$name: "
    time -p "$@" 2>&1 | grep real | awk '{print $2 "s"}'
}

# Clean to ensure fair comparison
echo "🧹 Cleaning generated files..."
make -f Makefile.modular clean >/dev/null 2>&1

echo ""
echo "⏱️  Sequential Execution (make -j1):"
echo "-----------------------------------"
time_execution "Total time" make -f Makefile.modular -j1 extract-perl >/dev/null 2>&1

# Clean again
make -f Makefile.modular clean >/dev/null 2>&1

echo ""
echo "⚡ Parallel Execution (make -j4):"
echo "--------------------------------"
time_execution "Total time" make -f Makefile.modular -j4 extract-perl >/dev/null 2>&1

echo ""
echo "📊 Individual Extractor Timing:"
echo "------------------------------"
time_execution "Tag tables      " perl extractors/tag_tables.pl >/dev/null 2>&1
time_execution "Composite tags  " perl extractors/composite_tags.pl >/dev/null 2>&1
time_execution "Regex patterns  " perl extractors/regex_patterns.pl >/dev/null 2>&1
time_execution "File type lookup" perl extractors/file_type_lookup.pl >/dev/null 2>&1
time_execution "Simple tables   " perl extractors/simple_tables.pl >/dev/null 2>&1

echo ""
echo "✅ Benefits of Modular Architecture:"
echo "-----------------------------------"
echo "1. Parallel execution reduces build time"
echo "2. Individual extractors can be run independently"
echo "3. Incremental regeneration for faster development"
echo "4. Failed extractors don't block others"
echo "5. Easy to add new extraction types"

echo ""
echo "🔧 Example: Regenerate only tag tables"
echo "--------------------------------------"
echo "$ make -f Makefile.modular regen-tags"
echo ""
echo "This regenerates only the tag tables without"
echo "re-running all other extractors, saving time"
echo "during development."