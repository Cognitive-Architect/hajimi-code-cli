/** Build a unified diff string from original and modified text (simplified line-level). */
export function unifiedDiff(original: string, modified: string, uri: string): string {
  const origLines = original.split('\n');
  const modLines = modified.split('\n');
  const header = `--- a/${uri}\n+++ b/${uri}\n`;
  let diff = header;
  let i = 0;
  while (i < origLines.length || i < modLines.length) {
    if (i < origLines.length && i < modLines.length && origLines[i] === modLines[i]) { i++; continue; }
    const oldStart = i + 1, oldCount = origLines.length - i;
    const newStart = i + 1, newCount = modLines.length - i;
    diff += `@@ -${oldStart},${oldCount} +${newStart},${newCount} @@\n`;
    while (i < origLines.length || i < modLines.length) {
      if (i < origLines.length && i < modLines.length && origLines[i] === modLines[i]) { diff += ` ${origLines[i]}\n`; i++; }
      else if (i < origLines.length) { diff += `-${origLines[i]}\n`; i++; }
      else if (i < modLines.length) { diff += `+${modLines[i]}\n`; i++; }
    }
  }
  return diff;
}
