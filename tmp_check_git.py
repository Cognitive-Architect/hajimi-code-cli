import sys, re, subprocess
result = subprocess.run(['git', 'show', 'HEAD:Cargo.lock'], capture_output=True, text=True)
content = result.stdout
for m in re.finditer(r'name = "(zstd[^"]*)"\nversion = "([^"]+)"', content):
    print(m.group(1) + ' = ' + m.group(2))
