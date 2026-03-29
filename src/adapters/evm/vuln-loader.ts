/** Vulnerability Loader - Phase 3.2 */
import { readFileSync } from 'fs';
import { resolve } from 'path';
import { IVulnerabilitySample } from './types';
import { LOG_PREFIX } from './constants';

const SAMPLES_PATH = resolve(process.cwd(), 'docs/intel/evm-vuln-samples.json');

export function loadVulnSamples(): IVulnerabilitySample[] {
  console.log(`${LOG_PREFIX} Loading vulnerability samples...`);
  try {
    const data = readFileSync(SAMPLES_PATH, 'utf8');
    const samples: IVulnerabilitySample[] = JSON.parse(data);
    console.log(`${LOG_PREFIX} Loaded ${samples.length} samples`);
    return samples;
  } catch (e) {
    console.error(`${LOG_PREFIX} Failed to load samples:`, e);
    return [];
  }
}

export function filterBySeverity(samples: IVulnerabilitySample[], severity: IVulnerabilitySample['severity']): IVulnerabilitySample[] {
  return samples.filter(s => s.severity === severity);
}

export function getSampleByName(name: string): IVulnerabilitySample | undefined {
  return loadVulnSamples().find(s => s.contractName === name);
}
