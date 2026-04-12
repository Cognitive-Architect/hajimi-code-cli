/**
 * EVM-10 Dashboard
 * Markdown表格格式输出（CLI）
 */

import { ScoringResult } from './scorer';

/** Markdown表格生成器 */
export class Dashboard {
  /**
   * 生成Markdown表格
   * @param result 评分结果
   * @returns Markdown字符串
   */
  generateTable(result: ScoringResult): string {
    const lines: string[] = [];
    
    lines.push('# EVM Bench Dashboard');
    lines.push('');
    lines.push(`Generated: ${new Date().toISOString()}`);
    lines.push('');
    
    // 三模式成功率表格
    lines.push('## Success Rates');
    lines.push('');
    lines.push('| Mode | Success Rate | Rank |');
    lines.push('|------|-------------|------|');
    
    result.rankings.forEach((item, idx) => {
      const medal = idx === 0 ? '🥇' : idx === 1 ? '🥈' : '🥉';
      lines.push(`| ${item.mode} | ${item.score.toFixed(2)}% | ${idx + 1} ${medal} |`);
    });
    
    lines.push('');
    
    // 平均利润
    lines.push('## Profit Summary');
    lines.push('');
    lines.push(`**Average Profit**: ${result.avgProfit.toString()} wei`);
    lines.push('');
    
    // 原始数据统计
    lines.push('## Statistics');
    lines.push('');
    lines.push('| Metric | Count |');
    lines.push('|--------|-------|');
    lines.push(`| Detect Tests | ${result.rawData.detectResults.length} |`);
    lines.push(`| Patch Tests | ${result.rawData.patchResults.length} |`);
    lines.push(`| Exploit Tests | ${result.rawData.exploitResults.length} |`);
    lines.push('');
    
    // 排名详情
    lines.push('## Rankings');
    lines.push('');
    lines.push('```');
    result.rankings.forEach((r, i) => lines.push(`${i + 1}. ${r.mode}: ${r.score.toFixed(2)}%`));
    lines.push('```');
    
    return lines.join('\n');
  }
  
  /**
   * CLI输出
   * @param result 评分结果
   */
  render(result: ScoringResult): void {
    console.log(this.generateTable(result));
  }
}

/** 便捷函数：生成Dashboard */
export function generateDashboard(result: ScoringResult): string {
  return new Dashboard().generateTable(result);
}

/** 便捷函数：渲染Dashboard */
export function renderDashboard(result: ScoringResult): void {
  new Dashboard().render(result);
}
