with open('F:/hajimi-code-cli/src/intelligence/memory/src/graph.rs', 'r') as f:
    lines = f.readlines()

# Strategy: Remove all blank lines and compact further
compact = []
for line in lines:
    stripped = line.rstrip()
    if stripped:  # Keep only non-empty lines
        compact.append(stripped)

# Now combine specific patterns
result = []
i = 0
while i < len(compact):
    line = compact[i]
    
    # Combine opening brace with content
    if line.endswith('{') and i + 1 < len(compact) and not compact[i+1].startswith('//') and not compact[i+1].startswith('#['):
        # Check if this is a simple function/impl that can be compacted
        if 'fn ' in line or 'impl ' in line or 'match ' in line:
            result.append(line)
            i += 1
            continue
    
    # Combine struct fields aggressively
    if i + 1 < len(compact) and compact[i+1].strip().startswith('pub ') and ':' in compact[i+1] and ',' in line:
        line = line + ' ' + compact[i+1].strip()
        i += 2
        result.append(line)
        continue
    
    result.append(line)
    i += 1

with open('F:/hajimi-code-cli/src/intelligence/memory/src/graph.rs', 'w') as f:
    for line in result:
        f.write(line + '\n')

print(f"Lines: {len(result)}")
