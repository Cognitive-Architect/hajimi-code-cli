/**
 * ICE延迟监控器 - RTT平滑算法
 * RFC 8445 ICE v2改进
 * smoothedRtt = 0.875 * oldRtt + 0.125 * newRtt
 */
export class LatencyMonitor {
  private smoothedRtt: number = 0;
  private rttVariance: number = 0;
  private lastRtt: number = 0;
  private sampleCount: number = 0;

  /**
   * 记录RTT样本，应用平滑算法
   */
  recordRtt(rtt: number): void {
    this.lastRtt = rtt;
    this.sampleCount++;

    if (this.sampleCount === 1) {
      this.smoothedRtt = rtt;
      this.rttVariance = rtt / 2;
    } else {
      // RTT平滑：指数加权移动平均
      const alpha = 0.125;  // 平滑系数
      const beta = 0.25;    // 偏差系数

      const diff = Math.abs(rtt - this.smoothedRtt);
      this.rttVariance = (1 - beta) * this.rttVariance + beta * diff;
      this.smoothedRtt = (1 - alpha) * this.smoothedRtt + alpha * rtt;
    }
  }

  getSmoothedRtt(): number {
    return this.smoothedRtt;
  }

  getRttVariance(): number {
    return this.rttVariance;
  }

  /**
   * 计算重传超时(RTO)
   * RTO = smoothedRtt + 4*rttVariance
   */
  getRetransmissionTimeout(): number {
    return this.smoothedRtt + 4 * this.rttVariance;
  }

  reset(): void {
    this.smoothedRtt = 0;
    this.rttVariance = 0;
    this.sampleCount = 0;
  }
}
