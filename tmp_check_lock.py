import re
with open('Cargo.lock', 'r') as f:
    content = f.read()
for m in re.finditer(r'name = "(parity-scale-codec[^"]*)"\nversion = "([^"]+)"', content):
    print(m.group(1) + ' = ' + m.group(2))
for m in re.finditer(r'name = "(scale-info[^"]*)"\nversion = "([^"]+)"', content):
    print(m.group(1) + ' = ' + m.group(2))
