/** Slither Parser - Phase 3.2 */
import { ISlitherResult } from './types';

export enum Severity {
  Critical = 'Critical',
  High = 'High',
  Medium = 'Medium',
  Low = 'Low',
}

export interface ParsedVulnerability {
  contract: string;
  vulnerability: string;
  line: number;
  severity: Severity;
  description: string;
  recommendation?: string;
}

export function parseSlitherJSON(json: string, contractName: string): ParsedVulnerability[] {
  try {
    const data = JSON.parse(json);
    const detectors = data.results?.detectors || [];
    if (detectors.length === 0) return [];

    return detectors.map((d: Record<string, unknown>) => {
      const elements = (d.elements as Array<{ source_mapping?: { lines: number[] } }>) || [];
      const line = elements[0]?.source_mapping?.lines?.[0] || 0;
      return {
        contract: contractName,
        vulnerability: String(d.check || 'Unknown'),
        line,
        severity: mapSeverity(String(d.impact)),
        description: String(d.description || ''),
        recommendation: generateRecommendation(String(d.check))
      };
    });
  } catch {
    return [];
  }
}

function mapSeverity(impact: string): Severity {
  switch (impact) {
    case 'High': return Severity.High;
    case 'Medium': return Severity.Medium;
    case 'Low': return Severity.Low;
    default: return Severity.Medium;
  }
}

function generateRecommendation(check: string): string {
  const recs: Record<string, string> = {
    'reentrancy-eth': 'Use ReentrancyGuard or checks-effects-interactions pattern',
    'access-control': 'Add proper access control modifiers',
    'unchecked-lowlevel': 'Check return value of low-level calls',
    'timestamp': 'Avoid using block.timestamp for critical logic'
  };
  return recs[check] || 'Review and fix according to best practices';
}
