#!/usr/bin/env python3
"""
Analyze all ValueConv, PrintConv, and Condition expressions for required/mainstream tags.
This helps identify what PPI AST patterns the codegen pipeline needs to support.
"""

import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path
from typing import Dict, List, Set, Any
import re

def load_composite_dependencies() -> Dict[str, List[str]]:
    """Load composite tag dependencies from generated JSON file."""
    dependencies = {}
    
    # First check if the JSON file exists
    composite_file = Path('docs/analysis/expressions/composite-dependencies.json')
    if composite_file.exists():
        try:
            with open(composite_file, 'r') as f:
                data = json.load(f)
                
            for tag_name, info in data.get('tags', {}).items():
                deps = []
                # Combine require and desire dependencies
                deps.extend(info.get('require', []))
                deps.extend(info.get('desire', []))
                
                if deps:
                    dependencies[tag_name] = deps
                    
            print(f"Loaded {len(dependencies)} composite tags from {composite_file}", file=sys.stderr)
            return dependencies
        except (json.JSONDecodeError, KeyError) as e:
            print(f"Error loading {composite_file}: {e}", file=sys.stderr)
    
    # If JSON doesn't exist, try to generate it
    print(f"Generating {composite_file}...", file=sys.stderr)
    result = subprocess.run(['./scripts/composite-dependencies.sh'], 
                          capture_output=True, text=True)
    if result.returncode == 0 and composite_file.exists():
        # Retry loading
        return load_composite_dependencies()
    else:
        print("Failed to generate composite dependencies", file=sys.stderr)
        return dependencies

def load_required_tags_with_dependencies() -> Set[str]:
    """Load required tags and their transitive dependencies."""
    required_tags = set()

    # First try to load from required-tags.json (list of tag names)
    required_tags_file = Path('docs/required-tags.json')
    if required_tags_file.exists():
        try:
            with open(required_tags_file, 'r') as f:
                required_list = json.load(f)
            if isinstance(required_list, list):
                required_tags.update(required_list)
                print(f"Loaded {len(required_tags)} tags from {required_tags_file}", file=sys.stderr)
        except (json.JSONDecodeError, KeyError) as e:
            print(f"Error loading {required_tags_file}: {e}", file=sys.stderr)

    # Fall back to / supplement from tag-metadata.json required field
    metadata_file = Path('docs/tag-metadata.json')
    if metadata_file.exists():
        try:
            with open(metadata_file, 'r') as f:
                metadata = json.load(f)
            for tag_name, info in metadata.items():
                if info.get('required', False):
                    required_tags.add(tag_name)
        except (json.JSONDecodeError, KeyError) as e:
            print(f"Error loading {metadata_file}: {e}", file=sys.stderr)

    print(f"Found {len(required_tags)} directly required tags", file=sys.stderr)
    
    # Load composite dependencies
    composite_deps = load_composite_dependencies()
    print(f"Found {len(composite_deps)} composite tags with dependencies", file=sys.stderr)
    
    # Transitively add dependencies of required composite tags
    added = True
    iterations = 0
    while added and iterations < 10:  # Prevent infinite loops
        added = False
        iterations += 1
        current_tags = list(required_tags)
        
        for tag in current_tags:
            if tag in composite_deps:
                for dep in composite_deps[tag]:
                    if dep not in required_tags:
                        required_tags.add(dep)
                        added = True
                        print(f"  Added dependency: {dep} (required by {tag})", file=sys.stderr)
    
    print(f"Total required tags with dependencies: {len(required_tags)}", file=sys.stderr)
    return required_tags

def normalize_tag_name(tag: str) -> str:
    """Normalize tag name for fuzzy matching."""
    return re.sub(r'[^a-z0-9]', '', tag.lower())

def extract_module_expressions(module_path: str, priority_tags: Set[str]) -> Dict[str, List[Dict]]:
    """Extract expressions from a single ExifTool module."""
    module_name = Path(module_path).stem
    
    # Run field_extractor.pl (use binary mode and decode with error handling)
    cmd = ['perl', './codegen/scripts/field_extractor.pl', f'third-party/exiftool/{module_path}']
    try:
        result = subprocess.run(cmd, capture_output=True, check=True)
        # Decode with error handling for binary data
        stdout_text = result.stdout.decode('utf-8', errors='replace')
        
        # field_extractor outputs multiple JSON objects, one per line
        all_data = []
        for line in stdout_text.strip().split('\n'):
            if line:
                try:
                    obj = json.loads(line)
                    all_data.append(obj)
                except json.JSONDecodeError:
                    pass  # Skip invalid lines
                    
    except subprocess.CalledProcessError as e:
        print(f"Error processing {module_path}: {e}", file=sys.stderr)
        return {'ValueConv': [], 'PrintConv': [], 'Condition': []}
    
    # Normalize priority tags for matching
    normalized_priority = {normalize_tag_name(tag) for tag in priority_tags}
    
    expressions = {
        'ValueConv': [],
        'PrintConv': [],
        'Condition': []
    }
    
    # Process each extracted symbol
    for symbol in all_data:
        if not isinstance(symbol, dict):
            continue
            
        # Get the symbol's data
        symbol_data = symbol.get('data', {})
        
        # For hash symbols, process each tag entry
        if symbol.get('type') == 'hash' and isinstance(symbol_data, dict):
            for tag_name, tag_def in symbol_data.items():
                if not isinstance(tag_def, dict):
                    continue
                    
                # Check if this tag is in our priority list (fuzzy match)
                normalized_tag = normalize_tag_name(tag_name)
                if normalized_tag not in normalized_priority:
                    continue
                
                # Collect expressions
                full_name = f"{module_name}.{tag_name}"
                
                for expr_type in ['ValueConv', 'PrintConv', 'Condition']:
                    if expr_type in tag_def:
                        expr = tag_def[expr_type]
                        # Skip simple values that aren't expressions
                        if isinstance(expr, str) and any(c in expr for c in ['$', '(', '?', '/', '"', '=']):
                            expressions[expr_type].append({
                                'tag': full_name,
                                'expression': expr
                            })
        
        # For array symbols with tag data
        elif symbol.get('type') == 'array' and isinstance(symbol_data, list):
            for item in symbol_data:
                if isinstance(item, dict) and 'Name' in item:
                    tag_name = item['Name']
                    
                    # Check if this tag is in our priority list (fuzzy match)
                    normalized_tag = normalize_tag_name(tag_name)
                    if normalized_tag not in normalized_priority:
                        continue
                    
                    # Collect expressions
                    full_name = f"{module_name}.{tag_name}"
                    
                    for expr_type in ['ValueConv', 'PrintConv', 'Condition']:
                        if expr_type in item:
                            expr = item[expr_type]
                            # Skip simple values that aren't expressions
                            if isinstance(expr, str) and any(c in expr for c in ['$', '(', '?', '/', '"', '=']):
                                expressions[expr_type].append({
                                    'tag': full_name,
                                    'expression': expr
                                })
    
    return expressions

def analyze_expression_patterns(expressions: List[Dict]) -> Dict[str, Any]:
    """Analyze patterns in expressions to identify what needs codegen support."""
    patterns = {
        'sprintf': [],
        'unpack': [],
        'pack': [],
        'ternary': [],
        'regex_match': [],
        'regex_substitute': [],
        'arithmetic': [],
        'string_concat': [],
        'function_calls': [],
        'hash_lookups': [],
        'complex': []
    }
    
    for item in expressions:
        expr = str(item['expression'])
        tag = item['tag']
        
        # Classify expression patterns
        if 'sprintf' in expr:
            patterns['sprintf'].append(tag)
        if 'unpack' in expr:
            patterns['unpack'].append(tag)
        if 'pack' in expr:
            patterns['pack'].append(tag)
        if '?' in expr and ':' in expr:
            patterns['ternary'].append(tag)
        if '=~' in expr and '/' in expr:
            if 's/' in expr:
                patterns['regex_substitute'].append(tag)
            else:
                patterns['regex_match'].append(tag)
        if any(op in expr for op in ['+', '-', '*', '/', '%', '**']):
            patterns['arithmetic'].append(tag)
        if '.' in expr and '"' in expr:
            patterns['string_concat'].append(tag)
        if re.search(r'\b\w+\s*\(', expr):
            patterns['function_calls'].append(tag)
        if isinstance(item['expression'], dict) or '{' in expr:
            patterns['hash_lookups'].append(tag)
        
        # Check complexity
        if expr.count('(') > 3 or len(expr) > 100:
            patterns['complex'].append(tag)
    
    # Deduplicate, sort, and count (sorting ensures deterministic output)
    return {k: (sorted(set(v)), len(set(v))) for k, v in patterns.items() if v}

def main():
    """Main entry point."""
    # Ensure we're in project root
    project_root = Path(__file__).parent.parent
    import os
    os.chdir(project_root)
    
    # Run patcher first
    print("Running ExifTool patcher...", file=sys.stderr)
    subprocess.run(['./codegen/scripts/exiftool-patcher.sh'], 
                   stderr=subprocess.DEVNULL, check=False)
    
    # Load priority tags with transitive dependencies
    all_priority_tags = load_required_tags_with_dependencies()
    
    # Load modules
    with open('config/exiftool_modules.json', 'r') as f:
        modules_config = json.load(f)
    
    # Flatten module paths
    module_paths = []
    for group_modules in modules_config['modules'].values():
        module_paths.extend(group_modules)
    
    print(f"Processing {len(module_paths)} modules...", file=sys.stderr)
    
    # Collect all expressions
    all_expressions = {
        'ValueConv': defaultdict(list),
        'PrintConv': defaultdict(list),
        'Condition': defaultdict(list)
    }
    
    for module_path in module_paths:
        expressions = extract_module_expressions(module_path, all_priority_tags)
        
        for expr_type in ['ValueConv', 'PrintConv', 'Condition']:
            for item in expressions[expr_type]:
                expr_str = json.dumps(item['expression']) if isinstance(item['expression'], dict) else str(item['expression'])
                all_expressions[expr_type][expr_str].append(item['tag'])
    
    # Generate output
    output = {
        'summary': {
            'total_required_tags': len(all_priority_tags),
            'modules_processed': len(module_paths)
        },
        'expressions': {}
    }
    
    for expr_type in ['ValueConv', 'PrintConv', 'Condition']:
        # Sort by usage count
        sorted_exprs = sorted(all_expressions[expr_type].items(), 
                            key=lambda x: len(x[1]), reverse=True)
        
        expr_list = []
        for expr, tags in sorted_exprs:
            expr_list.append({
                'expression': expr,
                'count': len(tags),
                'tags': sorted(tags)
            })
        
        output['expressions'][expr_type] = {
            'unique_count': len(sorted_exprs),
            'total_usage': sum(len(tags) for _, tags in sorted_exprs),
            'top_expressions': expr_list[:20],  # Top 20 most common
            'all_expressions': expr_list
        }
        
        # Pattern analysis
        all_items = []
        for expr, tags in all_expressions[expr_type].items():
            for tag in tags:
                all_items.append({'expression': expr, 'tag': tag})
        
        if all_items:
            patterns = analyze_expression_patterns(all_items)
            output['expressions'][expr_type]['patterns'] = patterns
    
    # Find expressions that appear in multiple types
    all_expr_strings = set()
    for expr_type in ['ValueConv', 'PrintConv', 'Condition']:
        all_expr_strings.update(all_expressions[expr_type].keys())
    
    shared = []
    for expr in all_expr_strings:
        types_used = []
        for expr_type in ['ValueConv', 'PrintConv', 'Condition']:
            if expr in all_expressions[expr_type]:
                types_used.append(expr_type)
        if len(types_used) > 1:
            shared.append({
                'expression': expr,
                'used_in': sorted(types_used),
                'tags': {
                    expr_type: sorted(all_expressions[expr_type].get(expr, []))
                    for expr_type in types_used
                }
            })
    
    output['shared_expressions'] = sorted(shared, 
                                         key=lambda x: sum(len(x['tags'][t]) for t in x['used_in']),
                                         reverse=True)[:20]
    
    # Output as JSON (sort_keys for deterministic output)
    print(json.dumps(output, indent=2, sort_keys=True))
    
    # Print summary to stderr
    print("\n=== Summary ===", file=sys.stderr)
    for expr_type in ['ValueConv', 'PrintConv', 'Condition']:
        info = output['expressions'][expr_type]
        print(f"\n{expr_type}:", file=sys.stderr)
        print(f"  Unique expressions: {info['unique_count']}", file=sys.stderr)
        print(f"  Total usage: {info['total_usage']}", file=sys.stderr)
        if 'patterns' in info:
            print(f"  Pattern breakdown:", file=sys.stderr)
            for pattern, (_, count) in sorted(info['patterns'].items(), 
                                             key=lambda x: x[1][1], reverse=True):
                print(f"    {pattern}: {count} tags", file=sys.stderr)

if __name__ == '__main__':
    main()