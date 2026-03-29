/** EVM Adapter Error Codes - Phase 3.0 */

export enum EVMErrorCode {
  DockerNotRunning = 1001,
  PortInUse = 1002,
  ImageNotFound = 1003,
  ContainerStartFailed = 1004,
  ContainerCrashed = 1005,
  ServiceTimeout = 1006,
  InvalidConfig = 1007,
  RPCError = 1008,
  SlitherError = 1009,
  ForgeError = 1010,
}

export const EVMErrorMessages: Record<EVMErrorCode, string> = {
  [EVMErrorCode.DockerNotRunning]: 'Docker daemon is not running',
  [EVMErrorCode.PortInUse]: 'Required port is already in use',
  [EVMErrorCode.ImageNotFound]: 'Docker image not found',
  [EVMErrorCode.ContainerStartFailed]: 'Failed to start container',
  [EVMErrorCode.ContainerCrashed]: 'Container crashed',
  [EVMErrorCode.ServiceTimeout]: 'Service timeout',
  [EVMErrorCode.InvalidConfig]: 'Invalid configuration',
  [EVMErrorCode.RPCError]: 'RPC call failed',
  [EVMErrorCode.SlitherError]: 'Slither analysis failed',
  [EVMErrorCode.ForgeError]: 'Forge execution failed',
};

export function getErrorMessage(code: EVMErrorCode): string {
  return EVMErrorMessages[code] || `Unknown error: ${code}`;
}
