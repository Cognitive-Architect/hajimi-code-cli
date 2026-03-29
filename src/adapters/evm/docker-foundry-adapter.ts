/** Docker Foundry Adapter - Phase 3.1 */
import { spawn } from 'child_process';
import { IDockerProvider, IAnvilInstance, IAnvilConfig, ISlitherResult } from './types';
import { DOCKER_IMAGE, DEFAULT_PORT, DEFAULT_CHAIN_ID, RPC_URL_TEMPLATE, PIPELINE_TIMEOUTS, LOG_PREFIX } from './constants';
import { EVMErrorCode, getErrorMessage } from './errors';
import { ContainerManager } from './container-manager';
import { checkDockerDaemon, retry, checkPort } from './health-check';

export class DockerFoundryProvider implements IDockerProvider {
  private containerManager: ContainerManager;
  private dockerProcess: ReturnType<typeof spawn> | null = null;

  constructor() { this.containerManager = new ContainerManager(); }

  async startAnvil(config?: IAnvilConfig): Promise<IAnvilInstance> {
    console.log(`${LOG_PREFIX} Starting Anvil...`);
    if (!await checkDockerDaemon()) throw new Error(getErrorMessage(EVMErrorCode.DockerNotRunning));

    const port = config?.port ?? await this.containerManager.allocatePort();
    const chainId = config?.chainId ?? DEFAULT_CHAIN_ID;
    const args = ['run', '--rm', '-d', '-p', `${port}:${port}`, DOCKER_IMAGE.FOUNDRY, 'anvil',
      '--host', '0.0.0.0', '--port', String(port), '--chain-id', String(chainId),
      ...(config?.forkUrl ? ['--fork-url', config.forkUrl] : []),
      ...(config?.blockTime ? ['--block-time', String(config.blockTime)] : []),
      '--accounts', String(config?.accounts ?? 10),
      '--balance', config?.balance ?? '10000'];

    this.dockerProcess = spawn('docker', args, { stdio: ['ignore', 'pipe', 'pipe'] });
    let containerId = '';
    this.dockerProcess.stdout?.on('data', (d) => { containerId += d.toString().trim(); });

    await new Promise<void>((resolve, reject) => {
      const to = setTimeout(() => reject(new Error(getErrorMessage(EVMErrorCode.ServiceTimeout))), PIPELINE_TIMEOUTS.DETECT);
      this.dockerProcess?.on('exit', (c) => { clearTimeout(to); c === 0 && containerId ? resolve() : reject(new Error(getErrorMessage(EVMErrorCode.ContainerStartFailed))); });
      this.dockerProcess?.on('error', (e) => { clearTimeout(to); reject(new Error(`${getErrorMessage(EVMErrorCode.ContainerStartFailed)}: ${e.message}`)); });
    });

    await retry(async () => { if (!await checkPort(port)) throw new Error('not ready'); });
    console.log(`${LOG_PREFIX} Anvil ready at ${RPC_URL_TEMPLATE}:${port}`);

    const instance: IAnvilInstance = {
      containerId, rpcUrl: `${RPC_URL_TEMPLATE}:${port}`, chainId, port, process: this.dockerProcess,
      stop: async () => { if (containerId) spawn('docker', ['kill', containerId], { stdio: 'ignore' }); this.dockerProcess?.kill(); this.containerManager.releasePort(port); },
      isRunning: () => this.dockerProcess?.exitCode === null
    };
    this.dockerProcess.on('exit', () => console.log(`${LOG_PREFIX} Anvil exited`));
    return instance;
  }

  async executeForge(args: string[], cwd?: string): Promise<string> {
    console.log(`${LOG_PREFIX} forge ${args.join(' ')}`);
    return new Promise((resolve, reject) => {
      const proc = spawn('docker', ['run', '--rm', '-v', `${cwd || process.cwd()}:/workspace`, '-w', '/workspace', DOCKER_IMAGE.FOUNDRY, 'forge', ...args], { stdio: ['ignore', 'pipe', 'pipe'] });
      let stdout = '', stderr = '';
      proc.stdout?.on('data', (d) => { stdout += d.toString(); });
      proc.stderr?.on('data', (d) => { stderr += d.toString(); });
      const to = setTimeout(() => { proc.kill(); reject(new Error(getErrorMessage(EVMErrorCode.ForgeError))); }, PIPELINE_TIMEOUTS.VERIFY);
      proc.on('exit', (c) => { clearTimeout(to); c === 0 ? resolve(stdout) : reject(new Error(`${getErrorMessage(EVMErrorCode.ForgeError)}: ${stderr || stdout}`)); });
    });
  }

  async executeSlither(contractPath: string): Promise<ISlitherResult[]> {
    console.log(`${LOG_PREFIX} slither ${contractPath}`);
    return new Promise((resolve) => {
      const proc = spawn('docker', ['run', '--rm', '-v', `${process.cwd()}:/workspace`, '-w', '/workspace', DOCKER_IMAGE.SLITHER, 'slither', contractPath, '--json', '-'], { stdio: ['ignore', 'pipe', 'pipe'] });
      let stdout = '';
      proc.stdout?.on('data', (d) => { stdout += d.toString(); });
      const to = setTimeout(() => { proc.kill(); resolve([]); }, PIPELINE_TIMEOUTS.DETECT);
      proc.on('exit', () => {
        clearTimeout(to);
        try { const p = JSON.parse(stdout); resolve((p.results?.detectors || []).map((d: Record<string, unknown>) => ({ contractName: contractPath, detector: String(d.check), impact: String(d.impact || 'Medium') as ISlitherResult['impact'], confidence: String(d.confidence || 'Medium') as ISlitherResult['confidence'], description: String(d.description), check: String(d.check) }))); }
        catch { resolve([]); }
      });
    });
  }
}
