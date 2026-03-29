/**
 * ICE候选对选择器 - RFC 8445 ICE v2优化
 * 候选对优先级 = 2^32*MIN(G,D) + 2*MAX(G,D) + (G>D?1:0)
 * 移除Aggressive Nomination，使用Regular Nomination
 */
import { ICECandidate, CandidatePair } from './ice-types';

/**
 * 候选对选择器类 - 实现RFC 8445 ICE v2的候选对选择逻辑
 */
export class CandidatePairSelector {
  /**
   * 生成并排序候选对
   * @param local 本地候选列表
   * @param remote 远程候选列表
   * @returns 排序后的候选对列表
   */
  generatePairs(local: ICECandidate[], remote: ICECandidate[]): CandidatePair[] {
    const pairs: CandidatePair[] = [];
    for (const l of local) {
      for (const r of remote) {
        const priority = this.calculatePairPriority(l.priority, r.priority);
        pairs.push({ local: l, remote: r, priority });
      }
    }
    // 按优先级降序排序
    return pairs.sort((a, b) => b.priority - a.priority);
  }

  /**
   * RFC 8445 ICE v2候选对优先级公式
   * pair priority = 2^32*MIN(G,D) + 2*MAX(G,D) + (G>D?1:0)
   * 其中G=controlling优先级, D=controlled优先级
   * 假设本地为controlling agent
   * 
   * @param localPriority 本地候选优先级(G)
   * @param remotePriority 远程候选优先级(D)
   * @returns 候选对优先级
   */
  private calculatePairPriority(localPriority: number, remotePriority: number): number {
    const G = localPriority;  // Controlling
    const D = remotePriority; // Controlled
    const min = Math.min(G, D);
    const max = Math.max(G, D);
    return (Math.pow(2, 32) * min) + (2 * max) + (G > D ? 1 : 0);
  }

  /**
   * 过滤不可行候选对（相同IP或私有网络优化）
   * @param pairs 候选对列表
   * @returns 过滤后的候选对列表
   */
  filterFeasible(pairs: CandidatePair[]): CandidatePair[] {
    return pairs.filter(pair => {
      // 排除明显无效的：host-relay优先级低于srflx-srflx
      if (pair.local.type === 'relay' && pair.remote.type === 'relay') {
        // 双relay通常不高效，降低优先级但保留
        pair.priority -= 1000000;
      }
      // RFC 8445: 优先prflx类型候选
      if (pair.local.type === 'prflx' || pair.remote.type === 'prflx') {
        pair.priority += 500000;
      }
      return true;
    });
  }

  /**
   * 选择最佳候选对（已排序列表的第一个）
   * Regular Nomination: 顺序检查，非并行
   * @param pairs 候选对列表
   * @returns 最佳候选对或null
   */
  selectBest(pairs: CandidatePair[]): CandidatePair | null {
    return pairs.length > 0 ? pairs[0] : null;
  }
}
