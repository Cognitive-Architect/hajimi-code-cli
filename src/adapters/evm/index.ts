/** EVM Adapter Module Entry - Phase 3.0-3.3 Complete */

// Phase 3.0 Core Types
export {
  IAnvilInstance,
  IAnvilConfig,
  ISlitherResult,
  IVulnerabilitySample,
  IPipelineResult,
  IDockerProvider,
  IHealthStatus,
} from './types';

// Legacy Types (backward compatibility)
export { FoundryTestOutput, SlitherJsonOutput } from './types';

// Phase 3.1 Docker Provider
export { DockerFoundryProvider } from './docker-foundry-adapter';
export { ContainerManager, IContainerInfo } from './container-manager';
export { checkDockerDaemon, checkPort, retry, checkHealth } from './health-check';

// Phase 3.2 Slither Integration
export { SlitherDetector, ISlitherOptions, runSlither } from './slither-detector';
export { loadVulnSamples, filterBySeverity, getSampleByName } from './vuln-loader';
export { parseSlitherJSON, ParsedVulnerability, Severity } from './slither-parser';

// Phase 3.3 Pipeline
export { EVMPipeline, runPipeline, IPipelineConfig } from './evm-pipeline';
export { PatchGenerator, IPatch, generatePatch } from './patch-generator';
export { VerifyRunner, IVerifyResult } from './verify-runner';

// Constants
export {
  DOCKER_IMAGE,
  DEFAULT_PORT,
  DEFAULT_CHAIN_ID,
  DEFAULT_ACCOUNTS,
  DEFAULT_BALANCE,
  PORT_POOL,
  RPC_URL_TEMPLATE,
  HEALTH_CHECK,
  PIPELINE_TIMEOUTS,
  LOG_PREFIX,
} from './constants';

// Errors
export { EVMErrorCode, EVMErrorMessages, getErrorMessage } from './errors';

// Legacy Adapters
export { SlitherAdapter } from './slither-adapter';
export { FoundryAdapter } from './foundry-adapter';
