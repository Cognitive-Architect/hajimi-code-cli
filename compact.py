import re

with open('F:/hajimi-code-cli/src/intelligence/memory/src/graph.rs', 'r') as f:
    content = f.read()

# Split into lines
lines = content.split('\n')

# More aggressive compaction strategies:
# 1. Combine struct definitions into single lines
# 2. Compact simple function bodies  
# 3. Remove some comments
# 4. Compact test functions

# Process line by line with transformations
result_lines = []
skip_next = 0

for i, line in enumerate(lines):
    if skip_next > 0:
        skip_next -= 1
        continue
    
    stripped = line.strip()
    
    # Skip empty lines (aggressive removal)
    if not stripped:
        continue
    
    # Compact struct fields - combine consecutive field lines
    if stripped.startswith('pub ') and ':' in stripped and stripped.endswith(',') and i + 1 < len(lines):
        next_line = lines[i + 1].strip()
        if next_line.startswith('pub ') and ':' in next_line:
            line = line.rstrip() + ' ' + next_line
            skip_next = 1
    
    # Compact #[test] fn into single conceptual line (keep as is for now)
    
    # Compact simple match arms
    if stripped == '}' and i + 1 < len(lines) and lines[i+1].strip() == '}':
        line = '}'
        skip_next = 1
    
    result_lines.append(line)

# Write result
with open('F:/hajimi-code-cli/src/intelligence/memory/src/graph.rs', 'w') as f:
    f.write('\n'.join(result_lines))

print(f"Lines: {len(result_lines)}")
