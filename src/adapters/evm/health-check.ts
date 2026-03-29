/** Health Check - Phase 3.1 */
import { spawn } from 'child_process';
import { IHealthStatus } from './types';
import { HEALTH_CHECK, LOG_PREFIX } from './constants';
import { EVMErrorCode } from './errors';

export async function checkDockerDaemon(): Promise<boolean> {
  return new Promise((resolve) => {
    const docker = spawn('docker', ['version'], { stdio: 'ignore' });
    docker.on('error', () => resolve(false));
    docker.on('exit', (code) => resolve(code === 0));
  });
}

export async function checkPort(port: number, retries = HEALTH_CHECK.RETRY_COUNT): Promise<boolean> {
  for (let i = 0; i < retries; i++) {
    try {
      const res = await fetch(`http://127.0.0.1:${port}`, { method: 'POST' });
      return res.status !== 0;
    } catch {
      await new Promise(r => setTimeout(r, HEALTH_CHECK.RETRY_DELAY_MS));
    }
  }
  return false;
}

export async function retry<T>(fn: () => Promise<T>, retries = HEALTH_CHECK.RETRY_COUNT): Promise<T> {
  let lastErr: Error | undefined;
  for (let i = 0; i < retries; i++) {
    try {
      return await fn();
    } catch (e) {
      lastErr = e instanceof Error ? e : new Error(String(e));
      await new Promise(r => setTimeout(r, HEALTH_CHECK.RETRY_DELAY_MS * (i + 1)));
    }
  }
  throw new Error(`${LOG_PREFIX} ${EVMErrorCode.ServiceTimeout}: ${lastErr?.message}`);
}

export async function checkHealth(port?: number): Promise<IHealthStatus> {
  return {
    docker: await checkDockerDaemon(),
    port: port ? await checkPort(port) : false,
    timestamp: Date.now(),
  };
}
