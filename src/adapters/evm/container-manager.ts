/** Container Manager - Phase 3.1 */
import { spawn, ChildProcess } from 'child_process';
import { EVMErrorCode } from './errors';
import { PORT_POOL, LOG_PREFIX } from './constants';

export interface IContainerInfo {
  id: string;
  port: number;
  process: ChildProcess;
}

export class ContainerManager {
  private containers = new Map<string, IContainerInfo>();
  private portPool = new Set<number>();

  constructor() {
    for (let p = PORT_POOL.MIN; p <= PORT_POOL.MAX; p++) {
      this.portPool.add(p);
    }
  }

  async allocatePort(): Promise<number> {
    for (const port of this.portPool) {
      if (!await this.isPortInUse(port)) {
        this.portPool.delete(port);
        return port;
      }
    }
    throw new Error(`${LOG_PREFIX} No available ports`);
  }

  async spawnContainer(image: string, args: string[], port: number): Promise<IContainerInfo> {
    const cmd = spawn('docker', [
      'run', '--rm', '-d', '-p', `${port}:${port}`,
      ...args, image
    ], { stdio: ['ignore', 'pipe', 'pipe'] });

    return new Promise((resolve, reject) => {
      let containerId = '';
      cmd.stdout?.on('data', (d) => { containerId += d.toString().trim(); });
      cmd.on('error', (e) => reject(new Error(`${LOG_PREFIX} ${EVMErrorCode.ContainerStartFailed}: ${e.message}`)));
      cmd.on('exit', (code) => {
        if (code === 0 && containerId) {
          const info: IContainerInfo = { id: containerId, port, process: cmd };
          this.containers.set(containerId, info);
          resolve(info);
        } else {
          reject(new Error(`${LOG_PREFIX} ${EVMErrorCode.ContainerStartFailed}`));
        }
      });
      setTimeout(() => reject(new Error(`${LOG_PREFIX} ${EVMErrorCode.ServiceTimeout}`)), 30000);
    });
  }

  async killContainer(id: string): Promise<void> {
    const info = this.containers.get(id);
    if (!info) return;
    try {
      spawn('docker', ['kill', id], { stdio: 'ignore' });
      info.process.kill();
    } finally {
      this.portPool.add(info.port);
      this.containers.delete(id);
    }
  }

  releasePort(port: number): void {
    this.portPool.add(port);
  }

  private async isPortInUse(port: number): Promise<boolean> {
    return new Promise((resolve) => {
      const check = spawn('powershell', ['-c', `Test-NetConnection -ComputerName localhost -Port ${port} -WarningAction SilentlyContinue | Select-Object -ExpandProperty TcpTestSucceeded`]);
      let out = '';
      check.stdout?.on('data', (d) => { out += d.toString(); });
      check.on('exit', () => resolve(out.includes('True')));
    });
  }
}
