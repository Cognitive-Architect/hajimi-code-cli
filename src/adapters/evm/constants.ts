/** EVM Adapter Constants - Phase 3.0 */

export const DOCKER_IMAGE = {
  FOUNDRY: 'ghcr.io/foundry-rs/foundry',
  SLITHER: 'trailofbits/slither',
  TAG: 'latest',
} as const;

export const DEFAULT_PORT = 8545;
export const DEFAULT_CHAIN_ID = 31337;
export const DEFAULT_ACCOUNTS = 10;
export const DEFAULT_BALANCE = '10000';

export const PORT_POOL = { MIN: 18545, MAX: 18645 } as const;
export const RPC_URL_TEMPLATE = 'http://127.0.0.1';

export const HEALTH_CHECK = {
  TIMEOUT_MS: 5000,
  RETRY_COUNT: 3,
  RETRY_DELAY_MS: 1000,
} as const;

export const PIPELINE_TIMEOUTS: Record<string, number> = {
  DETECT: 120000,
  PATCH: 60000,
  VERIFY: 180000,
};

export const LOG_PREFIX = '[EVM:Phase3.0]';
