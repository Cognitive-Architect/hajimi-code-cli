import re
with open('Cargo.lock', 'r') as f:
    content = f.read()
for m in re.finditer(r'name = "(zstd[^"]*)"\nversion = "([^"]+)"', content):
    print(m.group(1) + ' = ' + m.group(2))
