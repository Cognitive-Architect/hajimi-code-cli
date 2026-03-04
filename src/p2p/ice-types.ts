/**
 * ICE类型定义 - RFC 8445 ICE v2
 */

export interface ICECandidate {
  type: 'host' | 'prflx' | 'srflx' | 'relay';
  ip: string;
  port: number;
  priority: number;
}

export interface CandidatePair {
  local: ICECandidate;
  remote: ICECandidate;
  priority: number;
}

export interface STUNResponse {
  success: boolean;
  mappedAddress?: { ip: string; port: number };
}
