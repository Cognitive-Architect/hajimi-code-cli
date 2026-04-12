/**
 * EVM-11 Report Generator
 * JSON/Markdown双格式文件写入
 */

import * as fs from 'fs';
import * as path from 'path';
import { ScoringResult } from './scorer';
import { Dashboard } from './dashboard';

interface ReportOutput {
  markdownPath: string;
  jsonPath: string;
  timestamp: string;
}

class ReportError extends Error {
  constructor(msg: string) { super(`ReportError: ${msg}`); }
}

export class ReportGenerator {
  private outputDir: string;
  
  constructor(outputDir: string = 'reports') {
    this.outputDir = outputDir;
  }
  
  /**
   * 生成双格式报告
   * @param result 评分结果
   * @returns 输出文件路径
   */
  generate(result: ScoringResult): ReportOutput {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    
    // 确保目录存在
    if (!fs.existsSync(this.outputDir)) {
      fs.mkdirSync(this.outputDir, { recursive: true });
    }
    
    // Markdown报告
    const dashboard = new Dashboard();
    const markdown = dashboard.generateTable(result);
    const mdPath = path.join(this.outputDir, `report-${timestamp}.md`);
    
    // 检查文件是否存在（防覆盖）
    let finalMdPath = mdPath;
    let counter = 1;
    while (fs.existsSync(finalMdPath)) {
      finalMdPath = path.join(this.outputDir, `report-${timestamp}-${counter}.md`);
      counter++;
    }
    
    fs.writeFileSync(finalMdPath, markdown, 'utf8');
    console.log(`[Report] Markdown: ${finalMdPath}`);
    
    // JSON报告（处理bigint）
    const jsonPath = finalMdPath.replace('.md', '.json');
    const jsonData = JSON.stringify(result, (key, value) => {
      if (typeof value === 'bigint') {
        return value.toString();
      }
      return value;
    }, 2);
    
    fs.writeFileSync(jsonPath, jsonData, 'utf8');
    console.log(`[Report] JSON: ${jsonPath}`);
    
    return {
      markdownPath: finalMdPath,
      jsonPath: jsonPath,
      timestamp: new Date().toISOString()
    };
  }
}

/** 便捷函数 */
export function generateReport(result: ScoringResult, outputDir?: string): ReportOutput {
  return new ReportGenerator(outputDir).generate(result);
}
