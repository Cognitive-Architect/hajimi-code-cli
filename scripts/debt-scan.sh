#!/bin/bash
# 债务扫描脚本 - JSON输出

OUTPUT="target/debt-report.json"
mkdir -p target

echo "{\"scan_time\": \"$(date -Iseconds)\"," > "$OUTPUT"
echo "\"debts\": [" >> "$OUTPUT"

# 扫描unwrap
while IFS= read -r line; do
  echo "  {\"type\": \"unwrap\", \"location\": \"$line\"}," >> "$OUTPUT"
done < <(grep -rn "unwrap(" src --include="*.rs" | grep -v test)

# 扫描expect
while IFS= read -r line; do
  echo "  {\"type\": \"expect\", \"location\": \"$line\"}," >> "$OUTPUT"
done < <(grep -rn "expect(" src --include="*.rs" | grep -v test)

# 扫描unsafe
while IFS= read -r line; do
  echo "  {\"type\": \"unsafe\", \"location\": \"$line\"}," >> "$OUTPUT"
done < <(grep -rn "unsafe {" src --include="*.rs")

# 移除最后一个逗号
sed -i '$ s/,$//' "$OUTPUT"

echo "]," >> "$OUTPUT"
echo "\"summary\": {" >> "$OUTPUT"
echo "  \"total\": $(grep -c '"type"' "$OUTPUT")," >> "$OUTPUT"
echo "  \"status\": \"$(grep -c '"type"' "$OUTPUT" | awk '{print ($1==0 ? "debt-free" : "has-debt")}')\"" >> "$OUTPUT"
echo "}}" >> "$OUTPUT"

cat "$OUTPUT"
echo "Debt scan complete. Report: $OUTPUT"
