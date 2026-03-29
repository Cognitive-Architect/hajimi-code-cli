/** Patch Generator - Phase 3.3 */
import { LOG_PREFIX } from './constants';
import { ParsedVulnerability, Severity } from './slither-parser';

export interface IPatch {
  original: string;
  patched: string;
  description: string;
  lineStart: number;
  lineEnd: number;
}

export class PatchGenerator {
  generatePatch(code: string, vulns: ParsedVulnerability[]): IPatch[] {
    console.log(`${LOG_PREFIX} Generating ${vulns.length} patches...`);
    return vulns.map(v => this.createPatch(code, v)).filter((p): p is IPatch => p !== null);
  }

  private createPatch(code: string, vuln: ParsedVulnerability): IPatch | null {
    const lines = code.split('\n');
    const lineIdx = vuln.line - 1;
    if (lineIdx < 0 || lineIdx >= lines.length) return null;

    const contextStart = Math.max(0, lineIdx - 1);
    const contextEnd = Math.min(lines.length, lineIdx + 3);
    const original = lines.slice(contextStart, contextEnd).join('\n');

    let patched = original;
    const vType = vuln.vulnerability.toLowerCase();

    if (vType.includes('reentrancy')) {
      patched = this.fixReentrancy(lines, lineIdx);
    } else if (vType.includes('access')) {
      patched = this.fixAccessControl(lines, lineIdx);
    } else if (vType.includes('unchecked')) {
      patched = this.fixUncheckedCall(lines, lineIdx);
    } else if (vType.includes('timestamp')) {
      patched = this.fixTimestamp(lines, lineIdx);
    }

    return {
      original,
      patched,
      description: `Fix ${vuln.vulnerability}: ${vuln.description}`,
      lineStart: contextStart + 1,
      lineEnd: contextEnd
    };
  }

  private fixReentrancy(lines: string[], idx: number): string {
    return `// Apply checks-effects-interactions pattern
    uint256 amount = balances[msg.sender];
    balances[msg.sender] = 0;
    (bool sent, ) = msg.sender.call{value: amount}("");
    require(sent, "Transfer failed");`;
  }

  private fixAccessControl(lines: string[], idx: number): string {
    return `require(msg.sender == owner, "Not authorized");
    _;`;
  }

  private fixUncheckedCall(lines: string[], idx: number): string {
    return `(bool success, ) = recipient.call{value: amount}("");
    require(success, "Transfer failed");`;
  }

  private fixTimestamp(lines: string[], idx: number): string {
    return `// Avoid block.timestamp for randomness
    uint256 random = uint256(keccak256(abi.encodePacked(blockhash(block.number - 1))));`;
  }
}

export function generatePatch(code: string, vuln: ParsedVulnerability): IPatch {
  return new PatchGenerator().generatePatch(code, [vuln])[0];
}
