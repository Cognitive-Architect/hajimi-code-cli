/**
 * EVM-09 Scoring Algorithm
 * Detect/Patch/Exploit三模式成功率计算与排名
 */

import { ExploitResult } from './types';

/** 原始评分数据 */
interface RawData {
  detectResults: Array<{ vulnId: string; detected: boolean }>;
  patchResults: Array<{ vulnId: string; patched: boolean; gasUsed: bigint }>;
  exploitResults: Array<{ vulnId: string; profit: bigint; success: boolean }>;
}

/** 评分结果输出 */
export interface ScoringResult {
  detectRate: number;
  patchRate: number;
  exploitRate: number;
  avgProfit: bigint;
  rankings: Array<{ mode: string; score: number }>;
  rawData: RawData;
}

/** 评分器主类 */
export class Scorer {
  /**
   * 计算成功率（百分比，保留2位小数）
   * @param successes 成功次数
   * @param total 总次数
   * @returns 成功率百分比
   */
  calculateSuccessRate(successes: number, total: number): number {
    if (total === 0) {
      console.warn('[Scorer] Zero total, returning 0%');
      return 0;
    }
    return Math.round((successes / total) * 10000) / 100;
  }

  /**
   * 计算平均利润
   * @param profits 利润数组
   * @returns 平均利润
   */
  computeAverageProfit(profits: bigint[]): bigint {
    if (profits.length === 0) return BigInt(0);
    return profits.reduce((a, b) => a + b, BigInt(0)) / BigInt(profits.length);
  }

  /**
   * 执行完整评分
   * @param rawData 原始数据
   * @returns 评分结果
   */
  score(rawData: RawData): ScoringResult {
    console.log('[Scorer] Calculating scores...');

    // Detect评分
    const detectTotal = rawData.detectResults?.length || 0;
    const detectSuccess = rawData.detectResults?.filter(r => r.detected).length || 0;
    const detectRate = this.calculateSuccessRate(detectSuccess, detectTotal);

    // Patch评分
    const patchTotal = rawData.patchResults?.length || 0;
    const patchSuccess = rawData.patchResults?.filter(r => r.patched).length || 0;
    const patchRate = this.calculateSuccessRate(patchSuccess, patchTotal);

    // Exploit评分
    const exploitTotal = rawData.exploitResults?.length || 0;
    const exploitSuccess = rawData.exploitResults?.filter(r => r.success).length || 0;
    const exploitRate = this.calculateSuccessRate(exploitSuccess, exploitTotal);

    // 平均利润（仅成功项）
    const profits = rawData.exploitResults?.filter(r => r.success).map(r => r.profit) || [];
    const avgProfit = this.computeAverageProfit(profits);

    // 生成排名
    const rankings = [
      { mode: 'Detect', score: detectRate },
      { mode: 'Patch', score: patchRate },
      { mode: 'Exploit', score: exploitRate }
    ].sort((a, b) => b.score - a.score);

    console.log(`[Scorer] Detect:${detectRate}% Patch:${patchRate}% Exploit:${exploitRate}%`);

    return { detectRate, patchRate, exploitRate, avgProfit, rankings, rawData };
  }
}

/** 便捷函数：计算评分 */
export function calculateScores(rawData: RawData): ScoringResult {
  return new Scorer().score(rawData);
}

/** 便捷函数：序列化（处理bigint） */
export function serializeScoringResult(result: ScoringResult): string {
  return JSON.stringify(result, (k, v) => typeof v === 'bigint' ? v.toString() + 'n' : v, 2);
}
